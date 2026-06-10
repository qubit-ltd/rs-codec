// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Reusable buffered decoder engine.

use core::num::NonZeroUsize;

use super::{
    buffered_decode_hooks::BufferedDecodeHooks, buffered_transcoder::BufferedTranscoder,
    decode_action::DecodeAction, decode_context::DecodeContext, decode_state::DecodeState,
    decode_step::DecodeStep, finish_error::FinishError, transcode_progress::TranscodeProgress,
};
use crate::{CapacityError, Codec, codec::assert_unit_bounds};

/// Reusable buffered decoding engine for codec-backed decoders.
///
/// The engine owns the low-level codec and hook object. It keeps the common
/// buffered decoding loop private: input-index validation, output-capacity
/// checks, calls to [`Codec::decode`], hook dispatch, and
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
/// unsafe impl Codec for ByteCodec {
///     type Value = u8;
///     type Unit = u8;
///     type DecodeError = ByteDecodeError;
///     type EncodeError = core::convert::Infallible;
///     type DecodeState = ();
///     type EncodeState = ();
///
///     fn min_units_per_value(&self) -> NonZeroUsize {
///         NonZeroUsize::MIN
///     }
///
///     fn max_units_per_value(&self) -> NonZeroUsize {
///         NonZeroUsize::MIN
///     }
///
///     unsafe fn decode(
///         &mut self,
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
///     unsafe fn encode(
///         &mut self,
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
/// impl BufferedDecodeHooks<ByteCodec> for ReplacementHooks {
///     type Error = ByteDecodeError;
///
///     fn handle_decode_error(
///         &mut self,
///         _codec: &mut ByteCodec,
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
///         _codec: &mut ByteCodec,
///         _index: usize,
///         _input_len: usize,
///     ) -> Self::Error {
///         ByteDecodeError::Malformed {
///             consumed: NonZeroUsize::MIN,
///         }
///     }
///
///     fn invalid_output_index(
///         &mut self,
///         _codec: &mut ByteCodec,
///         _index: usize,
///         _output_len: usize,
///     ) -> Self::Error {
///         ByteDecodeError::Malformed {
///             consumed: NonZeroUsize::MIN,
///         }
///     }
/// }
///
/// let mut engine = BufferedDecodeEngine::<_, _>::new(ByteCodec, ReplacementHooks);
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
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct BufferedDecodeEngine<C, H> {
    /// Low-level codec used for one-value decoding.
    pub(super) codec: C,
    /// Policy hooks used for decode failures.
    pub(super) hooks: H,
}

impl<C, H> BufferedDecodeEngine<C, H>
where
    C: Codec,
    H: BufferedDecodeHooks<C>,
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
    #[inline(always)]
    pub const fn new(codec: C, hooks: H) -> Self {
        Self { codec, hooks }
    }

    /// Returns an upper bound for decoded values produced from `input_len`
    /// units.
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
    #[inline(always)]
    pub fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        assert_unit_bounds::<C>(&self.codec);
        self.hooks.max_output_len(&self.codec, input_len)
    }

    /// Returns the maximum values emitted by flushing codec state and finishing
    /// hook-owned state.
    ///
    /// # Returns
    ///
    /// Returns the hook-provided final output bound.
    #[must_use]
    #[inline(always)]
    pub fn max_finish_output_len(&self) -> usize {
        self.codec.max_decode_flush_values() + self.hooks.max_finish_output_len(&self.codec)
    }

    /// Resets codec decode state and hook-owned state.
    ///
    /// # Parameters
    ///
    /// - `self`: Decoder instance whose hook state is reset.
    ///
    /// # Returns
    ///
    /// Returns unit `()`.
    #[inline(always)]
    pub fn reset(&mut self) {
        self.codec.reset_decode_state();
        self.hooks.reset(&mut self.codec);
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
    /// Returns hook errors when `input_index` is outside `input`, when
    /// `output_index` is outside `output`, or when a concrete policy hook
    /// rejects a value.
    pub fn transcode(
        &mut self,
        input: &[C::Unit],
        input_index: usize,
        output: &mut [C::Value],
        output_index: usize,
    ) -> Result<TranscodeProgress, H::Error> {
        if input_index > input.len() {
            return Err(self
                .hooks
                .invalid_input_index(&mut self.codec, input_index, input.len()));
        }
        if output_index > output.len() {
            return Err(self.hooks.invalid_output_index(
                &mut self.codec,
                output_index,
                output.len(),
            ));
        }
        assert_unit_bounds::<C>(&self.codec);
        let mut state = DecodeState::new(input, input_index, output, output_index);

        while state.has_input() {
            let context = state.context();
            let min_units = self.codec.min_units_per_value().get();
            if context.available < min_units {
                let additional = NonZeroUsize::new(min_units - context.available)
                    .expect("missing input is non-zero");
                return Ok(state.need_input_progress_with(additional, context.available));
            }
            if state.needs_output() {
                return Ok(state.need_output_progress());
            }
            let step = self.decode_step(state.input(), context)?;
            if let Some(progress) = step.apply_to_decode_state(&mut state) {
                return Ok(progress);
            }
        }

        Ok(state.complete_progress())
    }

    /// Finishes hook-owned output after EOF.
    ///
    /// The engine owns no final output state itself. Hook implementations may
    /// finish their own retained state and emit final output after the caller
    /// has handled any incomplete input tail. The caller must provide enough
    /// output capacity for [`BufferedDecodeEngine::max_finish_output_len`].
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output value slice visible to the decoder.
    /// - `output_index`: Absolute output value index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of values written by finalization.
    ///
    /// # Errors
    ///
    /// Returns [`FinishError`] when the caller provides invalid or insufficient
    /// output capacity, or when hook finalization fails.
    ///
    /// # Panics
    ///
    /// Panics when the hook writes or reports more final output values than
    /// [`BufferedDecodeEngine::max_finish_output_len`] declared.
    pub fn finish(
        &mut self,
        output: &mut [C::Value],
        output_index: usize,
    ) -> Result<usize, FinishError<H::Error>> {
        let required = self.max_finish_output_len();
        FinishError::ensure_output_capacity(output.len(), output_index, required)?;
        let output_end = output_index + required;
        let output = &mut output[..output_end];
        let snapshot = self.codec.decode_state();
        let flushed =
            unsafe { self.codec.decode_flush(output, output_index) }.map_err(|error| {
                self.codec.set_decode_state(snapshot);
                FinishError::source(self.hooks.map_decode_flush_error(&mut self.codec, error))
            })?;
        assert!(
            flushed <= self.codec.max_decode_flush_values(),
            "Codec::decode_flush wrote beyond its flush bound",
        );
        let written = self
            .hooks
            .finish(&mut self.codec, output, output_index + flushed)
            .map_err(FinishError::source)?;
        assert!(
            flushed + written <= required,
            "BufferedDecodeEngine hook wrote beyond its finish bound",
        );
        Ok(flushed + written)
    }

    /// Decodes one value at a caller-proven readable input cursor.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that at least `codec.min_units_per_value()`
    /// units are readable from `input_index`.
    #[inline(always)]
    pub(crate) unsafe fn decode_at(
        &mut self,
        input: &[C::Unit],
        input_index: usize,
    ) -> Result<(C::Value, NonZeroUsize), C::DecodeError> {
        // SAFETY: Forwarded from this method's safety contract.
        unsafe { self.codec.decode(input, input_index) }
    }

    /// Lets the configured decode hooks classify a low-level decode error.
    ///
    /// # Parameters
    ///
    /// - `error`: Decode error returned by [`Codec::decode`].
    /// - `context`: Decode context used by policy hooks.
    ///
    /// # Returns
    ///
    /// Returns the decoded action chosen by policy hooks.
    ///
    /// # Errors
    ///
    /// Returns a hook-level error when the decode policy rejects the value.
    #[inline(always)]
    pub(crate) fn handle_decode_error(
        &mut self,
        error: C::DecodeError,
        context: DecodeContext,
    ) -> Result<DecodeAction<C::Value>, H::Error> {
        self.hooks
            .handle_decode_error(&mut self.codec, error, context)
    }

    /// Decodes one source value attempt into a normalized decode step.
    ///
    /// # Parameters
    ///
    /// - `input`: Complete input unit slice visible to the caller.
    /// - `context`: Decode context describing the current source and output
    ///   cursors.
    ///
    /// # Returns
    ///
    /// Returns one internal decode step, including successful decode, policy
    /// skip/emit, or need-input state.
    ///
    /// # Errors
    ///
    /// Returns hook errors when the decode policy rejects the input.
    #[inline]
    pub(super) fn decode_step(
        &mut self,
        input: &[C::Unit],
        context: DecodeContext,
    ) -> Result<DecodeStep<C::Value>, H::Error> {
        let min_units = self.codec.min_units_per_value().get();
        if context.available < min_units {
            let additional = NonZeroUsize::new(min_units - context.available)
                .expect("missing input is non-zero");
            return Ok(DecodeStep::need_input(additional, context.available));
        }

        // SAFETY: The context reports at least `min_units_per_value()` source
        // units available from `context.input_index`.
        let snapshot = self.codec.decode_state();
        let result = unsafe { self.decode_at(input, context.input_index) };
        self.handle_decode_result(context, snapshot, result)
    }

    /// Handles one low-level decode result and returns a normalized decode
    /// step.
    ///
    /// # Parameters
    ///
    /// - `context`: Decode context used by policy hooks.
    /// - `result`: Low-level codec decode result.
    ///
    /// # Returns
    ///
    /// Returns the normalized decode step selected by codec success or policy
    /// hooks.
    ///
    /// # Errors
    ///
    /// Returns hook errors when the policy rejects the input.
    #[inline]
    fn handle_decode_result(
        &mut self,
        context: DecodeContext,
        snapshot: C::DecodeState,
        result: Result<(C::Value, NonZeroUsize), C::DecodeError>,
    ) -> Result<DecodeStep<C::Value>, H::Error> {
        match result {
            Ok((value, consumed)) => {
                assert!(
                    consumed.get() <= context.available,
                    "Codec::decode consumed beyond available input",
                );
                Ok(DecodeStep::decoded(value, consumed, context.input_index))
            }
            Err(error) => {
                self.codec.set_decode_state(snapshot);
                let action = self.handle_decode_error(error, context)?;
                Ok(action.into_step(context.input_index, context.available))
            }
        }
    }
}

impl<C, H> BufferedTranscoder<C::Unit, C::Value> for BufferedDecodeEngine<C, H>
where
    C: Codec,
    H: BufferedDecodeHooks<C>,
{
    type Error = H::Error;

    /// Returns an upper bound for decoded values produced from `input_len`
    /// units.
    #[inline(always)]
    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        BufferedDecodeEngine::max_output_len(self, input_len)
    }

    /// Returns an upper bound for values produced by finishing hook state.
    #[inline(always)]
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(BufferedDecodeEngine::max_finish_output_len(self))
    }

    /// Resets hook-owned state.
    #[inline(always)]
    fn reset(&mut self) {
        BufferedDecodeEngine::reset(self)
    }

    /// Decodes source units into logical values.
    #[inline(always)]
    fn transcode(
        &mut self,
        input: &[C::Unit],
        input_index: usize,
        output: &mut [C::Value],
        output_index: usize,
    ) -> core::result::Result<TranscodeProgress, Self::Error> {
        BufferedDecodeEngine::transcode(self, input, input_index, output, output_index)
    }

    /// Finishes internally retained output after EOF.
    #[inline(always)]
    fn finish(
        &mut self,
        output: &mut [C::Value],
        output_index: usize,
    ) -> core::result::Result<usize, FinishError<Self::Error>> {
        BufferedDecodeEngine::finish(self, output, output_index)
    }
}
