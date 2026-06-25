// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Reusable buffered converter engine.

use super::super::internal::{
    convert_error_of::ConvertErrorOf,
    convert_state::ConvertState,
    lifecycle::LifecycleGuard,
    pending_value::PendingValue,
    pending_value_slot::PendingValueSlot,
};
use super::{
    transcode_decode_engine::TranscodeDecodeEngine,
    transcode_encode_engine::TranscodeEncodeEngine,
};
use crate::codec::assert_unit_bounds;
use crate::{
    CapacityError,
    Codec,
    CodecDecodeFlushError,
    CodecDecodeResetError,
    CodecEncodeFlushError,
    CodecEncodeResetError,
    EncodeContext,
    TranscodeConvertHooks,
    TranscodeDecodeHooks,
    TranscodeEncodeHooks,
    TranscodeError,
    TranscodeProgress,
    Transcoder,
};

/// Reusable buffered conversion engine.
///
/// The engine owns reusable buffered decode and encode engines plus a small
/// conversion-level error mapper. It keeps common converter control flow
/// private: index validation, pending-value retention, pending flush,
/// decode-error policy dispatch, encode attempts, output-capacity checks, and
/// progress reporting.
///
/// `TranscodeConvertEngine` is intentionally batch-oriented. Its public
/// `transcode` method drives a source/output buffer loop and reuses the same
/// unchecked codec and hook primitives as [`crate::TranscodeDecodeEngine`] and
/// [`crate::TranscodeEncodeEngine`]. It does not call one-value public
/// transcoders in the hot path.
///
/// # Type Parameters
///
/// - `D`: Source-side decoder codec.
/// - `E`: Target-side encoder codec.
/// - `DH`: Source-side decode hooks.
/// - `EH`: Target-side encode hooks.
/// - `H`: Conversion-level error mapper.
#[derive(Debug)]
pub struct TranscodeConvertEngine<D, E, DH, EH, H>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    DH: TranscodeDecodeHooks<D>,
    EH: TranscodeEncodeHooks<E>,
    H: TranscodeConvertHooks<
            D,
            E,
            DecodeError = DH::Error,
            EncodeError = EH::Error,
        >,
{
    /// Source-side buffered decoder engine.
    decode_engine: TranscodeDecodeEngine<D, DH>,
    /// Target-side buffered encoder engine.
    encode_engine: TranscodeEncodeEngine<E, EH>,
    /// Conversion-level error mapper.
    hooks: H,
    /// Decoded value waiting for target output capacity.
    pending: PendingValueSlot<D::Value>,
    /// Debug-only guard for the `reset → transcode* → finish` lifecycle.
    /// Zero-sized in release builds. The converter owns its own guard rather
    /// than delegating to the inner decode/encode engines, because lifecycle
    /// events here describe the converter as a whole.
    lifecycle: LifecycleGuard,
}

impl<D, E, DH, EH, H> TranscodeConvertEngine<D, E, DH, EH, H>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    DH: TranscodeDecodeHooks<D>,
    EH: TranscodeEncodeHooks<E>,
    H: TranscodeConvertHooks<
            D,
            E,
            DecodeError = DH::Error,
            EncodeError = EH::Error,
        >,
{
    /// Creates a buffered converter engine.
    ///
    /// The caller supplies decode hooks, encode hooks, and the converter-level
    /// error mapper directly.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Low-level codec used for source decoding.-
    /// - `encoder`: Low-level codec used for target encoding.
    /// - `decode_hooks`: Decode-side policy hooks.
    /// - `encode_hooks`: Encode-side policy hooks.
    /// - `hooks`: Conversion-level error mapper.
    ///
    /// # Returns
    ///
    /// Returns a buffered converter engine.
    /// # Panics
    ///
    /// In debug builds, panics when either codec violates the
    /// [`Codec::MIN_UNITS_PER_VALUE`] / [`Codec::MAX_UNITS_PER_VALUE`] ordering
    /// invariant. Release builds skip this check because the invariant is the
    /// responsibility of each [`Codec`] implementation.
    #[inline]
    #[must_use]
    pub fn new(
        decoder: D,
        encoder: E,
        decode_hooks: DH,
        encode_hooks: EH,
        hooks: H,
    ) -> Self {
        assert_unit_bounds::<D>();
        assert_unit_bounds::<E>();
        Self {
            decode_engine: TranscodeDecodeEngine::new(decoder, decode_hooks),
            encode_engine: TranscodeEncodeEngine::new(encoder, encode_hooks),
            hooks,
            pending: PendingValueSlot::empty(),
            lifecycle: LifecycleGuard::new(),
        }
    }

    /// Returns an upper bound for target units produced from `input_len` units.
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        let pending_units = self.pending_output_len()?;
        let decoded_values = self.decode_engine.max_output_len(input_len)?;
        let converted_units =
            self.encode_engine.max_output_len(decoded_values)?;
        converted_units
            .checked_add(pending_units)
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    /// Returns the maximum target units emitted when resetting stream state.
    ///
    /// Covers decode-side reset values (encoded to target units) plus
    /// encode-side reset units. Most codecs are stateless and return `0`
    /// for `MAX_DECODE_RESET_VALUES`; in that case this equals the encode
    /// reset bound only.
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        let decode_reset_units = self
            .encode_engine
            .max_output_len(D::MAX_DECODE_RESET_VALUES)?;
        let encode_reset_units = self.encode_engine.max_reset_output_len()?;
        decode_reset_units
            .checked_add(encode_reset_units)
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    /// Returns the maximum target units emitted by finishing retained state.
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        let pending_units = self.pending_output_len()?;
        let decoder_finish_values =
            self.decode_engine.max_finish_output_len()?;
        let decoder_finish_units =
            self.encode_engine.max_output_len(decoder_finish_values)?;
        let encoder_finish_units =
            self.encode_engine.max_finish_output_len()?;
        let pending_and_decoder = pending_units
            .checked_add(decoder_finish_units)
            .ok_or(CapacityError::OutputLengthOverflow)?;
        pending_and_decoder
            .checked_add(encoder_finish_units)
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    /// Converts source units into target units.
    ///
    /// # Parameters
    ///
    /// - `input`: Complete input unit slice visible to the converter.
    /// - `input_index`: Absolute input index where conversion starts.
    /// - `output`: Complete output unit slice visible to the converter.
    /// - `output_index`: Absolute output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns conversion progress.
    ///
    /// # Errors
    ///
    /// Returns hook errors when indices are invalid or concrete conversion
    /// fails. Invalid output indices are reported through the encode-side
    /// error path.
    pub fn transcode(
        &mut self,
        input: &[D::Unit],
        input_index: usize,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, ConvertErrorOf<D, E, H>> {
        self.lifecycle.on_transcode();
        TranscodeError::ensure_transcode_indices(
            input.len(),
            input_index,
            output.len(),
            output_index,
        )?;

        let mut state =
            ConvertState::new(input, input_index, output, output_index);

        // A retained decoded value must be written before consuming more input,
        // otherwise callers could observe output reordered across buffer turns.
        if let Some(progress) = self.drain_pending(&mut state)? {
            return Ok(progress);
        }

        let min_input_units = D::MIN_UNITS_PER_VALUE;
        while state.has_input() {
            let available = state.available_input();
            if available < min_input_units.get() {
                return Ok(
                    state.need_input_progress(min_input_units, available)
                );
            }

            let previous_read = state.read();
            // Each hot-path step decodes one source value and immediately tries
            // to encode it, preserving backpressure at the target output.
            if let Some(progress) = self.convert_next(&mut state)? {
                return Ok(progress);
            }
            debug_assert!(
                state.read() > previous_read,
                "TranscodeConvertEngine conversion step must consume input or stop",
            );
        }

        Ok(state.complete_progress())
    }

    /// Finishes retained output after EOF.
    ///
    /// Finalization drains a pending decoded value first, then lets the
    /// source-side decode hooks emit final values, encodes those values through
    /// the target-side encode hooks, and finally finishes target-side encode
    /// hook state. The decode-finish value buffer used for this cold path
    /// requires `D::Value: Default`; the normal `transcode` loop does not.
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output unit slice visible to the converter.
    /// - `output_index`: Absolute output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of target units written during finalization.
    ///
    /// # Errors
    ///
    /// Returns a converter error when output capacity checks fail or when
    /// hook finalization fails.
    pub fn finish(
        &mut self,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<usize, ConvertErrorOf<D, E, H>>
    where
        D::Value: Default,
        DH::Error: From<CodecDecodeFlushError<D::DecodeError>>,
        EH::Error: From<CodecEncodeFlushError<E::EncodeError>>,
    {
        self.lifecycle.on_finish_attempt();
        let required = self.max_finish_output_len()?;
        TranscodeError::ensure_output_capacity(
            output.len(),
            output_index,
            required,
        )?;

        let empty_input: &[D::Unit] = &[];
        let mut state = ConvertState::new(empty_input, 0, output, output_index);
        // Finish keeps the same priority as transcode: output any retained
        // decoded value before asking source-side hooks for final values.
        if self.drain_pending(&mut state)?.is_some() {
            unreachable!(
                "converter finish bound must reserve space for pending values"
            );
        }

        // Source-side finish may emit one or more final values. Drain them into
        // the target encoder before finishing target-side hook state.
        self.drain_decoder_finish(&mut state)?;

        let output_cursor = state.output_cursor();
        let written = self
            .encode_engine
            .finish(state.output_mut(), output_cursor)
            .map_err(|error| {
                error.map_domain(|domain| self.hooks.map_encode_error(domain))
            })?;
        state.advance_output(written);
        self.lifecycle.on_finish_success();
        Ok(state.written())
    }

    /// Clears retained conversion state, runs before-reset hooks, and emits
    /// stream-start encode output.
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output unit slice visible to the converter.
    /// - `output_index`: Absolute output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of target units written while resetting stream state.
    ///
    /// # Errors
    ///
    /// Returns a converter error if reset validation or target reset output
    /// emission fails.
    pub fn reset(
        &mut self,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<usize, ConvertErrorOf<D, E, H>>
    where
        D::Value: Default,
        DH::Error: From<CodecDecodeResetError<D::DecodeError>>,
        EH::Error: From<CodecEncodeResetError<E::EncodeError>>,
    {
        self.lifecycle.on_reset();
        let required = self.max_reset_output_len()?;
        TranscodeError::ensure_output_capacity(
            output.len(),
            output_index,
            required,
        )?;

        self.pending.clear();
        self.hooks.reset_hooks();

        // Source-side reset may emit stream-start values (such as a BOM) that
        // must be piped through the target encoder before any encoder-owned
        // reset output. `max_reset_output_len` already reserves space for both
        // halves of the pipeline, so encode_pending should never report
        // `NeedOutput` here.
        let empty_input: &[D::Unit] = &[];
        let mut state = ConvertState::new(empty_input, 0, output, output_index);
        self.drain_decoder_reset(&mut state)?;

        let output_cursor = state.output_cursor();
        let encoder_written = self
            .encode_engine
            .reset(state.output_mut(), output_cursor)
            .map_err(|error| {
                error.map_domain(|domain| self.hooks.map_encode_error(domain))
            })?;
        state.advance_output(encoder_written);
        Ok(state.written())
    }

    /// Drains source-side decode reset output and encodes emitted reset
    /// values.
    fn drain_decoder_reset(
        &mut self,
        state: &mut ConvertState<'_, D::Unit, E::Unit>,
    ) -> Result<(), ConvertErrorOf<D, E, H>>
    where
        D::Value: Default,
        DH::Error: From<CodecDecodeResetError<D::DecodeError>>,
    {
        let value_count = D::MAX_DECODE_RESET_VALUES;
        if value_count == 0 {
            // Stateless decoder: still call decode_reset so codecs whose
            // hooks own teardown side effects (e.g. clearing accumulators)
            // run them. The empty slice is safe because the capacity check
            // inside `reset` accepts `required == 0` against any slice.
            self.decode_engine.reset(&mut [], 0).map_err(|error| {
                error.map_domain(|domain| self.hooks.map_decode_error(domain))
            })?;
            return Ok(());
        }
        // `D::Value: Default` is only consulted when the decoder declares
        // reset output. Stateless codecs never reach this branch.
        let mut reset_values: Vec<D::Value> =
            (0..value_count).map(|_| D::Value::default()).collect();
        let written = self.decode_engine.reset(&mut reset_values, 0).map_err(
            |error| {
                error.map_domain(|domain| self.hooks.map_decode_error(domain))
            },
        )?;
        for value in reset_values.into_iter().take(written) {
            let pending = PendingValue::new(value, 0);
            if self.encode_pending(pending, state)?.is_some() {
                unreachable!(
                    "converter reset bound must reserve space for decode reset values"
                );
            }
        }
        Ok(())
    }

    /// Converts one value from the current state cursors.
    #[inline(always)]
    fn convert_next(
        &mut self,
        state: &mut ConvertState<'_, D::Unit, E::Unit>,
    ) -> Result<Option<TranscodeProgress>, ConvertErrorOf<D, E, H>> {
        let step = self
            .decode_engine
            .decode_step(state.input(), state.decode_context())
            .map_err(|error| {
                error.map_domain(|domain| self.hooks.map_decode_error(domain))
            })?;
        state.apply_decode_step(step, |pending, state| {
            self.encode_pending(pending, state)
        })
    }

    /// Returns the output bound for the retained pending value.
    #[inline(always)]
    fn pending_output_len(&self) -> Result<usize, CapacityError> {
        self.pending.max_output_len(&self.encode_engine)
    }

    /// Writes a retained decoded value before new input is consumed.
    #[inline(always)]
    fn drain_pending(
        &mut self,
        state: &mut ConvertState<'_, D::Unit, E::Unit>,
    ) -> Result<Option<TranscodeProgress>, ConvertErrorOf<D, E, H>> {
        let Some(pending) = self.pending.take() else {
            return Ok(None);
        };
        self.encode_pending(pending, state)
    }

    /// Drains source-side decode finish output and encodes emitted final
    /// values.
    fn drain_decoder_finish(
        &mut self,
        state: &mut ConvertState<'_, D::Unit, E::Unit>,
    ) -> Result<(), ConvertErrorOf<D, E, H>>
    where
        D::Value: Default,
        DH::Error: From<CodecDecodeFlushError<D::DecodeError>>,
    {
        let value_count = self.decode_engine.max_finish_output_len()?;
        if value_count == 0 {
            // Skip the Vec allocation when the decoder declares no finish
            // output. We still call finish() so that
            // codec.decode_flush and hooks.finish_hooks both run —
            // hooks may do validation or teardown (e.g. checksum
            // verification) that can fail even when emitting zero
            // values. Passing an empty slice is safe here because the capacity
            // check inside finish() accepts required == 0 against any slice.
            self.decode_engine.finish(&mut [], 0).map_err(|error| {
                error.map_domain(|domain| self.hooks.map_decode_error(domain))
            })?;
            return Ok(());
        }
        // D::Value: Default is required only when value_count > 0. The bound
        // remains on the method signature for the general case; stateless
        // codecs never reach this branch.
        let mut decoded: Vec<D::Value> =
            (0..value_count).map(|_| D::Value::default()).collect();
        let written =
            self.decode_engine
                .finish(&mut decoded, 0)
                .map_err(|error| {
                    error.map_domain(|domain| {
                        self.hooks.map_decode_error(domain)
                    })
                })?;
        for value in decoded.into_iter().take(written) {
            let pending = PendingValue::new(value, 0);
            if self.encode_pending(pending, state)?.is_some() {
                unreachable!(
                    "converter finish bound must reserve space for decode finish values"
                );
            }
        }
        Ok(())
    }

    /// Encodes one pending value and applies output/pending state changes.
    fn encode_pending(
        &mut self,
        pending: PendingValue<D::Value>,
        state: &mut ConvertState<'_, D::Unit, E::Unit>,
    ) -> Result<Option<TranscodeProgress>, ConvertErrorOf<D, E, H>> {
        let input_index = pending.input_index();
        let output_index = state.output_cursor();
        let context = EncodeContext::new(
            pending.value(),
            input_index,
            state.output_mut(),
            output_index,
        );
        let outcome =
            self.encode_engine.encode_one(context).map_err(|error| {
                TranscodeError::domain(self.hooks.map_encode_error(error))
            })?;
        let progress = state.apply_encode_outcome(outcome);
        if progress.is_some() {
            self.pending.put(pending);
        }
        Ok(progress)
    }
}

impl<D, E, DH, EH, H> Default for TranscodeConvertEngine<D, E, DH, EH, H>
where
    D: Codec + Default,
    E: Codec<Value = D::Value> + Default,
    DH: TranscodeDecodeHooks<D> + Default,
    EH: TranscodeEncodeHooks<E> + Default,
    H: TranscodeConvertHooks<
            D,
            E,
            DecodeError = DH::Error,
            EncodeError = EH::Error,
        > + Default,
{
    /// Creates a default buffered converter engine.
    ///
    /// # Returns
    ///
    /// Returns a converter engine constructed from default codecs and hooks.
    #[inline(always)]
    fn default() -> Self {
        Self::new(
            D::default(),
            E::default(),
            DH::default(),
            EH::default(),
            H::default(),
        )
    }
}

impl<D, E, DH, EH, H> Transcoder<D::Unit, E::Unit>
    for TranscodeConvertEngine<D, E, DH, EH, H>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    D::Value: Default,
    DH: TranscodeDecodeHooks<D>,
    EH: TranscodeEncodeHooks<E>,
    DH::Error: From<CodecDecodeResetError<D::DecodeError>>,
    DH::Error: From<CodecDecodeFlushError<D::DecodeError>>,
    EH::Error: From<CodecEncodeResetError<E::EncodeError>>,
    EH::Error: From<CodecEncodeFlushError<E::EncodeError>>,
    H: TranscodeConvertHooks<
            D,
            E,
            DecodeError = DH::Error,
            EncodeError = EH::Error,
        >,
{
    type Error = H::Error;

    /// Returns an upper bound for target units produced from `input_len` units.
    #[inline(always)]
    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        TranscodeConvertEngine::max_output_len(self, input_len)
    }

    /// Returns the maximum target units emitted by finishing retained state.
    #[inline(always)]
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        TranscodeConvertEngine::max_finish_output_len(self)
    }

    /// Returns the maximum target units emitted when resetting stream state.
    #[inline(always)]
    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        TranscodeConvertEngine::max_reset_output_len(self)
    }

    /// Clears retained conversion state and emits target reset output.
    #[inline(always)]
    fn reset(
        &mut self,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        TranscodeConvertEngine::reset(self, output, output_index)
    }

    /// Converts source units into target units.
    #[inline(always)]
    fn transcode(
        &mut self,
        input: &[D::Unit],
        input_index: usize,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
        TranscodeConvertEngine::transcode(
            self,
            input,
            input_index,
            output,
            output_index,
        )
    }

    /// Finishes retained converter output after EOF.
    #[inline(always)]
    fn finish(
        &mut self,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        TranscodeConvertEngine::finish(self, output, output_index)
    }
}
