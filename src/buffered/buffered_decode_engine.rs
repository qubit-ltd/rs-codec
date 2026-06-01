/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Reusable buffered decoder engine.

use core::marker::PhantomData;
use core::num::NonZeroUsize;

use super::{
    buffered_decode_hooks::BufferedDecodeHooks,
    decode_action::DecodeAction,
    decode_attempt::DecodeAttempt,
    decode_context::DecodeContext,
    decode_state::DecodeState,
    transcode_progress::TranscodeProgress,
};
use crate::{
    CapacityError,
    Codec,
    DecodeErrorFactory,
    codec::debug_assert_unit_bounds,
};

/// Reusable buffered decoding engine for codec-backed decoders.
///
/// The engine owns the low-level codec and hook object. It keeps the common
/// buffered decoding loop private: input-index validation, output-capacity
/// checks, calls to [`Codec::decode_unchecked`], hook dispatch, and
/// [`crate::TranscodeStatus`] reporting. Incomplete input tails are left in the
/// caller-provided input slice; callers own input-buffer refill.
///
/// Use this type to build a streaming decoder over a one-value [`Codec`]. The
/// engine decodes into a caller-provided output slice and returns
/// [`TranscodeProgress`] instead of allocating. On success it writes decoded
/// values directly to output. On codec errors it delegates to
/// [`crate::BufferedDecodeHooks`], allowing a policy to request more input,
/// skip invalid units, emit a replacement value, or fail.
///
/// The engine stops before reading an incomplete value when fewer than
/// [`Codec::min_units_per_value`] units are available. For variable-width
/// codecs, the codec may still return an incomplete decode error after that
/// minimum is satisfied; hooks should convert that error into
/// [`crate::DecodeAction::NeedInput`] when the stream may continue.
///
/// For strict decoding that wraps codec errors, use
/// [`crate::CodecBufferedDecoder`]. Use `BufferedDecodeEngine` directly when
/// invalid input should be repaired, skipped, counted, or otherwise handled by
/// policy.
///
/// # Example
///
/// ```rust,ignore
/// use qubit_codec::{BufferedDecodeEngine, TranscodeStatus};
///
/// let mut engine = BufferedDecodeEngine::<_, _, u8>::new(ByteCodec, ReplacementHooks);
/// let input = [b'a', 0xff, b'b'];
/// let mut output = [0_u8; 3];
///
/// let progress = engine.transcode(&input, 0, &mut output, 0)?;
/// match progress.status() {
///     TranscodeStatus::Complete => assert_eq!(&output[..progress.written()], b"a?b"),
///     TranscodeStatus::NeedInput { input_index, .. } => {
///         // Keep `input[input_index..]`, append more source units, and resume.
///     }
///     TranscodeStatus::NeedOutput { output_index, .. } => {
///         // Drain `output[..output_index]`, then resume with more output room.
///     }
/// }
/// # Ok::<(), MyError>(())
/// ```
///
/// # Type Parameters
///
/// - `C`: Low-level codec used by the engine.
/// - `H`: Policy hook object used by the engine.
/// - `Unit`: Encoded input unit type accepted by the engine.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct BufferedDecodeEngine<C, H, Unit> {
    /// Low-level codec used for one-value decoding.
    pub(super) codec: C,
    /// Policy hooks used for decode failures.
    pub(super) hooks: H,
    /// Binds the engine to the encoded input unit type.
    marker: PhantomData<fn(Unit)>,
}

impl<C, H, Unit> BufferedDecodeEngine<C, H, Unit> {
    /// Creates a buffered decoder engine.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec used for one-value decoding.
    /// - `hooks`: Policy hooks used for decode failures.
    ///
    /// # Returns
    ///
    /// Returns a buffered decoder engine.
    #[must_use]
    pub const fn new(codec: C, hooks: H) -> Self {
        Self {
            codec,
            hooks,
            marker: PhantomData,
        }
    }

    /// Returns an upper bound for decoded values produced from `input_len` units.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Number of source units the caller plans to decode.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound, or a capacity error on arithmetic
    /// overflow.
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_output_len<Value>(&self, input_len: usize) -> Result<usize, CapacityError>
    where
        C: Codec<Value, Unit>,
        H: BufferedDecodeHooks<C, Unit, Value>,
        Unit: Copy,
    {
        debug_assert_unit_bounds::<C, Value, Unit>(&self.codec);
        self.hooks.max_output_len(&self.codec, input_len)
    }

    /// Returns the maximum values emitted by finishing hook-owned state.
    ///
    /// # Returns
    ///
    /// Returns the hook-provided final output bound.
    #[must_use]
    pub fn max_finish_output_len<Value>(&self) -> usize
    where
        C: Codec<Value, Unit>,
        H: BufferedDecodeHooks<C, Unit, Value>,
        Unit: Copy,
    {
        self.hooks.max_finish_output_len(&self.codec)
    }

    /// Resets hook-owned state.
    pub fn reset<Value>(&mut self)
    where
        C: Codec<Value, Unit>,
        H: BufferedDecodeHooks<C, Unit, Value>,
        Unit: Copy,
    {
        self.hooks.reset(&self.codec);
    }

    /// Decodes source units into caller-provided output values.
    ///
    /// # Parameters
    ///
    /// - `input`: Complete input unit slice visible to the decoder.
    /// - `input_index`: Absolute input unit index where decoding starts.
    /// - `output`: Complete output value slice visible to the decoder.
    /// - `output_index`: Absolute output value index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns progress describing input units consumed, output values written,
    /// and why decoding stopped.
    ///
    /// # Errors
    ///
    /// Returns hook errors when `input_index` is outside `input`, or when a
    /// concrete policy hook rejects a value.
    pub fn transcode<Value>(
        &mut self,
        input: &[Unit],
        input_index: usize,
        output: &mut [Value],
        output_index: usize,
    ) -> Result<TranscodeProgress, <H as BufferedDecodeHooks<C, Unit, Value>>::Error>
    where
        C: Codec<Value, Unit>,
        H: BufferedDecodeHooks<C, Unit, Value>,
        Unit: Copy,
    {
        if input_index > input.len() {
            return Err(<H::Error as DecodeErrorFactory<C>>::invalid_input_index(
                &self.codec,
                input_index,
                input.len(),
            ));
        }
        debug_assert_unit_bounds::<C, Value, Unit>(&self.codec);
        let min_units = self.codec.min_units_per_value();
        let mut state = DecodeState::new(input, input_index, output, output_index, min_units);
        if !state.output_cursor_in_bounds() {
            return Ok(state.need_output_progress());
        }

        while state.has_input() {
            if state.needs_input() {
                return Ok(state.need_input_progress());
            }

            // SAFETY: `needs_input()` returned false, so the state has at
            // least `min_units_per_value()` units available from the current
            // cursor.
            let result = unsafe { self.decode_unchecked_at(state.input(), state.input_cursor()) };
            if let Some(progress) = self.handle_decode_result(&mut state, result)? {
                return Ok(progress);
            }
        }

        Ok(state.complete_progress())
    }

    /// Finishes hook-owned output after EOF.
    ///
    /// The engine owns no final output state itself. Hook implementations may
    /// finish their own retained state and emit final output after the caller
    /// has handled any incomplete input tail.
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output value slice visible to the decoder.
    /// - `output_index`: Absolute output value index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns hook-provided finalization progress.
    ///
    /// # Errors
    ///
    /// Returns hook errors when finalization fails.
    pub fn finish<Value>(
        &mut self,
        output: &mut [Value],
        output_index: usize,
    ) -> Result<TranscodeProgress, <H as BufferedDecodeHooks<C, Unit, Value>>::Error>
    where
        C: Codec<Value, Unit>,
        H: BufferedDecodeHooks<C, Unit, Value>,
        Unit: Copy,
    {
        if output_index > output.len() {
            return Ok(TranscodeProgress::need_output(output_index, 1, 0, 0, 0));
        }
        self.hooks.finish(&self.codec, output, output_index)
    }

    /// Decodes one value at a caller-proven readable input cursor.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that at least `codec.min_units_per_value()`
    /// units are readable from `input_index`.
    #[inline(always)]
    pub(crate) unsafe fn decode_unchecked_at<Value>(
        &self,
        input: &[Unit],
        input_index: usize,
    ) -> Result<(Value, NonZeroUsize), C::DecodeError>
    where
        C: Codec<Value, Unit>,
        Unit: Copy,
    {
        // SAFETY: Forwarded from this method's safety contract.
        unsafe { self.codec.decode_unchecked(input, input_index) }
    }

    /// Lets the configured decode hooks classify a low-level decode error.
    #[inline]
    pub(crate) fn handle_decode_error<Value>(
        &mut self,
        error: C::DecodeError,
        context: DecodeContext,
    ) -> Result<DecodeAction<Value>, <H as BufferedDecodeHooks<C, Unit, Value>>::Error>
    where
        C: Codec<Value, Unit>,
        H: BufferedDecodeHooks<C, Unit, Value>,
        Unit: Copy,
    {
        self.hooks.handle_decode_error(&self.codec, error, context)
    }

    /// Handles one low-level decode result and updates the decode state.
    ///
    /// # Parameters
    ///
    /// - `state`: Mutable decode call state.
    /// - `result`: Low-level codec decode result.
    ///
    /// # Returns
    ///
    /// Returns `Some(progress)` when the caller must stop transcoding, or `None`
    /// when the main loop should continue.
    ///
    /// # Errors
    ///
    /// Returns hook errors when the policy rejects the input.
    fn handle_decode_result<Value>(
        &mut self,
        state: &mut DecodeState<'_, Unit, Value>,
        result: Result<(Value, NonZeroUsize), C::DecodeError>,
    ) -> Result<Option<TranscodeProgress>, <H as BufferedDecodeHooks<C, Unit, Value>>::Error>
    where
        C: Codec<Value, Unit>,
        H: BufferedDecodeHooks<C, Unit, Value>,
        Unit: Copy,
    {
        match result {
            Ok((value, consumed)) => {
                let input_index = state.input_cursor();
                Ok(DecodeAttempt::decoded(value, consumed, input_index).apply_to_decode_state(state))
            }
            Err(error) => {
                let context = state.context();
                let action = self.handle_decode_error(error, context)?;
                Ok(action
                    .into_attempt(context.input_index, context.available)
                    .apply_to_decode_state(state))
            }
        }
    }
}
