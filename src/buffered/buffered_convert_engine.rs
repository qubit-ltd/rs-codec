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
    convert_error_of::ConvertProgressResult,
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

/// Internal helper types bound to one buffered converter engine.
trait BufferedConvertEngineTypes<'a> {
    /// Source-side reader type.
    type SourceValueReader;
    /// Source-side finish reader type.
    type SourceFinishReader;
    /// Target-side writer type.
    type TargetValueWriter;
}

/// Source-side reader type for one buffered converter engine.
type SourceValueReaderOf<'a, Engine> = <Engine as BufferedConvertEngineTypes<'a>>::SourceValueReader;

/// Source-side finish reader type for one buffered converter engine.
type SourceFinishReaderOf<'a, Engine> = <Engine as BufferedConvertEngineTypes<'a>>::SourceFinishReader;

/// Target-side writer type for one buffered converter engine.
type TargetValueWriterOf<'a, Engine> = <Engine as BufferedConvertEngineTypes<'a>>::TargetValueWriter;

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
/// - `Output`: Target unit type produced by `E`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BufferedConvertEngine<D, E, H, Input, Value, Output>
where
    D: Codec<Value, Input>,
    E: Codec<Value, Output>,
    H: BufferedConvertHooks<D, E, Input, Value, Output>,
    Input: Copy,
    Output: Copy,
{
    /// Source-side buffered decoder engine.
    decode_engine: BufferedDecodeEngine<D, H::DecodeHooks, Input, Value>,
    /// Target-side buffered encoder engine.
    encode_engine: BufferedEncodeEngine<E, H::EncodeHooks, Value, Output>,
    /// Conversion-level policy hooks.
    hooks: H,
    /// Decoded value waiting for target output capacity.
    pending: PendingValueSlot<Value>,
    /// Binds the engine to one decoded logical value and target unit type.
    marker: PhantomData<fn(Value, Output)>,
}

impl<'a, D, E, H, Input, Value, Output> BufferedConvertEngineTypes<'a>
    for BufferedConvertEngine<D, E, H, Input, Value, Output>
where
    D: Codec<Value, Input> + 'a,
    E: Codec<Value, Output> + 'a,
    H: BufferedConvertHooks<D, E, Input, Value, Output> + 'a,
    H::DecodeHooks: 'a,
    H::EncodeHooks: 'a,
    Input: Copy + 'a,
    Value: 'a,
    Output: Copy + 'a,
{
    type SourceFinishReader = SourceFinishReader<'a, D, E, H, Input, Value, Output>;
    type SourceValueReader = SourceValueReader<'a, D, E, H, Input, Value, Output>;
    type TargetValueWriter = TargetValueWriter<'a, D, E, H, Input, Value, Output>;
}

impl<D, E, H, Input, Value, Output> BufferedConvertEngine<D, E, H, Input, Value, Output>
where
    D: Codec<Value, Input>,
    E: Codec<Value, Output>,
    H: BufferedConvertHooks<D, E, Input, Value, Output>,
    Input: Copy,
    Output: Copy,
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

    /// Returns the source-side decode codec.
    #[must_use]
    #[inline(always)]
    fn decode_codec(&self) -> &D {
        &self.decode_engine.codec
    }

    /// Returns the target-side encode codec.
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
        input: &[Input],
        input_index: usize,
        output: &mut [Output],
        output_index: usize,
    ) -> ConvertProgressResult<D, E, H, Input, Value, Output> {
        if input_index > input.len() {
            return Err(self
                .hooks
                .invalid_input_index(self.decode_codec(), input_index, input.len()));
        }
        debug_assert_unit_bounds::<D, Value, Input>(self.decode_codec());
        debug_assert_unit_bounds::<E, Value, Output>(self.encode_codec());

        let mut state = ConvertState::new(input, input_index, output, output_index);
        if !state.output_cursor_in_bounds() {
            let additional = self
                .hooks
                .invalid_output_additional(self.decode_codec(), self.encode_codec());
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
    pub fn finish(
        &mut self,
        output: &mut [Output],
        output_index: usize,
    ) -> ConvertProgressResult<D, E, H, Input, Value, Output>
    where
        Value: Default,
    {
        debug_assert_unit_bounds::<D, Value, Input>(self.decode_codec());
        debug_assert_unit_bounds::<E, Value, Output>(self.encode_codec());
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
            let mut target = self.target_value_writer();
            target.finish(state.output_mut(), output_cursor)?
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

    /// Creates a source-side value reader bound to this engine.
    #[inline(always)]
    fn source_value_reader<'a>(&'a mut self) -> SourceValueReaderOf<'a, Self> {
        SourceValueReader::new(&mut self.decode_engine, &self.hooks)
    }

    /// Creates a source-side finish reader bound to this engine.
    #[inline(always)]
    fn source_finish_reader<'a>(&'a mut self) -> SourceFinishReaderOf<'a, Self> {
        SourceFinishReader::new(&mut self.decode_engine, &self.hooks)
    }

    /// Creates a target-side writer bound to this engine.
    #[inline(always)]
    fn target_value_writer<'a>(&'a mut self) -> TargetValueWriterOf<'a, Self> {
        TargetValueWriter::new(&mut self.encode_engine, &self.hooks)
    }

    /// Writes a retained decoded value before new input is consumed.
    #[inline(always)]
    fn drain_pending(
        &mut self,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> ConvertStepResult<D, E, H, Input, Value, Output> {
        let Some(pending) = self.pending.take() else {
            return Ok(None);
        };
        self.encode_pending(pending, state)
    }

    /// Converts one value from the current state cursors.
    #[inline(always)]
    fn convert_next(
        &mut self,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> ConvertStepResult<D, E, H, Input, Value, Output> {
        let attempt = {
            let mut source = self.source_value_reader();
            source.read_next(state)?
        };
        attempt.apply_to_convert_state(state, |pending, state| self.encode_pending(pending, state))
    }

    /// Finishes source-side decode hooks and encodes emitted final values.
    #[inline]
    fn finish_decoder(
        &mut self,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> ConvertStepResult<D, E, H, Input, Value, Output>
    where
        Value: Default,
    {
        loop {
            let step = {
                let mut source = self.source_finish_reader();
                source.read_next()?
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
        pending: PendingValue<Value>,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> ConvertStepResult<D, E, H, Input, Value, Output> {
        let step = {
            let mut target = self.target_value_writer();
            target.write_pending(pending, state)?
        };
        Ok(self.pending.apply_pending_encode_step(step, state))
    }
}

impl<D, E, H, Input, Value, Output> Default for BufferedConvertEngine<D, E, H, Input, Value, Output>
where
    D: Codec<Value, Input> + Default,
    E: Codec<Value, Output> + Default,
    H: BufferedConvertHooks<D, E, Input, Value, Output> + Default,
    Input: Copy,
    Output: Copy,
{
    /// Creates a default buffered converter engine.
    fn default() -> Self {
        Self::new(D::default(), E::default(), H::default())
    }
}
