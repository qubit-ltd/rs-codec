/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Reusable buffered converter engine.

use core::num::NonZeroUsize;

use super::{
    buffered_convert_hooks::BufferedConvertHooks,
    buffered_decode_engine::BufferedDecodeEngine,
    buffered_encode_engine::BufferedEncodeEngine,
    convert_error_of::ConvertProgressResult,
    convert_state::ConvertState,
    convert_step_result::ConvertStepResult,
    decode_finish_step::DecodeFinishStep,
    encode_step::EncodeStep,
    pending_encode_step::PendingEncodeStep,
    pending_value::PendingValue,
    pending_value_slot::PendingValueSlot,
    transcode_progress::TranscodeProgress,
    transcode_status::TranscodeStatus,
};
use crate::{
    CapacityError,
    Codec,
    codec::debug_assert_unit_bounds,
};

/// Reusable buffered conversion engine.
///
/// The engine owns reusable buffered decode and encode engines plus a small
/// conversion-level hook object. It keeps common converter control flow private:
/// index validation, pending-value retention, pending flush, decode-error
/// policy dispatch, encode planning, output-capacity checks, and progress
/// reporting.
///
/// `BufferedConvertEngine` is intentionally batch-oriented. Its public
/// `transcode` method drives a source/output buffer loop and reuses the same
/// unchecked codec and hook primitives as [`crate::BufferedDecodeEngine`] and
/// [`crate::BufferedEncodeEngine`]. It does not call one-value public
/// transcoders in the hot path.
///
/// # Type Parameters
///
/// - `D`: Source-side decoder codec.
/// - `E`: Target-side encoder codec.
/// - `H`: Conversion-level policy hooks.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BufferedConvertEngine<D, E, H>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    H: BufferedConvertHooks<D, E>,
{
    /// Source-side buffered decoder engine.
    decode_engine: BufferedDecodeEngine<D, H::DecodeHooks>,
    /// Target-side buffered encoder engine.
    encode_engine: BufferedEncodeEngine<E, H::EncodeHooks>,
    /// Conversion-level policy hooks.
    hooks: H,
    /// Decoded value waiting for target output capacity.
    pending: PendingValueSlot<D::Value>,
}

impl<D, E, H> BufferedConvertEngine<D, E, H>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    H: BufferedConvertHooks<D, E>,
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
            decode_engine: BufferedDecodeEngine::new(decoder, decode_hooks),
            encode_engine: BufferedEncodeEngine::new(encoder, encode_hooks),
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
        let pending_units = self.pending_output_len()?;
        let decoded_values = self.decode_engine.max_output_len(input_len)?;
        let converted_units = self.encode_engine.max_output_len(decoded_values)?;
        pending_units
            .checked_add(converted_units)
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    /// Returns the maximum target units emitted by finishing retained state.
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        let pending_units = self.pending_output_len()?;
        let decoder_finish_values = self.decode_engine.max_finish_output_len();
        let decoder_finish_units = self.encode_engine.max_output_len(decoder_finish_values)?;
        let encoder_finish_units = self.encode_engine.max_finish_output_len();
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
    /// Returns hook errors when indices are invalid or concrete conversion fails.
    pub fn transcode(
        &mut self,
        input: &[D::Unit],
        input_index: usize,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> ConvertProgressResult<D, E, H> {
        if input_index > input.len() {
            return Err(self
                .hooks
                .invalid_input_index(self.decode_codec(), input_index, input.len()));
        }
        debug_assert_unit_bounds::<D>(self.decode_codec());
        debug_assert_unit_bounds::<E>(self.encode_codec());

        let mut state = ConvertState::new(input, input_index, output, output_index);
        if !state.output_cursor_in_bounds() {
            let additional = self
                .hooks
                .invalid_output_additional(self.decode_codec(), self.encode_codec());
            return Ok(state.need_output_progress(additional, 0));
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
                "BufferedConvertEngine conversion step must consume input or stop",
            );
        }

        Ok(state.complete_progress())
    }

    /// Finishes retained output after EOF.
    ///
    /// Finalization drains a pending decoded value first, then lets the
    /// source-side decode hooks emit final values, encodes those values through
    /// the target-side encode hooks, and finally finishes target-side encode
    /// hook state. The one-value decode scratch used for this cold path requires
    /// `D::Value: Default`; the normal `transcode` loop does not.
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output unit slice visible to the converter.
    /// - `output_index`: Absolute output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns finalization progress.
    ///
    /// # Errors
    ///
    /// Returns hook errors when finalization fails.
    pub fn finish(&mut self, output: &mut [E::Unit], output_index: usize) -> ConvertProgressResult<D, E, H>
    where
        D::Value: Default,
    {
        debug_assert_unit_bounds::<D>(self.decode_codec());
        debug_assert_unit_bounds::<E>(self.encode_codec());
        if output_index > output.len() {
            return Ok(TranscodeProgress::need_output(output_index, NonZeroUsize::MIN, 0, 0, 0));
        }

        let empty_input: &[D::Unit] = &[];
        let mut state = ConvertState::new(empty_input, 0, output, output_index);
        // Finish keeps the same priority as transcode: output any retained
        // decoded value before asking source-side hooks for final values.
        if let Some(progress) = self.drain_pending(&mut state)? {
            return Ok(progress);
        }

        // Source-side finish may emit one or more final values. Drain them into
        // the target encoder before finishing target-side hook state.
        if let Some(progress) = self.drain_decoder_finish(&mut state)? {
            return Ok(progress);
        }

        let output_cursor = state.output_cursor();
        let finish = self
            .encode_engine
            .finish(state.output_mut(), output_cursor)
            .map_err(|error| self.hooks.map_encode_error(error))?;
        state.advance_output(finish.written());
        match finish.status() {
            TranscodeStatus::Complete => Ok(state.complete_progress()),
            TranscodeStatus::NeedOutput {
                output_index,
                additional,
                available,
            } => Ok(TranscodeProgress::need_output(
                output_index,
                additional,
                available,
                0,
                state.written(),
            )),
            TranscodeStatus::NeedInput { .. } => {
                unreachable!("buffered encode engine cannot request source input")
            }
        }
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
    pub fn reset(&mut self) {
        self.pending.clear();
        self.decode_engine.reset();
        self.encode_engine.reset();
        self.hooks.reset();
    }

    /// Returns the output bound for the retained pending value.
    #[inline(always)]
    fn pending_output_len(&self) -> Result<usize, CapacityError> {
        self.pending.max_output_len(&self.encode_engine)
    }

    /// Writes a retained decoded value before new input is consumed.
    #[inline(always)]
    fn drain_pending(&mut self, state: &mut ConvertState<'_, D::Unit, E::Unit>) -> ConvertStepResult<D, E, H> {
        let Some(pending) = self.pending.take() else {
            return Ok(None);
        };
        self.encode_pending(pending, state)
    }

    /// Converts one value from the current state cursors.
    #[inline(always)]
    fn convert_next(&mut self, state: &mut ConvertState<'_, D::Unit, E::Unit>) -> ConvertStepResult<D, E, H> {
        let attempt = self
            .decode_engine
            .decode_step(state.input(), state.decode_context())
            .map_err(|error| self.hooks.map_decode_error(error))?;
        attempt.apply_to_convert_state(state, |pending, state| self.encode_pending(pending, state))
    }

    /// Drains source-side decode finish output and encodes emitted final values.
    #[inline]
    fn drain_decoder_finish(&mut self, state: &mut ConvertState<'_, D::Unit, E::Unit>) -> ConvertStepResult<D, E, H>
    where
        D::Value: Default,
    {
        loop {
            // Source finish is drained one value at a time through the shared
            // decode engine, then each value is encoded by the target engine.
            let mut decoded: [D::Value; 1] = core::array::from_fn(|_| D::Value::default());
            let finish = self
                .decode_engine
                .finish(&mut decoded, 0)
                .map_err(|error| self.hooks.map_decode_error(error))?;
            let step = DecodeFinishStep::from_progress(finish, decoded);

            match step {
                DecodeFinishStep::Complete => return Ok(None),
                DecodeFinishStep::Emit { pending, after_emit } => {
                    // If target output is full, encode_pending stores the value
                    // back into `self.pending` and returns NeedOutput progress.
                    if let Some(progress) = self.encode_pending(pending, state)? {
                        return Ok(Some(progress));
                    }
                    if after_emit.is_complete() {
                        return Ok(None);
                    }
                }
                #[cfg(not(debug_assertions))]
                DecodeFinishStep::NeedOutputWithoutValue => {
                    let additional = self.encode_codec().max_units_per_value();
                    return Ok(Some(state.need_output_progress(additional, state.available_output())));
                }
            }
        }
    }

    /// Encodes one pending value and applies output/pending state changes.
    #[inline(always)]
    fn encode_pending(
        &mut self,
        pending: PendingValue<D::Value>,
        state: &mut ConvertState<'_, D::Unit, E::Unit>,
    ) -> ConvertStepResult<D, E, H> {
        let input_index = pending.input_index();
        let output_index = state.output_cursor();
        let step = self
            .encode_engine
            .encode_value_step(pending.value(), input_index, state.output_mut(), output_index)
            .map_err(|error| self.hooks.map_encode_error(error))?;
        let step = match step {
            EncodeStep::Written { written } => PendingEncodeStep::written(written),
            EncodeStep::NeedOutput { additional, available } => {
                PendingEncodeStep::need_output(pending, additional, available)
            }
        };
        Ok(self.pending.apply_pending_encode_step(step, state))
    }
}

impl<D, E, H> Default for BufferedConvertEngine<D, E, H>
where
    D: Codec + Default,
    E: Codec<Value = D::Value> + Default,
    H: BufferedConvertHooks<D, E> + Default,
{
    /// Creates a default buffered converter engine.
    ///
    /// # Returns
    ///
    /// Returns a converter engine constructed from default codecs and hooks.
    fn default() -> Self {
        Self::new(D::default(), E::default(), H::default())
    }
}
