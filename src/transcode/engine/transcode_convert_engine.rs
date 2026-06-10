// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Reusable buffered converter engine.

use crate::{CapacityError, Codec, EncodeContext, FinishError, TranscodeConvertHooks};
use super::{transcode_decode_engine::TranscodeDecodeEngine, transcode_encode_engine::TranscodeEncodeEngine};
use super::super::internal::{
    convert_error_of::ConvertErrorOf,
    convert_progress_result::ConvertProgressResult,
    convert_state::ConvertState,
    convert_step_result::ConvertStepResult,
    encode_step::EncodeStep,
    pending_encode_step::PendingEncodeStep,
    pending_value::PendingValue,
    pending_value_slot::PendingValueSlot,
};
use crate::core::assert_unit_bounds;

/// Reusable buffered conversion engine.
///
/// The engine owns reusable buffered decode and encode engines plus a small
/// conversion-level hook object. It keeps common converter control flow
/// private: index validation, pending-value retention, pending flush,
/// decode-error policy dispatch, encode planning, output-capacity checks, and
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
/// - `H`: Conversion-level policy hooks.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TranscodeConvertEngine<D, E, H>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    H: TranscodeConvertHooks<D, E>,
{
    /// Source-side buffered decoder engine.
    decode_engine: TranscodeDecodeEngine<D, H::DecodeHooks>,
    /// Target-side buffered encoder engine.
    encode_engine: TranscodeEncodeEngine<E, H::EncodeHooks>,
    /// Conversion-level policy hooks.
    hooks: H,
    /// Decoded value waiting for target output capacity.
    pending: PendingValueSlot<D::Value>,
}

impl<D, E, H> TranscodeConvertEngine<D, E, H>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    H: TranscodeConvertHooks<D, E>,
{
    /// Creates a buffered converter engine.
    ///
    /// The supplied conversion hooks create the internal decode and encode hook
    /// instances. This keeps codec-specific hook initialization with the
    /// conversion policy instead of requiring those hook types to implement
    /// [`Default`].
    ///
    /// # Parameters
    ///
    /// - `decoder`: Low-level codec used for source decoding.
    /// - `encoder`: Low-level codec used for target encoding.
    /// - `hooks`: Conversion-level policy hooks.
    ///
    /// # Returns
    ///
    /// Returns a buffered converter engine.
    #[must_use]
    #[inline]
    pub fn new(decoder: D, encoder: E, hooks: H) -> Self {
        let decode_hooks = hooks.create_decode_hooks(&decoder, &encoder);
        let encode_hooks = hooks.create_encode_hooks(&decoder, &encoder);
        Self::from_parts(decoder, encoder, hooks, decode_hooks, encode_hooks)
    }

    /// Builds the engine from already-created component hooks.
    ///
    /// # Type Parameters
    ///
    /// - `D`: Source-side decoder codec.
    /// - `E`: Target-side encoder codec.
    /// - `H`: Conversion-level policy hooks.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Low-level decode codec.
    /// - `encoder`: Low-level encode codec.
    /// - `hooks`: Conversion-level hook aggregator.
    /// - `decode_hooks`: Decode hooks instance created from `hooks`.
    /// - `encode_hooks`: Encode hooks instance created from `hooks`.
    ///
    /// # Returns
    ///
    /// Returns an engine assembled from the provided codecs and hooks.
    #[inline(always)]
    const fn from_parts(
        decoder: D,
        encoder: E,
        hooks: H,
        decode_hooks: H::DecodeHooks,
        encode_hooks: H::EncodeHooks,
    ) -> Self {
        Self {
            decode_engine: TranscodeDecodeEngine::new(decoder, decode_hooks),
            encode_engine: TranscodeEncodeEngine::new(encoder, encode_hooks),
            hooks,
            pending: PendingValueSlot::empty(),
        }
    }

    /// Returns the source-side decode codec.
    ///
    /// # Returns
    ///
    /// Returns the reference to the internal decode codec.
    #[must_use]
    #[inline(always)]
    fn decode_codec(&self) -> &D {
        &self.decode_engine.codec
    }

    /// Returns the target-side encode codec.
    ///
    /// # Returns
    ///
    /// Returns the reference to the internal encode codec.
    #[must_use]
    #[inline(always)]
    fn encode_codec(&self) -> &E {
        &self.encode_engine.codec
    }

    /// Returns an upper bound for target units produced from `input_len` units.
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        let reset_units = self.encode_engine.pending_encode_reset_units();
        let pending_units = self.pending_output_len()?;
        let decoded_values = self.decode_engine.max_output_len(input_len)?;
        let converted_units = self.encode_engine.max_values_output_len(decoded_values)?;
        reset_units
            .checked_add(converted_units)
            .and_then(|value| value.checked_add(pending_units))
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    /// Returns the maximum target units emitted by finishing retained state.
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        let reset_units = self.encode_engine.pending_encode_reset_units();
        let pending_units = self.pending_output_len()?;
        let decoder_finish_values = self.decode_engine.max_finish_output_len();
        let decoder_finish_units = self
            .encode_engine
            .max_values_output_len(decoder_finish_values)?;
        let encoder_finish_units = self.encode_engine.max_hook_finish_output_len();
        let reset_and_pending = reset_units
            .checked_add(pending_units)
            .ok_or(CapacityError::OutputLengthOverflow)?;
        let pending_and_decoder = reset_and_pending
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
    ) -> ConvertProgressResult<D, E, H> {
        if input_index > input.len() {
            return Err(self.hooks.invalid_input_index(
                self.decode_codec(),
                input_index,
                input.len(),
            ));
        }
        if output_index > output.len() {
            let error = self
                .encode_engine
                .invalid_output_index(output_index, output.len());
            return Err(self.hooks.map_encode_error(error));
        }
        assert_unit_bounds::<D>(self.decode_codec());
        assert_unit_bounds::<E>(self.encode_codec());

        let mut state = ConvertState::new(input, input_index, output, output_index);

        if let Some(progress) = self.drain_encode_reset(&mut state)? {
            return Ok(progress);
        }

        // A retained decoded value must be written before consuming more input,
        // otherwise callers could observe output reordered across buffer turns.
        if let Some(progress) = self.drain_pending(&mut state)? {
            return Ok(progress);
        }

        while state.has_input() {
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
    /// Returns [`FinishError`] when capacity planning overflows, when the
    /// caller provides invalid or insufficient output capacity, or when
    /// hook finalization fails.
    pub fn finish(
        &mut self,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<usize, FinishError<ConvertErrorOf<D, E, H>>>
    where
        D::Value: Default,
    {
        assert_unit_bounds::<D>(self.decode_codec());
        assert_unit_bounds::<E>(self.encode_codec());
        let required = self
            .max_finish_output_len()
            .map_err(FinishError::capacity)?;
        FinishError::ensure_output_capacity(output.len(), output_index, required)?;

        let empty_input: &[D::Unit] = &[];
        let mut state = ConvertState::new(empty_input, 0, output, output_index);
        if self
            .drain_encode_reset(&mut state)
            .map_err(FinishError::source)?
            .is_some()
        {
            unreachable!("converter finish bound must reserve encode reset output");
        }
        // Finish keeps the same priority as transcode: output any retained
        // decoded value before asking source-side hooks for final values.
        if self
            .drain_pending(&mut state)
            .map_err(FinishError::source)?
            .is_some()
        {
            unreachable!("converter finish bound must reserve space for pending values");
        }

        // Source-side finish may emit one or more final values. Drain them into
        // the target encoder before finishing target-side hook state.
        self.drain_decoder_finish(&mut state)?;

        let output_cursor = state.output_cursor();
        let written = self
            .encode_engine
            .finish(state.output_mut(), output_cursor)
            .map_err(|error| error.map_source(|error| self.hooks.map_encode_error(error)))?;
        state.advance_output(written);
        Ok(state.written())
    }

    /// Resets hook-owned and component-owned state.
    ///
    /// # Parameters
    ///
    /// - `self`: Converter instance whose retained state is cleared.
    ///
    /// # Returns
    ///
    /// Returns unit `()`.
    #[inline(always)]
    pub fn reset(&mut self) {
        self.pending.clear();
        self.decode_engine.reset();
        self.encode_engine.reset();
        self.hooks.reset();
    }

    /// Converts one value from the current state cursors.
    #[inline]
    fn convert_next(
        &mut self,
        state: &mut ConvertState<'_, D::Unit, E::Unit>,
    ) -> ConvertStepResult<D, E, H> {
        let step = self
            .decode_engine
            .decode_step(state.input(), state.decode_context())
            .map_err(|error| self.hooks.map_decode_error(error))?;
        step.apply_to_convert_state(state, |pending, state| self.encode_pending(pending, state))
    }

    /// Returns the output bound for the retained pending value.
    #[inline(always)]
    fn pending_output_len(&self) -> Result<usize, CapacityError> {
        self.pending.max_output_len(&self.encode_engine)
    }

    /// Writes a retained decoded value before new input is consumed.
    #[inline]
    fn drain_pending(
        &mut self,
        state: &mut ConvertState<'_, D::Unit, E::Unit>,
    ) -> ConvertStepResult<D, E, H> {
        let Some(pending) = self.pending.take() else {
            return Ok(None);
        };
        self.encode_pending(pending, state)
    }

    /// Drains target-side encode reset output before value output.
    #[inline]
    fn drain_encode_reset(
        &mut self,
        state: &mut ConvertState<'_, D::Unit, E::Unit>,
    ) -> ConvertStepResult<D, E, H> {
        let output_index = state.output_cursor();
        let step = self
            .encode_engine
            .encode_reset_step(state.output_mut(), output_index)
            .map_err(|error| self.hooks.map_encode_error(error))?;
        match step {
            None => Ok(None),
            Some(EncodeStep::Written { written }) => {
                state.advance_output(written);
                Ok(None)
            }
            Some(EncodeStep::NeedOutput {
                additional,
                available,
            }) => Ok(Some(state.need_output_progress(additional, available))),
        }
    }

    /// Drains source-side decode finish output and encodes emitted final
    /// values.
    fn drain_decoder_finish(
        &mut self,
        state: &mut ConvertState<'_, D::Unit, E::Unit>,
    ) -> Result<(), FinishError<ConvertErrorOf<D, E, H>>>
    where
        D::Value: Default,
    {
        let value_count = self.decode_engine.max_finish_output_len();
        let mut decoded: Vec<D::Value> = (0..value_count).map(|_| D::Value::default()).collect();
        let written = self
            .decode_engine
            .finish(&mut decoded, 0)
            .map_err(|error| error.map_source(|error| self.hooks.map_decode_error(error)))?;
        for value in decoded.into_iter().take(written) {
            let pending = PendingValue::new(value, 0);
            if self
                .encode_pending(pending, state)
                .map_err(FinishError::source)?
                .is_some()
            {
                unreachable!("converter finish bound must reserve space for decode finish values");
            }
        }
        Ok(())
    }

    /// Encodes one pending value and applies output/pending state changes.
    #[inline]
    fn encode_pending(
        &mut self,
        pending: PendingValue<D::Value>,
        state: &mut ConvertState<'_, D::Unit, E::Unit>,
    ) -> ConvertStepResult<D, E, H> {
        let input_index = pending.input_index();
        let output_index = state.output_cursor();
        let context = EncodeContext {
            input_value: pending.value(),
            input_index,
            output: state.output_mut(),
            output_index,
        };
        let step = self
            .encode_engine
            .encode_step(context)
            .map_err(|error| self.hooks.map_encode_error(error))?;
        let step = match step {
            EncodeStep::Written { written } => PendingEncodeStep::written(written),
            EncodeStep::NeedOutput {
                additional,
                available,
            } => PendingEncodeStep::need_output(pending, additional, available),
        };
        Ok(self.pending.apply_pending_encode_step(step, state))
    }
}

impl<D, E, H> Default for TranscodeConvertEngine<D, E, H>
where
    D: Codec + Default,
    E: Codec<Value = D::Value> + Default,
    H: TranscodeConvertHooks<D, E> + Default,
{
    /// Creates a default buffered converter engine.
    ///
    /// # Returns
    ///
    /// Returns a converter engine constructed from default codecs and hooks.
    #[inline(always)]
    fn default() -> Self {
        Self::new(D::default(), E::default(), H::default())
    }
}
