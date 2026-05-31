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
    convert_decode_attempt_result::ConvertDecodeAttemptResult,
    convert_encode_result::ConvertEncodeResult,
    convert_state::ConvertState,
    convert_step_result::ConvertStepResult,
    decode_action::DecodeAction,
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
    pending: Option<PendingValue<Value>>,
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
    #[inline(always)]
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
            pending: None,
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
            let previous_written = state.written();
            if let Some(progress) = self.convert_next(&mut state)? {
                return Ok(progress);
            }
            debug_assert!(
                state.read() > previous_read || state.written() > previous_written,
                "BufferedConvertEngine conversion step must make progress or stop",
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
        let finish = self
            .encode_engine
            .finish::<Value, Output>(state.output_mut(), output_cursor)
            .map_err(|error| self.hooks.map_encode_error::<Output>(error))?;
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
        self.pending = None;
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
        if self.pending.is_some() {
            self.encode_engine.max_output_len::<Value, Output>(1)
        } else {
            Ok(0)
        }
    }

    /// Writes a retained decoded value before new input is consumed.
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
        match self.try_encode_value(pending.value, pending.input_index, state)? {
            EncodeAttempt::Written { written } => {
                state.advance_output(written);
                Ok(None)
            }
            EncodeAttempt::NeedOutput {
                pending,
                additional,
                available,
            } => {
                self.pending = Some(pending);
                Ok(Some(state.need_output_progress(additional, available)))
            }
        }
    }

    /// Converts one value from the current state cursors.
    fn convert_next<Output>(
        &mut self,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> ConvertStepResult<D, E, H, Input, Value, Output>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
        match self.decode_current(state)? {
            DecodeAttempt::Decoded {
                value,
                consumed,
                input_index,
            } => {
                state.advance_input(consumed.get());
                match self.try_encode_value(value, input_index, state)? {
                    EncodeAttempt::Written { written } => {
                        state.advance_output(written);
                        Ok(None)
                    }
                    EncodeAttempt::NeedOutput {
                        pending,
                        additional,
                        available,
                    } => {
                        self.pending = Some(pending);
                        Ok(Some(state.need_output_progress(additional, available)))
                    }
                }
            }
            DecodeAttempt::Skipped { consumed } => {
                state.advance_input(consumed.get());
                Ok(None)
            }
            DecodeAttempt::NeedInput { additional, available } => {
                Ok(Some(state.need_input_progress(additional, available)))
            }
        }
    }

    /// Finishes source-side decode hooks and encodes emitted final values.
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
            let mut decoded: [Value; 1] = core::array::from_fn(|_| Value::default());
            let finish = self
                .decode_engine
                .finish::<Value>(&mut decoded, 0)
                .map_err(|error| self.hooks.map_decode_error::<Output>(error))?;
            debug_assert!(
                finish.written() <= 1,
                "BufferedDecodeEngine finish wrote beyond the converter scratch buffer",
            );

            if finish.written() != 0 {
                let [value] = decoded;
                match self.try_encode_value(value, 0, state)? {
                    EncodeAttempt::Written { written } => {
                        state.advance_output(written);
                    }
                    EncodeAttempt::NeedOutput {
                        pending,
                        additional,
                        available,
                    } => {
                        self.pending = Some(pending);
                        return Ok(Some(state.need_output_progress(additional, available)));
                    }
                }
            }

            match finish.status() {
                TranscodeStatus::Complete => return Ok(None),
                TranscodeStatus::NeedOutput { .. } => {
                    debug_assert!(
                        finish.written() != 0,
                        "decode finish hook must emit progress before requesting more decoded output",
                    );
                    if finish.written() == 0 {
                        let additional = self.encode_engine.codec.max_units_per_value();
                        return Ok(Some(state.need_output_progress(additional, state.available_output())));
                    }
                }
                TranscodeStatus::NeedInput { .. } => {
                    unreachable!("buffered decode engine finish cannot request source input")
                }
            }
        }
    }

    /// Decodes the value at the current input cursor.
    fn decode_current<Output>(
        &mut self,
        state: &ConvertState<'_, Input, Output>,
    ) -> ConvertDecodeAttemptResult<D, E, H, Input, Value, Output>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
        let available = state.available_input();
        let min_units = self.decode_engine.codec.min_units_per_value().get();
        if available < min_units {
            return Ok(DecodeAttempt::NeedInput {
                additional: NonZeroUsize::new(min_units - available).expect("missing input is non-zero"),
                available,
            });
        }

        let input_index = state.input_cursor();
        let result = {
            // SAFETY: The state has at least `min_units_per_value()` units
            // available from `input_index`.
            unsafe { self.decode_engine.decode_unchecked_at(state.input(), input_index) }
        };
        match result {
            Ok((value, consumed)) => {
                debug_assert!(
                    consumed.get() <= available,
                    "Codec::decode_unchecked consumed beyond available input",
                );
                Ok(DecodeAttempt::Decoded {
                    value,
                    consumed,
                    input_index,
                })
            }
            Err(error) => {
                let context = state.decode_context();
                let action = self
                    .decode_engine
                    .handle_decode_error(error, context)
                    .map_err(|error| self.hooks.map_decode_error::<Output>(error))?;
                self.apply_decode_action(action, input_index, available)
            }
        }
    }

    /// Applies one decode hook action to the converter loop.
    fn apply_decode_action<Output>(
        &self,
        action: DecodeAction<Value>,
        input_index: usize,
        available: usize,
    ) -> ConvertDecodeAttemptResult<D, E, H, Input, Value, Output>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
        match action {
            DecodeAction::NeedInput { required_total } => {
                let additional = required_total.saturating_sub(available).max(1);
                Ok(DecodeAttempt::NeedInput {
                    additional: NonZeroUsize::new(additional).expect("missing input is non-zero"),
                    available,
                })
            }
            DecodeAction::Skip { consumed } => Ok(DecodeAttempt::Skipped {
                consumed: Self::normalize_consumed(consumed, available),
            }),
            DecodeAction::Emit { value, consumed } => Ok(DecodeAttempt::Decoded {
                value,
                consumed: Self::normalize_consumed(consumed, available),
                input_index,
            }),
        }
    }

    /// Encodes one decoded value at the current output cursor.
    fn try_encode_value<Output>(
        &mut self,
        value: Value,
        input_index: usize,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> ConvertEncodeResult<D, E, H, Input, Value, Output>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
        let output_index = state.output_cursor();
        let available = state.available_output();
        let plan = self
            .encode_engine
            .prepare_value::<Value, Output>(&value, input_index)
            .map_err(|error| self.hooks.map_encode_error::<Output>(error))?;
        let required = plan.max_output_units;
        if available < required {
            let additional = required - available;
            return Ok(EncodeAttempt::NeedOutput {
                pending: PendingValue { value, input_index },
                additional: NonZeroUsize::new(additional).expect("missing output is non-zero"),
                available,
            });
        }

        let written = {
            let output = state.output_mut();
            // SAFETY: The capacity check above proves the prepared output bound.
            unsafe {
                self.encode_engine
                    .write_prepared_value(&value, input_index, plan, output, output_index)
            }
            .map_err(|error| self.hooks.map_encode_error::<Output>(error))?
        };
        debug_assert!(
            written <= required,
            "BufferedConvertEngine encode hook wrote beyond its prepared capacity bound",
        );
        Ok(EncodeAttempt::Written { written })
    }

    /// Normalizes a hook-reported consumption count.
    #[inline(always)]
    fn normalize_consumed(consumed: usize, available: usize) -> NonZeroUsize {
        debug_assert!(available > 0, "decode action cannot consume empty input");
        let consumed = consumed.min(available).max(1);
        // SAFETY: The normalized count is clamped to at least one.
        unsafe { NonZeroUsize::new_unchecked(consumed) }
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
    #[inline(always)]
    fn default() -> Self {
        Self::new(D::default(), E::default(), H::default())
    }
}
