// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Reusable buffered encoder engine.

use super::super::encode_context::EncodeContext;
use super::super::internal::{
    encode_state::EncodeState,
    lifecycle::LifecycleGuard,
};
use crate::codec::assert_unit_bounds;
use crate::{
    CapacityError,
    Codec,
    CodecEncodeFlushError,
    CodecEncodeResetError,
    EncodeOutcome,
    TranscodeEncodeHooks,
    TranscodeError,
    TranscodeProgress,
    Transcoder,
};

/// Reusable buffered encoding engine for codec-backed encoders.
///
/// The engine owns the low-level codec and hook object. It keeps the common
/// buffered encoding loop private: input-index validation, output-capacity
/// checks, input consumption, output progress, and [`crate::TranscodeStatus`]
/// reporting.
///
/// Use this type to build a streaming encoder over a one-value [`Codec`]. The
/// engine does not allocate output. It repeatedly asks hooks to process one
/// input value at the current output cursor. If the hook reports insufficient
/// output, the engine returns [`crate::TranscodeStatus::NeedOutput`] without
/// consuming that value; the caller can provide a larger or fresh output buffer
/// and resume with the returned input index.
///
/// For the common strict policy that simply wraps codec errors, use
/// [`crate::CodecTranscodeEncoder`]. Use `TranscodeEncodeEngine` directly when
/// the encode policy needs custom planning, replacement, skipped values, or
/// finish-time output.
///
/// # Example
///
/// ```rust
/// use core::{
///     convert::Infallible,
///     num::NonZeroUsize,
/// };
/// use qubit_codec::{
///     TranscodeEncodeEngine,
///     TranscodeEncodeHooks,
///     Codec,
///     CodecDecodeFailure,
///     CodecEncodeError,
///     EncodeContext,
///     EncodeOutcome,
///     TranscodeStatus,
/// };
///
/// #[derive(Clone, Copy)]
/// struct ByteCodec;
///
/// impl Codec for ByteCodec {
///     type Value = u8;
///     type Unit = u8;
///     type DecodeError = Infallible;
///     type EncodeError = Infallible;
///
///     const MIN_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;
///     const MAX_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;
///
///     unsafe fn decode(
///         &mut self,
///         input: &[u8],
///         index: usize,
///     ) -> Result<(u8, NonZeroUsize), CodecDecodeFailure<Self::DecodeError>> {
///         Ok((input[index], NonZeroUsize::MIN))
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
/// struct StrictHooks;
///
/// impl TranscodeEncodeHooks<ByteCodec> for StrictHooks {
///     type Error = CodecEncodeError<Infallible>;
///
///     fn encode_value(
///         &mut self,
///         codec: &mut ByteCodec,
///         context: EncodeContext<'_, u8, u8>,
///     ) -> Result<EncodeOutcome, Self::Error> {
///         let required = ByteCodec::MAX_UNITS_PER_VALUE;
///         if context.available_output() < required.get() {
///             return Ok(EncodeOutcome::need_output(required));
///         }
///         let (value, input_index, output, output_index) = context.into_parts();
///         let written = unsafe { codec.encode(value, output, output_index) }
///             .map(NonZeroUsize::get)
///             .map_err(|error| CodecEncodeError::encode(error, input_index))?;
///         Ok(EncodeOutcome::consumed(written))
///     }
/// }
///
/// let mut engine = TranscodeEncodeEngine::new(ByteCodec, StrictHooks);
/// let input = [1_u8, 2, 3];
/// let mut output = [0_u8; 2];
///
/// let progress = engine.transcode(&input, 0, &mut output, 0)?;
/// match progress.status() {
///     TranscodeStatus::Complete => unreachable!("output is intentionally short"),
///     TranscodeStatus::NeedOutput { output_index, .. } => {
///         assert_eq!(2, output_index);
///         assert_eq!([1, 2], output);
///         // Write out `output[..output_index]`, then resume at
///         // `progress.read()` with fresh output capacity.
///     }
///     TranscodeStatus::NeedInput { .. } => unreachable!("encoders do not read encoded input"),
/// }
/// # Ok::<(), qubit_codec::TranscodeError<CodecEncodeError<Infallible>>>(())
/// ```
///
/// # Type Parameters
///
/// - `C`: Low-level codec used by the engine.
/// - `H`: Policy hook object used by the engine.
#[derive(Debug)]
pub struct TranscodeEncodeEngine<C, H> {
    codec: C,
    hooks: H,
    /// Debug-only guard for the `reset → transcode* → finish` lifecycle.
    /// Zero-sized in release builds.
    lifecycle: LifecycleGuard,
}

impl<C, H> TranscodeEncodeEngine<C, H>
where
    C: Codec,
    H: TranscodeEncodeHooks<C>,
{
    /// Creates a buffered encoder engine.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec used for one-value encoding.
    /// - `hooks`: Policy hooks used for planning and writing values.
    ///
    /// # Returns
    ///
    /// Returns a buffered encoder engine.
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

    /// Encodes one value through the hook and codec.
    ///
    /// This is the single entry point for `TranscodeConvertEngine` to drive
    /// the encode side without accessing `codec` and `hooks` directly.
    ///
    /// # Parameters
    ///
    /// - `context`: Encode context for the current value.
    ///
    /// # Returns
    ///
    /// Returns whether the value was consumed or needs more output capacity.
    ///
    /// # Errors
    ///
    /// Returns `H::Error` when the hook rejects or cannot encode the value.
    #[inline(always)]
    pub(crate) fn encode_one(
        &mut self,
        context: EncodeContext<'_, C::Value, C::Unit>,
    ) -> Result<EncodeOutcome, H::Error> {
        self.hooks.encode_value(&mut self.codec, context)
    }

    /// Gets a conservative upper bound for output units needed for
    /// `input_len` values.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Number of input values the caller plans to encode.
    ///
    /// # Returns
    ///
    /// a conservative upper bound for output units, or a capacity error on
    /// arithmetic overflow.
    #[inline(always)]
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        self.hooks.max_output_len(&self.codec, input_len)
    }

    /// Gets the maximum output units emitted by stream reset.
    ///
    /// # Returns
    ///
    /// the codec's reset-output upper bound.
    #[inline(always)]
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        Ok(C::MAX_ENCODE_RESET_UNITS)
    }

    /// Gets the maximum output units emitted by finishing codec and hook state.
    ///
    /// Returns the sum of [`Codec::MAX_ENCODE_FLUSH_UNITS`] and the
    /// hook-provided final-output bound. The codec flush portion covers units
    /// written by [`Codec::encode_flush`]; hook implementations must not
    /// include that portion in
    /// [`TranscodeEncodeHooks::max_finish_output_len`].
    ///
    /// # Returns
    ///
    /// the combined codec-flush and hook-finish output bound.
    #[inline(always)]
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        C::MAX_ENCODE_FLUSH_UNITS
            .checked_add(self.hooks.max_finish_output_len(&self.codec))
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    /// Resets codec encode state, hook-owned state, and stream-start output.
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output unit slice visible to the encoder.
    /// - `output_index`: Absolute output unit index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of reset units written.
    ///
    /// # Errors
    ///
    /// Returns hook errors when the caller provides invalid or insufficient
    /// output capacity, or when reset output cannot be emitted. Codec reset
    /// errors are converted with [`From`].
    pub fn reset(
        &mut self,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<usize, TranscodeError<H::Error>>
    where
        H::Error: From<CodecEncodeResetError<C::EncodeError>>,
    {
        self.lifecycle.on_reset();
        let required = self.max_reset_output_len()?;
        TranscodeError::ensure_output_capacity(
            output.len(),
            output_index,
            required,
        )?;
        self.hooks.reset_hooks(&mut self.codec);
        let written = unsafe {
            // SAFETY: The capacity check above reserves the codec's declared
            // reset-output bound at `output_index`.
            self.codec.encode_reset(output, output_index)
        }
        .map_err(|error| {
            TranscodeError::domain(H::Error::from(CodecEncodeResetError::new(
                error,
            )))
        })?;
        assert!(
            written <= required,
            "Codec::encode_reset wrote beyond its reset bound",
        );
        Ok(written)
    }

    /// Encodes values into a caller-provided output buffer.
    ///
    /// The engine stops before consuming the next input value when the current
    /// output buffer does not satisfy that value's planned capacity bound.
    ///
    /// # Parameters
    ///
    /// - `input`: Complete input value slice visible to the encoder.
    /// - `input_index`: Absolute input value index where encoding starts.
    /// - `output`: Complete output unit slice visible to the encoder.
    /// - `output_index`: Absolute output unit index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns progress describing input values consumed, output units written,
    /// and why encoding stopped.
    ///
    /// # Errors
    ///
    /// Returns hook errors when `input_index` is outside `input`, when
    /// `output_index` is outside `output`, or when hook planning or writing
    /// rejects a value.
    pub fn transcode(
        &mut self,
        input: &[C::Value],
        input_index: usize,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<H::Error>> {
        self.lifecycle.on_transcode();
        TranscodeError::ensure_transcode_indices(
            input.len(),
            input_index,
            output.len(),
            output_index,
        )?;
        let mut state =
            EncodeState::new(input, input_index, output, output_index);

        while state.has_input() {
            // SAFETY: The loop condition proves that the current input cursor
            // points at an available value.
            let context = unsafe { state.context_unchecked() };
            let outcome = self
                .hooks
                .encode_value(&mut self.codec, context)
                .map_err(TranscodeError::domain)?;
            if let Some(progress) = state.apply_encode_outcome(outcome) {
                return Ok(progress);
            }
        }

        Ok(state.complete_progress())
    }

    /// Finishes codec and hook-owned output after EOF.
    ///
    /// Finalization first flushes encode-side codec state through
    /// [`Codec::encode_flush`], then lets hook implementations finish their
    /// own retained state. The caller must provide enough output capacity for
    /// [`TranscodeEncodeEngine::max_finish_output_len`], which includes both
    /// the codec flush bound and the hook-owned finish bound.
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output unit slice visible to the encoder.
    /// - `output_index`: Absolute output unit index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of units written by finalization.
    ///
    /// # Errors
    ///
    /// Returns hook errors when the caller provides invalid or insufficient
    /// output capacity, when codec flush errors are converted with [`From`],
    /// or when hook finalization fails.
    ///
    /// # Panics
    ///
    /// Panics when the codec flush writes beyond
    /// [`Codec::MAX_ENCODE_FLUSH_UNITS`] or when the combined codec and hook
    /// finalization writes beyond
    /// [`TranscodeEncodeEngine::max_finish_output_len`].
    pub fn finish(
        &mut self,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<usize, TranscodeError<H::Error>>
    where
        H::Error: From<CodecEncodeFlushError<C::EncodeError>>,
    {
        self.lifecycle.on_finish_attempt();
        let required = self.max_finish_output_len()?;
        TranscodeError::ensure_output_capacity(
            output.len(),
            output_index,
            required,
        )?;
        let flushed = unsafe {
            // SAFETY: The capacity check above reserves the codec's declared
            // flush-output bound at `output_index`.
            self.codec.encode_flush(output, output_index)
        }
        .map_err(|error| {
            TranscodeError::domain(H::Error::from(CodecEncodeFlushError::new(error)))
        })?;
        assert!(
            flushed <= C::MAX_ENCODE_FLUSH_UNITS,
            "Codec::encode_flush wrote beyond its flush bound",
        );
        let written = self
            .hooks
            .finish_hooks(&mut self.codec, output, output_index + flushed)
            .map_err(TranscodeError::domain)?;
        assert!(
            flushed + written <= required,
            "TranscodeEncodeEngine hook wrote beyond its finish bound",
        );
        self.lifecycle.on_finish_success();
        Ok(flushed + written)
    }
}

impl<C, H> Default for TranscodeEncodeEngine<C, H>
where
    C: Codec + Default,
    H: TranscodeEncodeHooks<C> + Default,
{
    /// Creates a default buffered encoder engine.
    ///
    /// # Returns
    ///
    /// Returns an engine with default codec and hooks.
    #[inline(always)]
    fn default() -> Self {
        Self::new(C::default(), H::default())
    }
}

impl<C, H> Transcoder<C::Value, C::Unit> for TranscodeEncodeEngine<C, H>
where
    C: Codec,
    H: TranscodeEncodeHooks<C>,
    H::Error: From<CodecEncodeResetError<C::EncodeError>>,
    H::Error: From<CodecEncodeFlushError<C::EncodeError>>,
{
    type Error = H::Error;

    /// Returns an upper bound for units produced from `input_len` values.
    #[inline(always)]
    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        TranscodeEncodeEngine::max_output_len(self, input_len)
    }

    /// Returns the maximum units emitted when resetting stream state.
    #[inline(always)]
    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        TranscodeEncodeEngine::max_reset_output_len(self)
    }

    /// Returns the maximum units emitted by finishing internal state.
    #[inline(always)]
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        TranscodeEncodeEngine::max_finish_output_len(self)
    }

    /// Resets codec encode state, hook-owned state, and stream-start output.
    #[inline(always)]
    fn reset(
        &mut self,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        TranscodeEncodeEngine::reset(self, output, output_index)
    }

    /// Encodes input values into caller-provided output units.
    #[inline(always)]
    fn transcode(
        &mut self,
        input: &[C::Value],
        input_index: usize,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
        TranscodeEncodeEngine::transcode(
            self,
            input,
            input_index,
            output,
            output_index,
        )
    }

    /// Finishes hook-owned encoder state.
    #[inline(always)]
    fn finish(
        &mut self,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        TranscodeEncodeEngine::finish(self, output, output_index)
    }
}
