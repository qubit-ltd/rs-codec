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
    decode_context::DecodeContext,
    decode_state::DecodeState,
    decode_step::DecodeStep,
    transcode_progress::TranscodeProgress,
};
use crate::{
    CapacityError,
    Codec,
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
/// ```rust
/// use core::num::NonZeroUsize;
/// use qubit_codec::{
///     BufferedDecodeEngine,
///     BufferedDecodeHooks,
///     Codec,
///     DecodeAction,
///     DecodeContext,
///     TranscodeStatus,
/// };
///
/// #[derive(Clone, Copy)]
/// struct ByteCodec;
///
/// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// enum ByteDecodeError {
///     Malformed { consumed: NonZeroUsize },
/// }
///
/// unsafe impl Codec<u8, u8> for ByteCodec {
///     type DecodeError = ByteDecodeError;
///     type EncodeError = core::convert::Infallible;
///
///     fn min_units_per_value(&self) -> NonZeroUsize {
///         NonZeroUsize::MIN
///     }
///
///     fn max_units_per_value(&self) -> NonZeroUsize {
///         NonZeroUsize::MIN
///     }
///
///     unsafe fn decode_unchecked(
///         &self,
///         input: &[u8],
///         index: usize,
///     ) -> Result<(u8, NonZeroUsize), Self::DecodeError> {
///         match input[index] {
///             0xff => Err(ByteDecodeError::Malformed {
///                 consumed: NonZeroUsize::MIN,
///             }),
///             value => Ok((value, NonZeroUsize::MIN)),
///         }
///     }
///
///     unsafe fn encode_unchecked(
///         &self,
///         value: &u8,
///         output: &mut [u8],
///         index: usize,
///     ) -> Result<usize, Self::EncodeError> {
///         output[index] = *value;
///         Ok(1)
///     }
/// }
///
/// struct ReplacementHooks;
///
/// impl BufferedDecodeHooks<ByteCodec, u8, u8> for ReplacementHooks {
///     type Error = ByteDecodeError;
///
///     fn handle_decode_error(
///         &mut self,
///         _codec: &ByteCodec,
///         error: ByteDecodeError,
///         _context: DecodeContext,
///     ) -> Result<DecodeAction<u8>, Self::Error> {
///         match error {
///             ByteDecodeError::Malformed { consumed } => {
///                 Ok(DecodeAction::Emit { value: b'?', consumed })
///             }
///         }
///     }
///
///     fn invalid_input_index(
///         &mut self,
///         _codec: &ByteCodec,
///         _index: usize,
///         _input_len: usize,
///     ) -> Self::Error {
///         ByteDecodeError::Malformed {
///             consumed: NonZeroUsize::MIN,
///         }
///     }
/// }
///
/// let mut engine = BufferedDecodeEngine::<_, _, u8, u8>::new(ByteCodec, ReplacementHooks);
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
/// # Ok::<(), ByteDecodeError>(())
/// ```
///
/// # Type Parameters
///
/// - `C`: Low-level codec used by the engine.
/// - `H`: Policy hook object used by the engine.
/// - `Unit`: Encoded input unit type accepted by the engine.
/// - `Value`: Logical value decoded by the engine.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct BufferedDecodeEngine<C, H, Unit, Value> {
    /// Low-level codec used for one-value decoding.
    pub(super) codec: C,
    /// Policy hooks used for decode failures.
    pub(super) hooks: H,
    /// Binds the engine to the encoded input unit and decoded value types.
    marker: PhantomData<fn(Unit) -> Value>,
}

impl<C, H, Unit, Value> BufferedDecodeEngine<C, H, Unit, Value>
where
    C: Codec<Value, Unit>,
    H: BufferedDecodeHooks<C, Unit, Value>,
    Unit: Copy,
{
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
    pub fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        debug_assert_unit_bounds::<C, Value, Unit>(&self.codec);
        self.hooks.max_output_len(&self.codec, input_len)
    }

    /// Returns the maximum values emitted by finishing hook-owned state.
    ///
    /// # Returns
    ///
    /// Returns the hook-provided final output bound.
    #[must_use]
    pub fn max_finish_output_len(&self) -> usize {
        self.hooks.max_finish_output_len(&self.codec)
    }

    /// Resets hook-owned state.
    pub fn reset(&mut self) {
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
    pub fn transcode(
        &mut self,
        input: &[Unit],
        input_index: usize,
        output: &mut [Value],
        output_index: usize,
    ) -> Result<TranscodeProgress, H::Error> {
        if input_index > input.len() {
            return Err(self.hooks.invalid_input_index(&self.codec, input_index, input.len()));
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
    pub fn finish(&mut self, output: &mut [Value], output_index: usize) -> Result<TranscodeProgress, H::Error> {
        if output_index > output.len() {
            return Ok(TranscodeProgress::need_output(output_index, NonZeroUsize::MIN, 0, 0, 0));
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
    pub(crate) unsafe fn decode_unchecked_at(
        &self,
        input: &[Unit],
        input_index: usize,
    ) -> Result<(Value, NonZeroUsize), C::DecodeError> {
        // SAFETY: Forwarded from this method's safety contract.
        unsafe { self.codec.decode_unchecked(input, input_index) }
    }

    /// Lets the configured decode hooks classify a low-level decode error.
    #[inline]
    pub(crate) fn handle_decode_error(
        &mut self,
        error: C::DecodeError,
        context: DecodeContext,
    ) -> Result<DecodeAction<Value>, H::Error> {
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
    fn handle_decode_result(
        &mut self,
        state: &mut DecodeState<'_, Unit, Value>,
        result: Result<(Value, NonZeroUsize), C::DecodeError>,
    ) -> Result<Option<TranscodeProgress>, H::Error> {
        match result {
            Ok((value, consumed)) => {
                let input_index = state.input_cursor();
                Ok(DecodeStep::decoded(value, consumed, input_index).apply_to_decode_state(state))
            }
            Err(error) => {
                let context = state.context();
                let action = self.handle_decode_error(error, context)?;
                Ok(action
                    .into_step(context.input_index, context.available)
                    .apply_to_decode_state(state))
            }
        }
    }
}
