// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Reusable buffered encoder engine.

use core::num::NonZeroUsize;

use super::super::internal::{
    encode_state::EncodeState,
    lifecycle::LifecycleGuard,
};
use super::encode_context::EncodeContext;
use crate::codec::assert_unit_bounds;
use crate::{
    CapacityError,
    Codec,
    EncodeOutcome,
    EncodeUnencodableAction,
    TranscodeEncodeEngineError,
    TranscodeEncodeHooks,
    TranscodeError,
    TranscodeProgress,
    Transcoder,
};

type EncodeEngineErrorOf<C, H> = TranscodeEncodeEngineError<
    <C as Codec>::EncodeError,
    <H as TranscodeEncodeHooks<C>>::Error,
>;

/// Reusable buffered encoding engine for codec-backed encoders.
///
/// The engine owns the low-level codec and hook object. It keeps the common
/// buffered encoding loop private: input-index validation, output-capacity
/// checks, input consumption, output progress, and [`crate::TranscodeStatus`]
/// reporting.
///
/// Use this type to build a streaming encoder over a one-value [`Codec`]. The
/// engine does not allocate output. It encodes values accepted by
/// [`Codec::can_encode_value`] directly through the codec. When a value is
/// outside the codec's encodable domain, the engine asks hooks whether to
/// reject, skip, or replace it. If the current output buffer is too small for
/// the selected value, the engine returns
/// [`crate::TranscodeStatus::NeedOutput`] without consuming that input; the
/// caller can provide a larger or fresh output buffer and resume with the
/// returned input index.
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
///     DecodeFailure,
///     CodecEncodeError,
///     EncodeUnencodableAction,
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
///     ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
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
///     fn handle_unencodable_encode(
///         &mut self,
///         _codec: &mut ByteCodec,
///         _value: &u8,
///         _input_index: usize,
///     ) -> Result<EncodeUnencodableAction<u8>, Self::Error> {
///         unreachable!("ByteCodec accepts every u8")
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
/// # Ok::<(), qubit_codec::TranscodeError<qubit_codec::TranscodeEncodeEngineError<Infallible, CodecEncodeError<Infallible>>>>(())
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

    /// Returns the encode hooks used by this engine.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the hook object owned by this engine.
    #[inline(always)]
    #[must_use]
    pub const fn hooks(&self) -> &H {
        &self.hooks
    }

    /// Returns the encode hooks mutably.
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
    /// Returns the wrapped codec followed by the encode hooks.
    #[inline]
    #[must_use]
    pub fn into_parts(self) -> (C, H) {
        let Self { codec, hooks, .. } = self;
        (codec, hooks)
    }

    /// Encodes one value through the codec and unencodable-value hooks.
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
    /// Returns an engine-domain error when the codec fails or when the hook
    /// rejects an unencodable value.
    pub(crate) fn encode_one(
        &mut self,
        context: EncodeContext<'_, C::Value, C::Unit>,
    ) -> Result<EncodeOutcome, EncodeEngineErrorOf<C, H>> {
        if self.codec.can_encode_value(context.input_value()) {
            return self.encode_encodable_value(context);
        }
        let input_index = context.input_index();
        let action = self
            .hooks
            .handle_unencodable_encode(
                &mut self.codec,
                context.input_value(),
                input_index,
            )
            .map_err(TranscodeEncodeEngineError::hook)?;
        self.apply_unencodable_action(action, context)
    }

    /// Encodes an encodable input value.
    ///
    /// # Parameters
    ///
    /// - `context`: Encode context for the current value.
    ///
    /// # Returns
    ///
    /// Returns the encode outcome for the current value.
    ///
    /// # Errors
    ///
    /// Returns a codec error when encoding fails.
    fn encode_encodable_value(
        &mut self,
        context: EncodeContext<'_, C::Value, C::Unit>,
    ) -> Result<EncodeOutcome, EncodeEngineErrorOf<C, H>> {
        let required =
            Self::encoded_value_len(&self.codec, context.input_value());
        if context.available_output() < required.get() {
            return Ok(EncodeOutcome::need_output(required));
        }
        let (value, input_index, output, output_index) = context.into_parts();
        let written = unsafe {
            // SAFETY: The capacity check above reserves the exact value width.
            self.codec.encode(value, output, output_index)
        }
        .map_err(|error| {
            TranscodeEncodeEngineError::codec_encode(error, input_index)
        })?;
        assert!(
            written == required,
            "Codec::encode wrote a different length than Codec::encode_len",
        );
        Ok(EncodeOutcome::consumed(written.get()))
    }

    /// Applies the hook-selected unencodable-value action.
    ///
    /// # Parameters
    ///
    /// - `action`: Policy action selected by the encode hooks.
    /// - `context`: Encode context for the rejected input value.
    ///
    /// # Returns
    ///
    /// Returns the encode outcome for the current input value.
    ///
    /// # Errors
    ///
    /// Returns a codec error when replacement encoding fails.
    #[inline(always)]
    fn apply_unencodable_action(
        &mut self,
        action: EncodeUnencodableAction<C::Value>,
        context: EncodeContext<'_, C::Value, C::Unit>,
    ) -> Result<EncodeOutcome, EncodeEngineErrorOf<C, H>> {
        match action {
            EncodeUnencodableAction::Skip => Ok(EncodeOutcome::consumed(0)),
            EncodeUnencodableAction::Replace { value } => {
                self.encode_replacement_value(value, context)
            }
        }
    }

    /// Encodes a hook-provided replacement value.
    ///
    /// # Parameters
    ///
    /// - `value`: Replacement value selected by hooks.
    /// - `context`: Encode context for the original unencodable input value.
    ///
    /// # Returns
    ///
    /// Returns the encode outcome for the replacement.
    ///
    /// # Errors
    ///
    /// Returns a codec error when replacement encoding fails.
    ///
    /// # Panics
    ///
    /// Panics when hooks return a replacement value that the codec cannot
    /// encode.
    fn encode_replacement_value(
        &mut self,
        value: C::Value,
        context: EncodeContext<'_, C::Value, C::Unit>,
    ) -> Result<EncodeOutcome, EncodeEngineErrorOf<C, H>> {
        assert!(
            self.codec.can_encode_value(&value),
            "EncodeUnencodableAction::Replace returned an unencodable replacement value",
        );
        let required = Self::encoded_value_len(&self.codec, &value);
        if context.available_output() < required.get() {
            return Ok(EncodeOutcome::need_output(required));
        }
        let (_, input_index, output, output_index) = context.into_parts();
        let written = unsafe {
            // SAFETY: The capacity check above reserves the exact replacement
            // value width, and the hook contract requires encodability.
            self.codec.encode(&value, output, output_index)
        }
        .map_err(|error| {
            TranscodeEncodeEngineError::codec_encode(error, input_index)
        })?;
        assert!(
            written == required,
            "Codec::encode wrote a different length than Codec::encode_len",
        );
        Ok(EncodeOutcome::consumed(written.get()))
    }

    /// Returns the encoded length for `value`.
    ///
    /// # Parameters
    ///
    /// - `codec`: Codec used for variable-width length queries.
    /// - `value`: Encodable value whose output width is requested.
    ///
    /// # Returns
    ///
    /// Returns the exact encoded width for `value`.
    #[inline(always)]
    fn encoded_value_len(codec: &C, value: &C::Value) -> NonZeroUsize {
        codec.encode_len(value)
    }

    /// Gets a conservative upper bound for output units needed for
    /// `input_len` values.
    ///
    /// This bound covers only the streaming encode phase. It is delegated to
    /// [`TranscodeEncodeHooks::max_transcode_output_len`], so it includes hook
    /// policy. Downstream encoders must use this engine-level API for capacity
    /// planning instead of recomputing the bound from [`Codec`] constants.
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
    pub fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        self.hooks.max_transcode_output_len(&self.codec, input_len)
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

    /// Gets the maximum output units needed by a complete one-shot encode
    /// stream.
    ///
    /// The returned bound covers reset output, the streaming encode phase for
    /// `input_len` values, and finish output. Higher-level complete encode
    /// helpers should use this engine-level bound instead of recomputing
    /// capacity from [`Codec`] constants, because hook policy may change
    /// streaming or finish output.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Number of input values in the complete stream.
    ///
    /// # Returns
    ///
    /// Returns the complete-stream output bound, or a capacity error on
    /// arithmetic overflow.
    #[inline]
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_total_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        let reset = self.max_reset_output_len()?;
        let transcode = self.max_transcode_output_len(input_len)?;
        let finish = self.max_finish_output_len()?;
        reset
            .checked_add(transcode)
            .and_then(|len| len.checked_add(finish))
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
    /// Returns framework errors when the caller provides invalid or
    /// insufficient output capacity. Returns domain errors when codec reset or
    /// hook reset handling fails.
    pub fn reset(
        &mut self,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<usize, TranscodeError<EncodeEngineErrorOf<C, H>>> {
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
            TranscodeError::domain(TranscodeEncodeEngineError::codec_reset(
                error,
            ))
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
    ) -> Result<TranscodeProgress, TranscodeError<EncodeEngineErrorOf<C, H>>>
    {
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
            let outcome =
                self.encode_one(context).map_err(TranscodeError::domain)?;
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
    /// Returns framework errors when the caller provides invalid or
    /// insufficient output capacity. Returns domain errors when codec flush or
    /// hook finalization fails.
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
    ) -> Result<usize, TranscodeError<EncodeEngineErrorOf<C, H>>> {
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
            TranscodeError::domain(TranscodeEncodeEngineError::codec_flush(
                error,
            ))
        })?;
        assert!(
            flushed <= C::MAX_ENCODE_FLUSH_UNITS,
            "Codec::encode_flush wrote beyond its flush bound",
        );
        let written = self
            .hooks
            .finish_hooks(&mut self.codec, output, output_index + flushed)
            .map_err(|error| {
                TranscodeError::domain(TranscodeEncodeEngineError::hook(error))
            })?;
        assert!(
            flushed + written <= required,
            "TranscodeEncodeEngine hook wrote beyond its finish bound",
        );
        self.lifecycle.on_finish_success();
        Ok(flushed + written)
    }

    /// Runs a complete one-shot `reset -> transcode -> finish` encode stream.
    ///
    /// The complete input is supplied as `input`, and output starts at index
    /// `0` in `output`. Callers that need subranges should slice their
    /// buffers before calling this method. Downstream one-shot encoder
    /// helpers should call this engine method instead of reproducing the
    /// reset, transcode, and finish sequence themselves.
    ///
    /// # Parameters
    ///
    /// - `input`: Complete input values.
    /// - `output`: Output unit slice for the whole encoded stream.
    ///
    /// # Returns
    ///
    /// Returns the number of output units written.
    ///
    /// # Errors
    ///
    /// Returns framework errors for insufficient output or capacity overflow,
    /// and domain errors from reset, encode, or finish.
    #[inline]
    pub fn transcode_complete_into(
        &mut self,
        input: &[C::Value],
        output: &mut [C::Unit],
    ) -> Result<usize, TranscodeError<EncodeEngineErrorOf<C, H>>> {
        <Self as Transcoder<C::Value, C::Unit>>::transcode_complete_into(
            self, input, output,
        )
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
{
    type Error = EncodeEngineErrorOf<C, H>;

    /// Returns an upper bound for units produced from `input_len` values.
    #[inline(always)]
    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        TranscodeEncodeEngine::max_transcode_output_len(self, input_len)
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
