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

use core::marker::PhantomData;

use super::{
    buffered_convert_hooks::BufferedConvertHooks,
    buffered_decode_engine::BufferedDecodeEngine,
    buffered_encode_engine::BufferedEncodeEngine,
    buffered_encode_hooks::BufferedEncodeHooks,
    convert_decode_attempt_result::ConvertDecodeAttemptResult,
    convert_encode_result::ConvertEncodeResult,
    convert_state::ConvertState,
    convert_step_result::ConvertStepResult,
    decode_attempt::DecodeAttempt,
    encode_attempt::EncodeAttempt,
    pending_value::PendingValue,
    transcode_progress::TranscodeProgress,
    transcode_status::TranscodeStatus,
};
use crate::{
    CapacityError,
    Codec,
    ConvertErrorFactory,
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

/// Slot that owns the converter's retained decoded value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct PendingValueSlot<Value> {
    /// Retained decoded value waiting for output capacity.
    value: Option<PendingValue<Value>>,
}

impl<Value> PendingValueSlot<Value> {
    /// Creates an empty pending-value slot.
    #[must_use]
    #[inline(always)]
    const fn empty() -> Self {
        Self { value: None }
    }

    /// Returns the target-output bound for the retained value.
    #[must_use = "capacity planning can fail on overflow"]
    #[inline(always)]
    fn max_output_len<E, H, Output>(&self, engine: &BufferedEncodeEngine<E, H>) -> Result<usize, CapacityError>
    where
        E: Codec<Value, Output>,
        H: BufferedEncodeHooks<E, Value, Output>,
        Output: Copy,
    {
        if self.value.is_some() {
            engine.max_output_len::<Value, Output>(1)
        } else {
            Ok(0)
        }
    }

    /// Removes any retained decoded value.
    #[inline(always)]
    fn clear(&mut self) {
        self.value = None;
    }

    /// Takes the retained decoded value, if any.
    #[inline(always)]
    fn take(&mut self) -> Option<PendingValue<Value>> {
        self.value.take()
    }

    /// Applies an encode attempt to this slot and the current conversion state.
    #[inline(always)]
    fn apply_encode_attempt<Input, Output>(
        &mut self,
        attempt: EncodeAttempt<Value>,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> Option<TranscodeProgress> {
        match attempt {
            EncodeAttempt::Written { written } => {
                state.advance_output(written);
                None
            }
            EncodeAttempt::NeedOutput {
                pending,
                additional,
                available,
            } => {
                self.value = Some(pending);
                Some(state.need_output_progress(additional, available))
            }
        }
    }
}

/// Result type for source-side finish steps.
type ConvertDecodeFinishResult<D, E, H, Input, Value, Output> =
    Result<DecodeFinishStep<Value>, <H as BufferedConvertHooks<D, E, Input, Value>>::Error<Output>>;

/// Source-side reader object used by the converter coordinator.
struct SourceValueReader<'a, D, E, H, Input, Value>
where
    D: Codec<Value, Input>,
    H: BufferedConvertHooks<D, E, Input, Value>,
    Input: Copy,
{
    /// Source-side buffered decoder engine.
    engine: &'a mut BufferedDecodeEngine<D, H::DecodeHooks, Input>,
    /// Conversion hooks used for error mapping.
    hooks: &'a H,
    /// Binds this helper to the target codec and value types.
    marker: PhantomData<fn(E, Value)>,
}

impl<'a, D, E, H, Input, Value> SourceValueReader<'a, D, E, H, Input, Value>
where
    D: Codec<Value, Input>,
    H: BufferedConvertHooks<D, E, Input, Value>,
    Input: Copy,
{
    /// Creates a source-side reader.
    #[inline(always)]
    const fn new(engine: &'a mut BufferedDecodeEngine<D, H::DecodeHooks, Input>, hooks: &'a H) -> Self {
        Self {
            engine,
            hooks,
            marker: PhantomData,
        }
    }

    /// Reads the next source value or source-side stop condition.
    #[inline(always)]
    fn read_next<Output>(
        &mut self,
        state: &ConvertState<'_, Input, Output>,
    ) -> ConvertDecodeAttemptResult<D, E, H, Input, Value, Output>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
        let available = state.available_input();
        let min_units = self.engine.codec.min_units_per_value().get();
        if let Some(attempt) = state.need_input_for_min_units(min_units) {
            return Ok(attempt);
        }

        let input_index = state.input_cursor();
        let result = {
            // SAFETY: The state has at least `min_units_per_value()` units
            // available from `input_index`.
            unsafe { self.engine.decode_unchecked_at(state.input(), input_index) }
        };
        match result {
            Ok((value, consumed)) => {
                debug_assert!(
                    consumed.get() <= available,
                    "Codec::decode_unchecked consumed beyond available input",
                );
                Ok(DecodeAttempt::decoded(value, consumed, input_index))
            }
            Err(error) => {
                let context = state.decode_context();
                let action = match self.engine.handle_decode_error(error, context) {
                    Ok(action) => action,
                    Err(error) => return Err(self.hooks.map_decode_error::<Output>(error)),
                };
                Ok(action.into_attempt(context.input_index, context.available))
            }
        }
    }

    /// Lets source-side finish hooks emit at most one final value.
    #[inline]
    fn finish_one<Output>(
        &mut self,
        decoded: &mut [Value; 1],
    ) -> Result<TranscodeProgress, <H as BufferedConvertHooks<D, E, Input, Value>>::Error<Output>>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
        match self.engine.finish::<Value>(decoded, 0) {
            Ok(finish) => Ok(finish),
            Err(error) => Err(self.hooks.map_decode_error::<Output>(error)),
        }
    }
}

/// Source-side finish reader used by the converter finalization path.
struct SourceFinishReader<'a, D, E, H, Input, Value>
where
    D: Codec<Value, Input>,
    H: BufferedConvertHooks<D, E, Input, Value>,
    Input: Copy,
{
    /// Source-side reader used for finish hook dispatch.
    source: SourceValueReader<'a, D, E, H, Input, Value>,
}

impl<'a, D, E, H, Input, Value> SourceFinishReader<'a, D, E, H, Input, Value>
where
    D: Codec<Value, Input>,
    H: BufferedConvertHooks<D, E, Input, Value>,
    Input: Copy,
{
    /// Creates a source-side finish reader.
    #[inline(always)]
    const fn new(engine: &'a mut BufferedDecodeEngine<D, H::DecodeHooks, Input>, hooks: &'a H) -> Self {
        Self {
            source: SourceValueReader::new(engine, hooks),
        }
    }

    /// Reads the next source-side finish step.
    ///
    /// # Returns
    ///
    /// Returns the decoded final value, completion, or a source finish stop
    /// condition.
    ///
    /// # Errors
    ///
    /// Returns mapped decode errors produced by source-side finish hooks.
    #[inline]
    fn read_next<Output>(&mut self) -> ConvertDecodeFinishResult<D, E, H, Input, Value, Output>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Value: Default,
        Output: Copy,
    {
        let mut decoded: [Value; 1] = core::array::from_fn(|_| Value::default());
        let finish = self.source.finish_one::<Output>(&mut decoded)?;
        Ok(DecodeFinishStep::from_progress(finish, decoded))
    }
}

/// Converter action produced by one source-side finish hook call.
enum DecodeFinishStep<Value> {
    /// Source-side finish hooks are complete.
    Complete,
    /// A final decoded value must be encoded.
    Emit {
        /// Pending final value produced by source-side finish hooks.
        pending: PendingValue<Value>,
        /// Source-side finish state after the value is encoded.
        after_emit: DecodeFinishAfterEmit,
    },
    /// Source-side finish requested more decoded output without emitting.
    #[cfg(not(debug_assertions))]
    NeedOutputWithoutValue,
}

impl<Value> DecodeFinishStep<Value> {
    /// Builds a finish step from source-side finish progress.
    ///
    /// # Parameters
    ///
    /// - `finish`: Progress returned by the source-side finish hook.
    /// - `decoded`: One-value scratch buffer passed to the finish hook.
    ///
    /// # Returns
    ///
    /// Returns the converter-level finish step represented by `finish`.
    #[must_use]
    fn from_progress(finish: TranscodeProgress, decoded: [Value; 1]) -> Self {
        debug_assert!(
            finish.written() <= 1,
            "BufferedDecodeEngine finish wrote beyond the converter scratch buffer",
        );

        if finish.written() != 0 {
            let [value] = decoded;
            return Self::Emit {
                pending: PendingValue::new(value, 0),
                after_emit: DecodeFinishAfterEmit::from_status(finish.status()),
            };
        }

        match finish.status() {
            TranscodeStatus::Complete => Self::Complete,
            TranscodeStatus::NeedOutput { .. } => {
                #[cfg(debug_assertions)]
                {
                    unreachable!("decode finish hook must emit progress before requesting more decoded output")
                }
                #[cfg(not(debug_assertions))]
                {
                    Self::NeedOutputWithoutValue
                }
            }
            TranscodeStatus::NeedInput { .. } => {
                unreachable!("buffered decode engine finish cannot request source input")
            }
        }
    }
}

/// Source-side finish state after an emitted final value is encoded.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum DecodeFinishAfterEmit {
    /// Source-side finish hooks are complete after this value.
    Complete,
    /// Source-side finish hooks may emit more values.
    Continue,
}

impl DecodeFinishAfterEmit {
    /// Converts source-side finish status into post-emit control flow.
    ///
    /// # Parameters
    ///
    /// - `status`: Status returned by source-side finish hooks.
    ///
    /// # Returns
    ///
    /// Returns whether finalization is complete after the emitted value.
    #[must_use]
    fn from_status(status: TranscodeStatus) -> Self {
        match status {
            TranscodeStatus::Complete => Self::Complete,
            TranscodeStatus::NeedOutput { .. } => Self::Continue,
            TranscodeStatus::NeedInput { .. } => {
                unreachable!("buffered decode engine finish cannot request source input")
            }
        }
    }

    /// Returns whether no more source-side finish values are expected.
    #[must_use]
    #[inline(always)]
    const fn is_complete(self) -> bool {
        matches!(self, Self::Complete)
    }
}

/// Target-side writer object used by the converter coordinator.
struct TargetValueWriter<'a, D, E, H, Input, Value>
where
    D: Codec<Value, Input>,
    H: BufferedConvertHooks<D, E, Input, Value>,
    Input: Copy,
{
    /// Target-side buffered encoder engine.
    engine: &'a mut BufferedEncodeEngine<E, H::EncodeHooks>,
    /// Conversion hooks used for error mapping.
    hooks: &'a H,
    /// Binds this helper to the source codec and value types.
    marker: PhantomData<fn(D, Input, Value)>,
}

impl<'a, D, E, H, Input, Value> TargetValueWriter<'a, D, E, H, Input, Value>
where
    D: Codec<Value, Input>,
    H: BufferedConvertHooks<D, E, Input, Value>,
    Input: Copy,
{
    /// Creates a target-side writer.
    #[inline(always)]
    const fn new(engine: &'a mut BufferedEncodeEngine<E, H::EncodeHooks>, hooks: &'a H) -> Self {
        Self {
            engine,
            hooks,
            marker: PhantomData,
        }
    }

    /// Encodes one pending source value at the current output cursor.
    #[inline(always)]
    fn write_pending<Output>(
        &mut self,
        pending: PendingValue<Value>,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> ConvertEncodeResult<D, E, H, Input, Value, Output>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
        let input_index = pending.input_index();
        let output_index = state.output_cursor();
        let available = state.available_output();
        let plan = match self.engine.prepare_value::<Value, Output>(pending.value(), input_index) {
            Ok(plan) => plan,
            Err(error) => return Err(self.hooks.map_encode_error::<Output>(error)),
        };
        let required = plan.max_output_units;
        if available < required {
            return Ok(EncodeAttempt::need_output(pending, required, available));
        }

        let written = {
            let output = state.output_mut();
            // SAFETY: The capacity check above proves the prepared output bound.
            match unsafe {
                self.engine
                    .write_prepared_value(pending.value(), input_index, plan, output, output_index)
            } {
                Ok(written) => written,
                Err(error) => return Err(self.hooks.map_encode_error::<Output>(error)),
            }
        };
        debug_assert!(
            written <= required,
            "BufferedConvertEngine encode hook wrote beyond its prepared capacity bound",
        );
        Ok(EncodeAttempt::written(written))
    }

    /// Finishes target-side hook-owned output.
    #[inline]
    fn finish<Output>(
        &mut self,
        output: &mut [Output],
        output_index: usize,
    ) -> Result<TranscodeProgress, <H as BufferedConvertHooks<D, E, Input, Value>>::Error<Output>>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
        match self.engine.finish::<Value, Output>(output, output_index) {
            Ok(finish) => Ok(finish),
            Err(error) => Err(self.hooks.map_encode_error::<Output>(error)),
        }
    }
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
            return Err(<H::Error<Output> as ConvertErrorFactory<D>>::invalid_input_index(
                &self.decode_engine.codec,
                input_index,
                input.len(),
            ));
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
            return Ok(TranscodeProgress::need_output(output_index, 1, 0, 0, 0));
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
        let attempt = {
            let mut target = TargetValueWriter::<D, E, H, Input, Value>::new(&mut self.encode_engine, &self.hooks);
            target.write_pending(pending, state)?
        };
        Ok(self.pending.apply_encode_attempt(attempt, state))
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
