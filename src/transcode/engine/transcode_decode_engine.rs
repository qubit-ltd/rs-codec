// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Reusable buffered decoder engine.

use core::num::NonZeroUsize;

use super::super::internal::{decode_state::DecodeState, lifecycle::LifecycleGuard};
use crate::codec::assert_unit_bounds;
use crate::{
    CapacityError, Codec, CodecPhase, DecodeContext, DecodeFailure, DecodeInvalidAction,
    DecodeOutcome, TranscodeDecodeError, TranscodeDecodeHooks, TranscodeError, TranscodeProgress,
    Transcoder,
};

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
/// [`crate::TranscodeDecodeHooks`], allowing a policy to skip invalid units,
/// emit a replacement value, or fail.
///
/// The engine stops before reading an incomplete value when fewer than
/// [`Codec::MIN_UNITS_PER_VALUE`] units are available. For variable-width
/// codecs, the codec may still return an incomplete decode error after that
/// minimum is satisfied; the engine converts that failure directly into
/// [`crate::TranscodeStatus::NeedInput`].
///
/// For strict decoding that wraps codec errors, use
/// [`crate::CodecTranscodeDecoder`]. Use `TranscodeDecodeEngine` directly when
/// invalid input should be repaired, skipped, counted, or otherwise handled by
/// policy.
///
/// # Example
///
/// ```rust
/// use core::num::NonZeroUsize;
/// use qubit_codec::{
///     Codec,
///     DecodeFailure,
///     DecodeInvalidAction,
///     DecodeContext,
///     TranscodeStatus,
///     TranscodeDecodeEngine,
///     TranscodeDecodeHooks,
///     TranscodeError,
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
/// impl Codec for ByteCodec {
///     type Value = u8;
///     type Unit = u8;
///     type DecodeError = ByteDecodeError;
///     type EncodeError = core::convert::Infallible;
///
///     const MIN_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;
///     const MAX_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;
///
///     unsafe fn decode(
///         &mut self,
///         input: &[u8],
///         index: usize,
///     ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
///         match input[index] {
///             0xff => Err(DecodeFailure::invalid(
///                 ByteDecodeError::Malformed {
///                     consumed: NonZeroUsize::MIN,
///                 },
///                 NonZeroUsize::MIN,
///             )),
///             value => Ok((value, NonZeroUsize::MIN)),
///         }
///     }
///
///     unsafe fn encode(
///         &mut self,
///         value: &u8,
///         output: &mut [u8],
///         index: usize,
///     ) -> Result<NonZeroUsize, Self::EncodeError> {
///         output[index] = *value;
///         Ok(NonZeroUsize::MIN)
///     }
/// }
///
/// struct ReplacementHooks;
///
/// impl TranscodeDecodeHooks<ByteCodec> for ReplacementHooks {
///     fn handle_invalid_decode(
///         &mut self,
///         _codec: &mut ByteCodec,
///         error: &ByteDecodeError,
///         consumed: Option<NonZeroUsize>,
///         _context: DecodeContext,
///     ) -> Result<DecodeInvalidAction<u8>, qubit_codec::TranscodeDecodeError<ByteCodec>> {
///         match error {
///             ByteDecodeError::Malformed { .. } => {
///                 Ok(DecodeInvalidAction::Emit {
///                     value: b'?',
///                     consumed: consumed.expect("codec reported malformed width"),
///                 })
///             }
///         }
///     }
/// }
///
/// let mut engine = TranscodeDecodeEngine::<_, _>::new(ByteCodec, ReplacementHooks);
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
/// # Ok::<(), TranscodeError<ByteDecodeError>>(())
/// ```
///
/// # Type Parameters
///
/// - `C`: Low-level codec used by the engine.
/// - `H`: Policy hook object used by the engine.
#[derive(Debug, Default)]
pub struct TranscodeDecodeEngine<C, H> {
    /// Low-level codec used for one-value decoding.
    pub(super) codec: C,
    /// Policy hooks used for decode failures.
    pub(super) hooks: H,
    /// Debug-only guard for the `reset → transcode* → finish` lifecycle.
    /// Zero-sized in release builds.
    lifecycle: LifecycleGuard,
}

impl<C, H> TranscodeDecodeEngine<C, H>
where
    C: Codec,
    H: TranscodeDecodeHooks<C>,
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
    ///
    /// # Panics
    ///
    /// In debug builds, panics when the supplied codec violates the
    /// [`Codec::MIN_UNITS_PER_VALUE`] / [`Codec::MAX_UNITS_PER_VALUE`] ordering
    /// invariant. Release builds skip this check because the invariant is the
    /// responsibility of the [`Codec`] implementation.
    #[inline]
    #[must_use]
    pub fn new(codec: C, hooks: H) -> Self {
        assert_unit_bounds::<C>();
        Self {
            codec,
            hooks,
            lifecycle: LifecycleGuard::new(),
        }
    }

    /// Returns the wrapped low-level codec.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the codec owned by this engine.
    #[inline(always)]
    #[must_use]
    pub const fn codec(&self) -> &C {
        &self.codec
    }

    /// Returns the wrapped low-level codec mutably.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the codec owned by this engine.
    #[inline(always)]
    #[must_use]
    pub fn codec_mut(&mut self) -> &mut C {
        &mut self.codec
    }

    /// Returns the decode hooks used by this engine.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the hook object owned by this engine.
    #[inline(always)]
    #[must_use]
    pub const fn hooks(&self) -> &H {
        &self.hooks
    }

    /// Returns the decode hooks mutably.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the hook object owned by this engine.
    #[inline(always)]
    #[must_use]
    pub fn hooks_mut(&mut self) -> &mut H {
        &mut self.hooks
    }

    /// Consumes the engine and returns its codec and hooks.
    ///
    /// Any lifecycle state owned by the engine is discarded.
    ///
    /// # Returns
    ///
    /// Returns the wrapped codec followed by the decode hooks.
    #[inline]
    #[must_use]
    pub fn into_parts(self) -> (C, H) {
        let Self { codec, hooks, .. } = self;
        (codec, hooks)
    }

    /// Returns an upper bound for decoded values produced from `input_len`
    /// units.
    ///
    /// This bound covers only the streaming decode phase. It is delegated to
    /// [`TranscodeDecodeHooks::max_transcode_output_len`], so it includes hook
    /// policy. Downstream decoders must use this engine-level API for capacity
    /// planning instead of recomputing the bound from [`Codec`] constants.
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
    pub fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, TranscodeDecodeError<C>> {
        self.hooks.max_transcode_output_len(&self.codec, input_len)
    }

    /// Returns the maximum values emitted by flushing codec state and finishing
    /// hook-owned state.
    ///
    /// # Returns
    ///
    /// Returns the sum of [`Codec::MAX_DECODE_FLUSH_VALUES`] and the
    /// hook-provided final-output bound. The codec flush portion covers values
    /// written by [`Codec::decode_flush`]; hook implementations must not
    /// include that portion in
    /// [`TranscodeDecodeHooks::max_finish_output_len`].
    #[must_use = "capacity planning can fail on overflow"]
    #[inline(always)]
    pub fn max_finish_output_len(&self) -> Result<usize, TranscodeDecodeError<C>> {
        C::MAX_DECODE_FLUSH_VALUES
            .checked_add(self.hooks.max_finish_output_len(&self.codec))
            .ok_or_else(TranscodeError::output_length_overflow)
    }

    /// Returns the maximum values emitted when resetting stream state.
    ///
    /// Returns [`Codec::MAX_DECODE_RESET_VALUES`] for the wrapped codec.
    /// Stateless decoders always return `0`.
    #[inline(always)]
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        Ok(C::MAX_DECODE_RESET_VALUES)
    }

    /// Returns the maximum values needed by a complete one-shot decode stream.
    ///
    /// The returned bound covers reset output, the streaming decode phase for
    /// `input_len` units, and finish output. Higher-level complete decode
    /// helpers should use this engine-level bound instead of recomputing
    /// capacity from [`Codec`] constants, because hook policy may change
    /// streaming or finish output.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Number of source units in the complete stream.
    ///
    /// # Returns
    ///
    /// Returns the complete-stream output bound, or a capacity error on
    /// arithmetic overflow.
    #[inline(never)]
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_total_output_len(&self, input_len: usize) -> Result<usize, TranscodeDecodeError<C>> {
        checked_stream_total(
            self.max_reset_output_len()?,
            self.max_transcode_output_len(input_len)?,
            self.max_finish_output_len()?,
        )
        .ok_or_else(TranscodeError::output_length_overflow)
    }

    /// Resets codec decode state, runs reset hooks, and emits stream-start
    /// values.
    ///
    /// The sequence is: validate capacity → run `reset_hooks` → call
    /// [`Codec::decode_reset`]. Stateless decoders (`MAX_DECODE_RESET_VALUES
    /// == 0`) write nothing and return `Ok(0)`.
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output value slice visible to the decoder.
    /// - `output_index`: Absolute output value index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of reset values written.
    ///
    /// # Errors
    ///
    /// Returns framework errors when the caller provides invalid or
    /// insufficient output capacity. Returns domain errors when codec reset
    /// or hook reset handling fails.
    pub fn reset(
        &mut self,
        output: &mut [C::Value],
        output_index: usize,
    ) -> Result<usize, TranscodeDecodeError<C>> {
        self.lifecycle.on_reset();
        let required = C::MAX_DECODE_RESET_VALUES;
        TranscodeError::ensure_output_capacity(output.len(), output_index, required)?;
        self.hooks.reset_hooks(&mut self.codec);
        let written = unsafe {
            // SAFETY: The capacity check above reserves the codec's declared
            // reset-output bound at `output_index`.
            self.codec.decode_reset(output, output_index)
        }
        .map_err(|error| TranscodeError::domain(error, CodecPhase::Reset, None))?;
        assert!(
            written <= required,
            "Codec::decode_reset wrote beyond its reset bound",
        );
        Ok(written)
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
    ) -> Result<TranscodeProgress, TranscodeDecodeError<C>> {
        self.lifecycle.on_transcode();
        TranscodeError::ensure_transcode_indices(
            input.len(),
            input_index,
            output.len(),
            output_index,
        )?;

        let min_units = C::MIN_UNITS_PER_VALUE;
        let mut state = DecodeState::new(input, input_index, output, output_index);
        while state.has_input() {
            let context = state.context();
            let available = context.available();
            if available < min_units.get() {
                return Ok(state.need_input_progress_with(min_units, available));
            }
            if state.needs_output() {
                return Ok(state.need_output_progress());
            }
            let output_index = state.output_cursor();
            let output = state.output_mut();
            let (outcome, _) = self.decode_one(input, context, |value, _input_index| {
                // SAFETY: `needs_output()` returned false, so the output
                // cursor points at a writable slot. `ptr::write` moves
                // the decoded value into that slot without requiring
                // `C::Value: Copy`.
                unsafe {
                    output.as_mut_ptr().add(output_index).write(value);
                }
            })?;
            if let Some(progress) = state.apply_decode_outcome(outcome) {
                return Ok(progress);
            }
        }

        Ok(state.complete_progress())
    }

    /// Finishes codec and hook-owned output after EOF.
    ///
    /// Finalization first flushes decode-side codec state through
    /// [`Codec::decode_flush`], then lets hook implementations finish their
    /// own retained state. The caller must provide enough output capacity for
    /// [`TranscodeDecodeEngine::max_finish_output_len`], which includes both
    /// the codec flush bound and the hook-owned finish bound.
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
    /// Returns framework errors when the caller provides invalid or
    /// insufficient output capacity. Returns domain errors when codec flush or
    /// hook finalization fails.
    ///
    /// # Panics
    ///
    /// Panics when the codec flush writes beyond
    /// [`Codec::MAX_DECODE_FLUSH_VALUES`] or when the combined codec and hook
    /// finalization writes beyond
    /// [`TranscodeDecodeEngine::max_finish_output_len`].
    pub fn finish(
        &mut self,
        output: &mut [C::Value],
        output_index: usize,
    ) -> Result<usize, TranscodeDecodeError<C>> {
        self.lifecycle.on_finish_attempt();
        let required = self.max_finish_output_len()?;
        TranscodeError::ensure_output_capacity(output.len(), output_index, required)?;
        let flushed = unsafe { self.codec.decode_flush(output, output_index) }
            .map_err(|error| TranscodeError::domain(error, CodecPhase::Flush, None))?;
        assert!(
            flushed <= C::MAX_DECODE_FLUSH_VALUES,
            "Codec::decode_flush wrote beyond its flush bound",
        );
        let written = self
            .hooks
            .finish_hooks(&mut self.codec, output, output_index + flushed)?;
        assert!(
            flushed + written <= required,
            "TranscodeDecodeEngine hook wrote beyond its finish bound",
        );
        self.lifecycle.on_finish_success();
        Ok(flushed + written)
    }

    /// Runs a complete one-shot `reset -> transcode -> finish` decode stream.
    ///
    /// The complete input is supplied as `input`, and output starts at index
    /// `0` in `output`. Callers that need subranges should slice their
    /// buffers before calling this method. Downstream one-shot decoder
    /// helpers should call this engine method instead of reproducing the
    /// reset, transcode, and finish sequence themselves.
    ///
    /// # Parameters
    ///
    /// - `input`: Complete source unit slice.
    /// - `output`: Output value slice for the whole decoded stream.
    ///
    /// # Returns
    ///
    /// Returns the number of output values written.
    ///
    /// # Errors
    ///
    /// Returns framework errors for insufficient output, capacity overflow, or
    /// an incomplete EOF tail, and domain errors from reset, decode, or
    /// finish.
    #[inline(always)]
    pub fn transcode_complete_into(
        &mut self,
        input: &[C::Unit],
        output: &mut [C::Value],
    ) -> Result<usize, TranscodeDecodeError<C>> {
        <Self as Transcoder<C::Unit, C::Value>>::transcode_complete_into(self, input, output)
    }

    /// Decodes one source value attempt and delivers emitted values.
    ///
    /// # Parameters
    ///
    /// - `input`: Complete input unit slice visible to the caller.
    /// - `context`: Decode context describing the current source and output
    ///   cursors.
    /// - `consume`: Callback invoked exactly once when this attempt emits a
    ///   logical value.
    ///
    /// # Type Parameters
    ///
    /// - `R`: Value returned by the consumer when a decoded value is emitted.
    /// - `F`: Consumer callback type.
    ///
    /// # Returns
    ///
    /// Returns the decode outcome and the consumer result when a value was
    /// emitted.
    ///
    /// # Errors
    ///
    /// Returns hook errors when the decode policy rejects the input.
    pub(crate) fn decode_one<R, F>(
        &mut self,
        input: &[C::Unit],
        context: DecodeContext,
        consume: F,
    ) -> Result<(DecodeOutcome, Option<R>), TranscodeDecodeError<C>>
    where
        F: FnOnce(C::Value, usize) -> R,
    {
        debug_assert!(
            context.available() >= C::MIN_UNITS_PER_VALUE.get(),
            "decode_one requires at least Codec::MIN_UNITS_PER_VALUE input units",
        );

        // SAFETY: The context reports at least `MIN_UNITS_PER_VALUE` source
        // units available from `context.input_index()`.
        let result = unsafe { self.codec.decode(input, context.input_index()) };
        match result {
            Ok((value, consumed)) => {
                assert!(
                    consumed.get() <= context.available(),
                    "Codec::decode consumed beyond available input",
                );
                let consumed_value = consume(value, context.input_index());
                Ok((
                    DecodeOutcome::emitted(consumed, NonZeroUsize::MIN),
                    Some(consumed_value),
                ))
            }
            Err(DecodeFailure::Incomplete { required_total }) => {
                assert!(
                    required_total.get() > context.available(),
                    "Codec::decode incomplete required_total must exceed available input",
                );
                Ok((DecodeOutcome::need_input(required_total), None))
            }
            Err(DecodeFailure::Invalid { source, consumed }) => {
                let action = self.hooks.handle_invalid_decode(
                    &mut self.codec,
                    &source,
                    consumed,
                    context,
                )?;
                if matches!(action, DecodeInvalidAction::Reject) {
                    return Err(TranscodeError::domain(
                        source,
                        CodecPhase::Main,
                        Some(context.input_index()),
                    ));
                }
                Ok(Self::apply_invalid_decode_action(action, context, consume))
            }
        }
    }

    /// Applies a hook-selected invalid-decode action.
    fn apply_invalid_decode_action<R, F>(
        action: DecodeInvalidAction<C::Value>,
        context: DecodeContext,
        consume: F,
    ) -> (DecodeOutcome, Option<R>)
    where
        F: FnOnce(C::Value, usize) -> R,
    {
        match action {
            DecodeInvalidAction::Skip { consumed } => {
                let read = DecodeInvalidAction::<C::Value>::bound_consumed(
                    consumed,
                    context.available(),
                );
                (DecodeOutcome::skipped(read), None)
            }
            DecodeInvalidAction::Emit { value, consumed } => {
                let read = DecodeInvalidAction::<C::Value>::bound_consumed(
                    consumed,
                    context.available(),
                );
                let consumed_value = consume(value, context.input_index());
                (
                    DecodeOutcome::emitted(read, NonZeroUsize::MIN),
                    Some(consumed_value),
                )
            }
            DecodeInvalidAction::Reject => {
                debug_assert!(
                    false,
                    "DecodeInvalidAction::Reject is handled before action application",
                );
                (DecodeOutcome::skipped(NonZeroUsize::MIN), None)
            }
        }
    }
}

impl<C, H> Transcoder<C::Unit, C::Value> for TranscodeDecodeEngine<C, H>
where
    C: Codec,
    H: TranscodeDecodeHooks<C>,
{
    type Error = TranscodeDecodeError<C>;
    type DomainError = C::DecodeError;

    /// Returns the engine error unchanged.
    #[inline(always)]
    fn map_error(&self, error: TranscodeError<Self::DomainError>) -> Self::Error {
        error
    }

    /// Returns an upper bound for decoded values produced from `input_len`
    /// units.
    #[inline(always)]
    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        TranscodeDecodeEngine::max_transcode_output_len(self, input_len)
            .map_err(transcode_capacity_error)
    }

    /// Returns an upper bound for values produced by finishing codec and hook
    /// state.
    #[inline(always)]
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        TranscodeDecodeEngine::max_finish_output_len(self).map_err(transcode_capacity_error)
    }

    /// Returns an upper bound for values emitted when resetting stream state.
    #[inline(always)]
    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        TranscodeDecodeEngine::max_reset_output_len(self)
    }

    /// Runs hook-owned cleanup before a logical decoder reset.
    #[inline(always)]
    fn reset(
        &mut self,
        output: &mut [C::Value],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        TranscodeDecodeEngine::reset(self, output, output_index)
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
        TranscodeDecodeEngine::transcode(self, input, input_index, output, output_index)
    }

    /// Finishes internally retained output after EOF.
    #[inline(always)]
    fn finish(
        &mut self,
        output: &mut [C::Value],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        TranscodeDecodeEngine::finish(self, output, output_index)
    }
}

/// Adds reset, transcode, and finish bounds for one complete stream.
#[inline(never)]
fn checked_stream_total(reset: usize, transcode: usize, finish: usize) -> Option<usize> {
    reset
        .checked_add(transcode)
        .and_then(|len| len.checked_add(finish))
}

/// Converts planning failures from hook-shaped errors into capacity errors.
#[inline(always)]
fn transcode_capacity_error<E>(_error: TranscodeError<E>) -> CapacityError {
    CapacityError::OutputLengthOverflow
}
