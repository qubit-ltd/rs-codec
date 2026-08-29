// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Reusable buffered decoder engine.

use core::num::NonZeroUsize;

use super::super::internal::applied_decode_invalid_action::AppliedDecodeInvalidAction;
use super::super::internal::decode_state::DecodeState;
use super::super::internal::lifecycle_guard::LifecycleGuard;
use super::DecodeContext;
use super::DecodeIncompleteAction;
use super::DecodeInvalidAction;
use super::DecodeOutcome;
use super::TranscodeDecodeHooks;
use crate::CapacityError;
use crate::Codec;
use crate::DecodeFailure;
use crate::TranscodeDecodeError;
use crate::TranscodeDecodeErrorOf;
use crate::TranscodeDecoder;
use crate::TranscodeFailure;
use crate::TranscodeProgress;
use crate::Transcoder;
use crate::codec::assert_unit_bounds;

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
/// [`crate::engine::TranscodeDecodeHooks`], allowing a policy to skip invalid
/// units, emit a replacement value, or fail.
///
/// The engine stops before reading an incomplete value when fewer than
/// [`Codec::MIN_UNITS_PER_VALUE`] units are available. For variable-width
/// codecs, the codec may still return an incomplete decode error after that
/// minimum is satisfied. Both conditions return
/// [`crate::TranscodeStatus::NeedInput`] while the stream is open; at EOF the
/// hook selects whether to reject, skip, or replace the remaining tail.
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
///     TranscodeDecodeErrorOf,
///     DecodeFailure,
///     TranscodeStatus,
/// };
/// use qubit_codec::engine::{
///     DecodeContext,
///     DecodeInvalidAction,
///     TranscodeDecodeEngine,
///     TranscodeDecodeHooks,
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
///     const MIN_UNITS_PER_VALUE: usize = 1;
///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
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
///     ) -> Result<usize, Self::EncodeError> {
///         output[index] = *value;
///         Ok(1)
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
///     ) -> Result<DecodeInvalidAction<u8>, TranscodeDecodeErrorOf<ByteCodec>> {
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
/// let mut reset_output = [];
/// engine.reset(&mut reset_output, 0)?;
///
/// let progress = engine.transcode(&input, 0, &mut output, 0)?;
/// match progress.status() {
///     TranscodeStatus::Complete => assert_eq!(&output[..progress.written()], b"a?b"),
///     TranscodeStatus::NeedInput { .. } => {
///         // Keep `input[input_index + progress.read()..]`, append more source
///         // units, and resume.
///     }
///     TranscodeStatus::NeedOutput { .. } => {
///         // Drain `output[..progress.written()]`, then resume with more output
///         // room.
///     }
/// }
/// # Ok::<(), qubit_codec::TranscodeDecodeError<ByteDecodeError>>(())
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
    /// Guard for the `reset → transcode* → finish` lifecycle in every build
    /// profile.
    lifecycle: LifecycleGuard,
}

/// Adds two independent decoded-value capacity bounds.
fn add_decode_output_bounds(first: usize, second: usize) -> Result<usize, CapacityError> {
    first.checked_add(second).ok_or(CapacityError::OutputLengthOverflow)
}

/// Adds reset, transcode, and finish decoded-value capacity bounds.
fn sum_decode_output_bounds(reset: usize, transcode: usize, finish: usize) -> Result<usize, CapacityError> {
    let before_finish = add_decode_output_bounds(reset, transcode)?;
    add_decode_output_bounds(before_finish, finish)
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
    /// # Compile-Time Checks
    ///
    /// Fails to compile when the supplied codec declares a zero decode unit
    /// bound or when [`Codec::MIN_UNITS_PER_VALUE`] exceeds
    /// [`Codec::MAX_DECODE_UNITS_PER_VALUE`].
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
    #[inline(always)]
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
    /// policy and is valid for every reachable transient codec and hook state.
    /// Downstream decoders must use this engine-level API for capacity planning
    /// instead of recomputing the bound from [`Codec`] constants.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Number of source units the caller plans to decode.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound, or a capacity error on arithmetic
    /// overflow.
    #[inline(always)]
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        self.hooks.max_transcode_output_len(&self.codec, input_len)
    }

    /// Returns the global maximum values emitted by finishing codec state and
    /// finishing hook-owned state.
    ///
    /// # Returns
    ///
    /// Returns the sum of [`Codec::MAX_DECODE_FINISH_VALUES`] and the
    /// hook-provided final-output bound. The codec finish portion covers values
    /// written by [`Codec::decode_finish`]; hook implementations must not
    /// include that portion in
    /// [`TranscodeDecodeHooks::max_finish_output_len`]. Both component bounds
    /// cover every reachable transient state.
    #[inline(always)]
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        add_decode_output_bounds(
            C::MAX_DECODE_FINISH_VALUES,
            self.hooks.max_finish_output_len(&self.codec),
        )
    }

    /// Returns the global maximum values emitted when resetting stream state.
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
    /// `input_len` units, and finish output. Its components are global across
    /// transient state. Higher-level complete decode helpers should use this
    /// engine-level bound instead of recomputing capacity from [`Codec`]
    /// constants, because hook policy may change streaming or finish output.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Number of source units in the complete stream.
    ///
    /// # Returns
    ///
    /// Returns the complete-stream output bound, or a capacity error on
    /// arithmetic overflow.
    #[inline]
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_total_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        let transcode = self.max_transcode_output_len(input_len)?;
        let finish = self.max_finish_output_len()?;
        sum_decode_output_bounds(C::MAX_DECODE_RESET_VALUES, transcode, finish)
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
    /// or hook reset handling fails. Capacity and index failures occur before
    /// any reset state is changed. Once reset execution starts, an error
    /// poisons the engine until a later reset succeeds.
    pub fn reset(&mut self, output: &mut [C::Value], output_index: usize) -> Result<usize, TranscodeDecodeErrorOf<C>> {
        let required = C::MAX_DECODE_RESET_VALUES;
        TranscodeFailure::ensure_output_capacity(output.len(), output_index, required)?;
        self.lifecycle.on_reset_start();
        self.hooks.reset_hooks(&mut self.codec);
        let written = unsafe {
            // SAFETY: The capacity check above reserves the codec's declared
            // reset-output bound at `output_index`.
            self.codec.decode_reset(output, output_index)
        }
        .map_err(TranscodeDecodeError::domain_reset)?;
        assert!(written <= required, "Codec::decode_reset wrote beyond its reset bound",);
        self.lifecycle.on_reset_success();
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
    /// rejects a value. Returns
    /// [`TranscodeFailure::TranscodeBeforeReset`] when the engine has not
    /// completed its first reset. Returns
    /// [`TranscodeFailure::TranscodeAfterFinish`] when the logical stream was
    /// already finished and has not been reset, or
    /// [`TranscodeFailure::LifecyclePoisoned`] when an earlier reset or finish
    /// failed after execution started.
    pub fn transcode(
        &mut self,
        input: &[C::Unit],
        input_index: usize,
        output: &mut [C::Value],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeDecodeErrorOf<C>> {
        self.transcode_with_eof(input, input_index, output, output_index, false)
    }

    /// Decodes source units after the caller has established end of input.
    ///
    /// This follows the normal streaming contract, except that codec attempts
    /// use [`Codec::decode_eof`]. A codec may therefore resolve a trailing
    /// prefix that would remain incomplete in an open stream.
    #[inline(always)]
    pub fn transcode_eof(
        &mut self,
        input: &[C::Unit],
        input_index: usize,
        output: &mut [C::Value],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeDecodeErrorOf<C>> {
        self.transcode_with_eof(input, input_index, output, output_index, true)
    }

    fn transcode_with_eof(
        &mut self,
        input: &[C::Unit],
        input_index: usize,
        output: &mut [C::Value],
        output_index: usize,
        end_of_input: bool,
    ) -> Result<TranscodeProgress, TranscodeDecodeErrorOf<C>> {
        self.lifecycle.on_transcode()?;
        TranscodeFailure::ensure_transcode_indices(input.len(), input_index, output.len(), output_index)?;

        let min_units = NonZeroUsize::new(C::MIN_UNITS_PER_VALUE).expect("Codec::MIN_UNITS_PER_VALUE is non-zero");
        let min_units_len = min_units.get();
        let mut state = DecodeState::new(input, input_index, output, output_index);
        while state.has_input() {
            let context = state.context();
            let available = context.available();
            if available < min_units_len {
                if !end_of_input {
                    return Ok(state.need_input_progress_with(min_units));
                }
                match self
                    .hooks
                    .handle_incomplete_decode(&mut self.codec, None, min_units, context)?
                {
                    DecodeIncompleteAction::Reject => {
                        return Err(TranscodeFailure::incomplete_input(
                            context.input_index(),
                            min_units.get(),
                            available,
                        )
                        .into());
                    }
                    DecodeIncompleteAction::Skip => {
                        let read =
                            NonZeroUsize::new(available).expect("incomplete decode tail must contain source units");
                        if let Some(progress) = state.apply_decode_outcome(DecodeOutcome::skipped(read)) {
                            return Ok(progress);
                        }
                        continue;
                    }
                    DecodeIncompleteAction::Emit { value } => {
                        if state.needs_output() {
                            return Ok(state.need_output_progress());
                        }
                        let output_index = state.output_cursor();
                        // SAFETY: `needs_output()` returned false, so the
                        // output cursor points at a writable initialized slot.
                        unsafe {
                            *state.output_mut().get_unchecked_mut(output_index) = value;
                        }
                        let read =
                            NonZeroUsize::new(available).expect("incomplete decode tail must contain source units");
                        if let Some(progress) =
                            state.apply_decode_outcome(DecodeOutcome::emitted(read, NonZeroUsize::MIN))
                        {
                            return Ok(progress);
                        }
                        continue;
                    }
                }
            }
            if state.needs_output() {
                return Ok(state.need_output_progress());
            }
            let output_index = state.output_cursor();
            let output = state.output_mut();
            let (outcome, _) = self.decode_one(input, context, end_of_input, |value, _input_index| {
                // SAFETY: `needs_output()` returned false, so the output
                // cursor points at a writable initialized slot.
                unsafe {
                    *output.get_unchecked_mut(output_index) = value;
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
    /// Finalization first finishes decode-side codec state through
    /// [`Codec::decode_finish`], then lets hook implementations finish their
    /// own retained state. The caller must provide enough output capacity for
    /// [`TranscodeDecodeEngine::max_finish_output_len`], which includes both
    /// the codec finish bound and the hook-owned finish bound.
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
    /// insufficient output capacity. Returns domain errors when codec finish or
    /// hook finalization fails. Returns
    /// [`TranscodeFailure::FinishBeforeReset`] when the engine has not
    /// completed its first reset. Returns
    /// [`TranscodeFailure::FinishAfterFinish`] when the logical stream was
    /// already finished and has not been reset, or
    /// [`TranscodeFailure::LifecyclePoisoned`] when an earlier reset or finish
    /// failed after execution started. Capacity and index failures occur before
    /// finish execution and remain retryable; later failures poison the engine
    /// until reset succeeds.
    ///
    /// # Panics
    ///
    /// Panics when the codec finish writes beyond
    /// [`Codec::MAX_DECODE_FINISH_VALUES`] or when the combined codec and hook
    /// finalization writes beyond
    /// [`TranscodeDecodeEngine::max_finish_output_len`].
    pub fn finish(&mut self, output: &mut [C::Value], output_index: usize) -> Result<usize, TranscodeDecodeErrorOf<C>> {
        self.lifecycle.on_finish_attempt()?;
        let required = self.max_finish_output_len()?;
        TranscodeFailure::ensure_output_capacity(output.len(), output_index, required)?;
        self.lifecycle.on_finish_start();
        let finished =
            unsafe { self.codec.decode_finish(output, output_index) }.map_err(TranscodeDecodeError::domain_finish)?;
        assert!(
            finished <= C::MAX_DECODE_FINISH_VALUES,
            "Codec::decode_finish wrote beyond its finish bound",
        );
        let written = self
            .hooks
            .finish_hooks(&mut self.codec, output, output_index + finished)?;
        assert!(
            finished + written <= required,
            "TranscodeDecodeEngine hook wrote beyond its finish bound",
        );
        self.lifecycle.on_finish_success();
        Ok(finished + written)
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
    ) -> Result<usize, TranscodeDecodeErrorOf<C>> {
        <Self as Transcoder>::transcode_complete_into(self, input, output)
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
    ///
    /// # Panics
    ///
    /// Panics when the codec reports consumption beyond the available input,
    /// an incomplete-input requirement beyond
    /// [`Codec::MAX_DECODE_UNITS_PER_VALUE`], or hooks return an action that
    /// consumes beyond the available input.
    pub(crate) fn decode_one<R, F>(
        &mut self,
        input: &[C::Unit],
        context: DecodeContext,
        end_of_input: bool,
        consume: F,
    ) -> Result<(DecodeOutcome, Option<R>), TranscodeDecodeErrorOf<C>>
    where
        F: FnOnce(C::Value, usize) -> R,
    {
        debug_assert!(
            context.available() > 0,
            "decode_one requires at least one source input unit",
        );

        if context.available() < C::MIN_UNITS_PER_VALUE {
            let required_total =
                NonZeroUsize::new(C::MIN_UNITS_PER_VALUE).expect("Codec::MIN_UNITS_PER_VALUE is non-zero");
            if !end_of_input {
                return Ok((DecodeOutcome::need_input(required_total), None));
            }
            return match self
                .hooks
                .handle_incomplete_decode(&mut self.codec, None, required_total, context)?
            {
                DecodeIncompleteAction::Reject => Err(TranscodeFailure::incomplete_input(
                    context.input_index(),
                    required_total.get(),
                    context.available(),
                )
                .into()),
                DecodeIncompleteAction::Skip => {
                    let read = NonZeroUsize::new(context.available())
                        .expect("incomplete decode tail must contain source units");
                    Ok((DecodeOutcome::skipped(read), None))
                }
                DecodeIncompleteAction::Emit { value } => {
                    let read = NonZeroUsize::new(context.available())
                        .expect("incomplete decode tail must contain source units");
                    let consumed_value = consume(value, context.input_index());
                    Ok((DecodeOutcome::emitted(read, NonZeroUsize::MIN), Some(consumed_value)))
                }
            };
        }

        // SAFETY: The context reports at least `MIN_UNITS_PER_VALUE` source
        // units available from `context.input_index()`.
        let result = unsafe {
            if end_of_input {
                self.codec.decode_eof(input, context.input_index())
            } else {
                self.codec.decode(input, context.input_index())
            }
        };
        match result {
            Ok((value, consumed)) => {
                assert!(
                    consumed.get() <= context.available(),
                    "Codec::decode consumed beyond available input",
                );
                assert!(
                    consumed.get() <= C::MAX_DECODE_UNITS_PER_VALUE,
                    "Codec::decode consumed beyond Codec::MAX_DECODE_UNITS_PER_VALUE",
                );
                let consumed_value = consume(value, context.input_index());
                Ok((
                    DecodeOutcome::emitted(consumed, NonZeroUsize::MIN),
                    Some(consumed_value),
                ))
            }
            Err(DecodeFailure::Incomplete { source, required_total }) => {
                assert!(
                    required_total.get() > context.available(),
                    "Codec::decode incomplete required_total must exceed available input",
                );
                assert!(
                    required_total.get() <= C::MAX_DECODE_UNITS_PER_VALUE,
                    "Codec::decode incomplete required_total exceeded Codec::MAX_DECODE_UNITS_PER_VALUE",
                );
                if !end_of_input {
                    return Ok((DecodeOutcome::need_input(required_total), None));
                }
                match self
                    .hooks
                    .handle_incomplete_decode(&mut self.codec, source.as_ref(), required_total, context)?
                {
                    DecodeIncompleteAction::Reject => match source {
                        Some(source) => Err(TranscodeDecodeError::domain_main(source, context.input_index())),
                        None => Err(TranscodeFailure::incomplete_input(
                            context.input_index(),
                            required_total.get(),
                            context.available(),
                        )
                        .into()),
                    },
                    DecodeIncompleteAction::Skip => {
                        let read = NonZeroUsize::new(context.available())
                            .expect("incomplete decode tail must contain source units");
                        Ok((DecodeOutcome::skipped(read), None))
                    }
                    DecodeIncompleteAction::Emit { value } => {
                        let read = NonZeroUsize::new(context.available())
                            .expect("incomplete decode tail must contain source units");
                        let consumed_value = consume(value, context.input_index());
                        Ok((DecodeOutcome::emitted(read, NonZeroUsize::MIN), Some(consumed_value)))
                    }
                }
            }
            Err(DecodeFailure::Invalid { source, consumed }) => {
                let action = match self
                    .hooks
                    .handle_invalid_decode(&mut self.codec, &source, consumed, context)?
                {
                    DecodeInvalidAction::Reject => {
                        return Err(TranscodeDecodeError::domain_main_with_consumed(
                            source,
                            context.input_index(),
                            consumed,
                        ));
                    }
                    DecodeInvalidAction::Skip { consumed } => AppliedDecodeInvalidAction::Skip { consumed },
                    DecodeInvalidAction::Emit { value, consumed } => {
                        AppliedDecodeInvalidAction::Emit { value, consumed }
                    }
                };
                Ok(Self::apply_invalid_decode_action(action, context, consume))
            }
        }
    }

    /// Applies a hook-selected invalid-decode action after reject handling.
    fn apply_invalid_decode_action<R, F>(
        action: AppliedDecodeInvalidAction<C::Value>,
        context: DecodeContext,
        consume: F,
    ) -> (DecodeOutcome, Option<R>)
    where
        F: FnOnce(C::Value, usize) -> R,
    {
        match action {
            AppliedDecodeInvalidAction::Skip { consumed } => {
                let read = DecodeInvalidAction::<C::Value>::bound_consumed(consumed, context.available());
                (DecodeOutcome::skipped(read), None)
            }
            AppliedDecodeInvalidAction::Emit { value, consumed } => {
                let read = DecodeInvalidAction::<C::Value>::bound_consumed(consumed, context.available());
                let consumed_value = consume(value, context.input_index());
                (DecodeOutcome::emitted(read, NonZeroUsize::MIN), Some(consumed_value))
            }
        }
    }
}

impl<C, H> Transcoder for TranscodeDecodeEngine<C, H>
where
    C: Codec,
    H: TranscodeDecodeHooks<C>,
{
    type Input = C::Unit;
    type Output = C::Value;
    type Error = TranscodeDecodeErrorOf<C>;

    /// Returns an upper bound for decoded values produced from `input_len`
    /// units.
    #[inline(always)]
    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        TranscodeDecodeEngine::max_transcode_output_len(self, input_len)
    }

    /// Returns an upper bound for values produced by finishing codec and hook
    /// state.
    #[inline(always)]
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        TranscodeDecodeEngine::max_finish_output_len(self)
    }

    /// Returns an upper bound for values emitted when resetting stream state.
    #[inline(always)]
    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        TranscodeDecodeEngine::max_reset_output_len(self)
    }

    /// Runs hook-owned cleanup before a logical decoder reset.
    #[inline(always)]
    fn reset(&mut self, output: &mut [C::Value], output_index: usize) -> Result<usize, TranscodeDecodeErrorOf<C>> {
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
    ) -> Result<TranscodeProgress, TranscodeDecodeErrorOf<C>> {
        TranscodeDecodeEngine::transcode(self, input, input_index, output, output_index)
    }

    /// Decodes source units after end of input is known.
    #[inline(always)]
    fn transcode_eof(
        &mut self,
        input: &[C::Unit],
        input_index: usize,
        output: &mut [C::Value],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeDecodeErrorOf<C>> {
        TranscodeDecodeEngine::transcode_eof(self, input, input_index, output, output_index)
    }

    /// Finishes internally retained output after EOF.
    #[inline(always)]
    fn finish(&mut self, output: &mut [C::Value], output_index: usize) -> Result<usize, TranscodeDecodeErrorOf<C>> {
        TranscodeDecodeEngine::finish(self, output, output_index)
    }
}

impl<C, H> TranscodeDecoder for TranscodeDecodeEngine<C, H>
where
    C: Codec,
    H: TranscodeDecodeHooks<C>,
{
    type DecodeError = C::DecodeError;
}
