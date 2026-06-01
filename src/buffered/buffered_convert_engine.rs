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

use core::{
    marker::PhantomData,
    num::NonZeroUsize,
};

use super::{
    buffered_convert_hooks::BufferedConvertHooks,
    buffered_decode_engine::BufferedDecodeEngine,
    buffered_encode_engine::BufferedEncodeEngine,
    buffered_encode_hooks::BufferedEncodeHooks,
    convert_state::ConvertState,
    convert_step_result::ConvertStepResult,
    decode_finish_step::DecodeFinishStep,
    pending_value::PendingValue,
    pending_value_slot::PendingValueSlot,
    source_finish_reader::SourceFinishReader,
    source_value_reader::SourceValueReader,
    target_value_writer::TargetValueWriter,
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
/// - `Input`: Source unit type.
/// - `Value`: Logical value decoded by `D` and encoded by `E`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BufferedConvertEngine<D, E, H, Input, Value>
where
    D: Codec<Value, Input>,
    H: BufferedConvertHooks<D, E, Input, Value>,
    Input: Copy,
{
    /// Source-side buffered decoder engine.
    decode_engine: BufferedDecodeEngine<D, H::DecodeHooks, Input>,
    /// Target-side buffered encoder engine.
    encode_engine: BufferedEncodeEngine<E, H::EncodeHooks>,
    /// Conversion-level policy hooks.
    hooks: H,
    /// Decoded value waiting for target output capacity.
    pending: PendingValueSlot<Value>,
    /// Binds the engine to one decoded logical value type.
    marker: PhantomData<fn(Value)>,
}

impl<D, E, H, Input, Value> BufferedConvertEngine<D, E, H, Input, Value>
where
    D: Codec<Value, Input>,
    H: BufferedConvertHooks<D, E, Input, Value>,
    Input: Copy,
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
            marker: PhantomData,
        }
    }

    /// Returns an upper bound for target units produced from `input_len` units.
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_output_len<Output>(&self, input_len: usize) -> Result<usize, CapacityError>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
        let pending_units = self.pending_output_len::<Output>()?;
        let decoded_values = self.decode_engine.max_output_len::<Value>(input_len)?;
        let converted_units = self.encode_engine.max_output_len::<Value, Output>(decoded_values)?;
        pending_units
            .checked_add(converted_units)
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    /// Returns the maximum target units emitted by finishing retained state.
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_finish_output_len<Output>(&self) -> Result<usize, CapacityError>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
        let pending_units = self.pending_output_len::<Output>()?;
        let decoder_finish_values = self.decode_engine.max_finish_output_len::<Value>();
        let decoder_finish_units = self
            .encode_engine
            .max_output_len::<Value, Output>(decoder_finish_values)?;
        let encoder_finish_units = self.encode_engine.max_finish_output_len::<Value, Output>();
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
    pub fn transcode<Output>(
        &mut self,
        input: &[Input],
        input_index: usize,
        output: &mut [Output],
        output_index: usize,
    ) -> Result<TranscodeProgress, <H as BufferedConvertHooks<D, E, Input, Value>>::Error<Output>>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
        if input_index > input.len() {
            return Err(self
                .hooks
                .invalid_input_index::<Output>(&self.decode_engine.codec, input_index, input.len()));
        }
        debug_assert_unit_bounds::<D, Value, Input>(&self.decode_engine.codec);
        debug_assert_unit_bounds::<E, Value, Output>(&self.encode_engine.codec);

        let mut state = ConvertState::new(input, input_index, output, output_index);
        if !state.output_cursor_in_bounds() {
            let additional = self
                .hooks
                .invalid_output_additional::<Output>(&self.decode_engine.codec, &self.encode_engine.codec);
            return Ok(state.need_output_progress(additional, 0));
        }

        if let Some(progress) = self.drain_pending(&mut state)? {
            return Ok(progress);
        }

        while state.has_input() {
            let previous_read = state.read();
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
    /// `Value: Default`; the normal `transcode` loop does not.
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
    pub fn finish<Output>(
        &mut self,
        output: &mut [Output],
        output_index: usize,
    ) -> Result<TranscodeProgress, <H as BufferedConvertHooks<D, E, Input, Value>>::Error<Output>>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Value: Default,
        Output: Copy,
    {
        debug_assert_unit_bounds::<D, Value, Input>(&self.decode_engine.codec);
        debug_assert_unit_bounds::<E, Value, Output>(&self.encode_engine.codec);
        if output_index > output.len() {
            return Ok(TranscodeProgress::need_output(output_index, NonZeroUsize::MIN, 0, 0, 0));
        }

        let empty_input: &[Input] = &[];
        let mut state = ConvertState::new(empty_input, 0, output, output_index);
        if let Some(progress) = self.drain_pending(&mut state)? {
            return Ok(progress);
        }

        if let Some(progress) = self.finish_decoder(&mut state)? {
            return Ok(progress);
        }

        let output_cursor = state.output_cursor();
        let finish = {
            let mut target = TargetValueWriter::<D, E, H, Input, Value>::new(&mut self.encode_engine, &self.hooks);
            target.finish::<Output>(state.output_mut(), output_cursor)?
        };
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
    pub fn reset<Output>(&mut self)
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
        self.pending.clear();
        self.decode_engine.reset::<Value>();
        self.encode_engine.reset::<Value, Output>();
        self.hooks.reset();
    }

    /// Returns the output bound for the retained pending value.
    #[inline(always)]
    fn pending_output_len<Output>(&self) -> Result<usize, CapacityError>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
        self.pending
            .max_output_len::<E, H::EncodeHooks, Output>(&self.encode_engine)
    }

    /// Writes a retained decoded value before new input is consumed.
    #[inline(always)]
    fn drain_pending<Output>(
        &mut self,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> ConvertStepResult<D, E, H, Input, Value, Output>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
        let Some(pending) = self.pending.take() else {
            return Ok(None);
        };
        self.encode_pending(pending, state)
    }

    /// Converts one value from the current state cursors.
    #[inline(always)]
    fn convert_next<Output>(
        &mut self,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> ConvertStepResult<D, E, H, Input, Value, Output>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
        let attempt = {
            let mut source = SourceValueReader::<D, E, H, Input, Value>::new(&mut self.decode_engine, &self.hooks);
            source.read_next::<Output>(state)?
        };
        attempt.apply_to_convert_state(state, |pending, state| self.encode_pending(pending, state))
    }

    /// Finishes source-side decode hooks and encodes emitted final values.
    #[inline]
    fn finish_decoder<Output>(
        &mut self,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> ConvertStepResult<D, E, H, Input, Value, Output>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Value: Default,
        Output: Copy,
    {
        loop {
            let step = {
                let mut source = SourceFinishReader::<D, E, H, Input, Value>::new(&mut self.decode_engine, &self.hooks);
                source.read_next::<Output>()?
            };

            match step {
                DecodeFinishStep::Complete => return Ok(None),
                DecodeFinishStep::Emit { pending, after_emit } => {
                    if let Some(progress) = self.encode_pending(pending, state)? {
                        return Ok(Some(progress));
                    }
                    if after_emit.is_complete() {
                        return Ok(None);
                    }
                }
                #[cfg(not(debug_assertions))]
                DecodeFinishStep::NeedOutputWithoutValue => {
                    let additional = self.encode_engine.codec.max_units_per_value();
                    return Ok(Some(state.need_output_progress(additional, state.available_output())));
                }
            }
        }
    }

    /// Encodes one pending value and applies output/pending state changes.
    #[inline(always)]
    fn encode_pending<Output>(
        &mut self,
        pending: PendingValue<Value>,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> ConvertStepResult<D, E, H, Input, Value, Output>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
        let step = {
            let mut target = TargetValueWriter::<D, E, H, Input, Value>::new(&mut self.encode_engine, &self.hooks);
            target.write_pending(pending, state)?
        };
        Ok(self.pending.apply_pending_encode_step(step, state))
    }
}

impl<D, E, H, Input, Value> Default for BufferedConvertEngine<D, E, H, Input, Value>
where
    D: Codec<Value, Input> + Default,
    E: Default,
    H: BufferedConvertHooks<D, E, Input, Value> + Default,
    Input: Copy,
{
    /// Creates a default buffered converter engine.
    fn default() -> Self {
        Self::new(D::default(), E::default(), H::default())
    }
}
