// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Reusable buffered converter engine for codec-backed transcoding.
//!
//! Bridges a source [`crate::TranscodeDecodeEngine`] and a target
//! [`crate::TranscodeEncodeEngine`] into one unit-to-unit conversion pipeline.

use super::super::internal::{
    convert_error_of::ConvertErrorOf,
    convert_state::ConvertState,
    lifecycle::LifecycleGuard,
    pending_value::PendingValue,
    pending_value_slot::PendingValueSlot,
};
use super::{
    transcode_decode_engine::TranscodeDecodeEngine,
    transcode_encode_engine::TranscodeEncodeEngine,
};
use crate::codec::assert_unit_bounds;
use crate::{
    CapacityError,
    Codec,
    EncodeContext,
    TranscodeConvertEngineError,
    TranscodeDecodeEngineError,
    TranscodeDecodeHooks,
    TranscodeEncodeEngineError,
    TranscodeEncodeHooks,
    TranscodeError,
    TranscodeProgress,
    Transcoder,
};

/// Reusable buffered conversion engine for codec-backed converters.
///
/// The engine owns reusable buffered decode and encode engines. It keeps
/// common converter control flow private: index validation, pending-value
/// retention, pending flush, decode-error policy dispatch, encode attempts,
/// output-capacity checks, and [`crate::TranscodeStatus`] reporting.
///
/// Use this type to build a streaming converter over two one-value [`Codec`]
/// implementations that share the same logical value type. Each hot-path step
/// decodes one source unit sequence into a value, then immediately tries to
/// encode that value into the target output buffer. If the target buffer lacks
/// capacity, the decoded value is retained in an internal pending slot and
/// must be drained before more source input is consumed, preserving output
/// order across buffer turns.
///
/// `TranscodeConvertEngine` is intentionally batch-oriented. Its public
/// [`Self::transcode`] method drives a source/output buffer loop and reuses the
/// same unchecked codec and hook primitives as [`crate::TranscodeDecodeEngine`]
/// and [`crate::TranscodeEncodeEngine`]. It does not call one-value public
/// transcoders in the hot path.
///
/// For strict codec-backed conversion with default decode and encode policies,
/// use [`crate::CodecTranscodeConverter`]. Use `TranscodeConvertEngine`
/// directly when either side needs custom malformed-input repair, encode
/// planning, skipped values, or finish-time output.
///
/// The engine follows the same lifecycle as [`crate::Transcoder`]:
/// `reset → transcode* → finish → reset`. Call [`Self::reset`] before starting
/// a new logical stream and [`Self::finish`] after EOF once any incomplete
/// source tail has been handled.
///
/// # Example
///
/// ```rust
/// use core::{
///     convert::Infallible,
///     num::NonZeroUsize,
/// };
/// use qubit_codec::{
///     Codec,
///     DecodeContext,
///     DecodeFailure,
///     EncodeUnencodableAction,
///     TranscodeConvertEngine,
///     TranscodeDecodeHooks,
///     TranscodeEncodeHooks,
///     TranscodeStatus,
/// };
///
/// #[derive(Clone, Copy)]
/// struct SourceCodec;
///
/// #[derive(Clone, Copy)]
/// struct TargetCodec;
///
/// impl Codec for SourceCodec {
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
///         Ok((input[index].wrapping_add(1), NonZeroUsize::MIN))
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
/// impl Codec for TargetCodec {
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
/// struct StrictDecodeHooks;
///
/// impl TranscodeDecodeHooks<SourceCodec> for StrictDecodeHooks {
///     type Error = Infallible;
///
///     fn handle_invalid_decode(
///         &mut self,
///         _codec: &mut SourceCodec,
///         error: Infallible,
///         _consumed: Option<NonZeroUsize>,
///         _context: DecodeContext,
///     ) -> Result<qubit_codec::DecodeInvalidAction<u8>, Self::Error> {
///         match error {}
///     }
/// }
///
/// struct StrictEncodeHooks;
///
/// impl TranscodeEncodeHooks<TargetCodec> for StrictEncodeHooks {
///     type Error = Infallible;
///
///     fn handle_unencodable_encode(
///         &mut self,
///         _codec: &mut TargetCodec,
///         _value: &u8,
///         _input_index: usize,
///     ) -> Result<EncodeUnencodableAction<u8>, Self::Error> {
///         unreachable!("TargetCodec accepts every u8")
///     }
/// }
///
/// let mut engine = TranscodeConvertEngine::new(
///     SourceCodec,
///     TargetCodec,
///     StrictDecodeHooks,
///     StrictEncodeHooks,
/// );
/// let input = [1_u8, 2, 3];
/// let mut output = [0_u8; 2];
///
/// let progress = engine.transcode(&input, 0, &mut output, 0)?;
/// match progress.status() {
///     TranscodeStatus::NeedOutput { output_index, .. } => {
///         assert_eq!(2, output_index);
///         assert_eq!([2, 3], output);
///         // Drain `output[..output_index]`, then resume at
///         // `progress.read()` with fresh output capacity.
///     }
///     TranscodeStatus::Complete => unreachable!("output is intentionally short"),
///     TranscodeStatus::NeedInput { .. } => unreachable!("input is complete"),
/// }
/// # Ok::<(), qubit_codec::TranscodeError<qubit_codec::TranscodeConvertEngineError<
/// #     qubit_codec::TranscodeDecodeEngineError<Infallible, Infallible>,
/// #     qubit_codec::TranscodeEncodeEngineError<Infallible, Infallible>,
/// # >>>(())
/// ```
///
/// # Type Parameters
///
/// - `D`: Source-side decoder codec.
/// - `E`: Target-side encoder codec.
/// - `DH`: Source-side decode hooks.
/// - `EH`: Target-side encode hooks.
#[derive(Debug)]
pub struct TranscodeConvertEngine<D, E, DH, EH>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    DH: TranscodeDecodeHooks<D>,
    EH: TranscodeEncodeHooks<E>,
{
    /// Source-side buffered decoder engine.
    decode_engine: TranscodeDecodeEngine<D, DH>,
    /// Target-side buffered encoder engine.
    encode_engine: TranscodeEncodeEngine<E, EH>,
    /// Decoded value waiting for target output capacity.
    pending: PendingValueSlot<D::Value>,
    /// Debug-only guard for the `reset → transcode* → finish` lifecycle.
    /// Zero-sized in release builds. The converter owns its own guard rather
    /// than delegating to the inner decode/encode engines, because lifecycle
    /// events here describe the converter as a whole.
    lifecycle: LifecycleGuard,
}

impl<D, E, DH, EH> TranscodeConvertEngine<D, E, DH, EH>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    DH: TranscodeDecodeHooks<D>,
    EH: TranscodeEncodeHooks<E>,
{
    /// Creates a buffered converter engine.
    ///
    /// The caller supplies decode hooks and encode hooks directly.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Low-level codec used for source decoding.
    /// - `encoder`: Low-level codec used for target encoding.
    /// - `decode_hooks`: Decode-side policy hooks.
    /// - `encode_hooks`: Encode-side policy hooks.
    ///
    /// # Returns
    ///
    /// Returns a buffered converter engine.
    ///
    /// # Panics
    ///
    /// In debug builds, panics when either codec violates the
    /// [`Codec::MIN_UNITS_PER_VALUE`] / [`Codec::MAX_UNITS_PER_VALUE`] ordering
    /// invariant. Release builds skip this check because the invariant is the
    /// responsibility of each [`Codec`] implementation.
    #[inline]
    #[must_use]
    pub fn new(
        decoder: D,
        encoder: E,
        decode_hooks: DH,
        encode_hooks: EH,
    ) -> Self {
        assert_unit_bounds::<D>();
        assert_unit_bounds::<E>();
        Self {
            decode_engine: TranscodeDecodeEngine::new(decoder, decode_hooks),
            encode_engine: TranscodeEncodeEngine::new(encoder, encode_hooks),
            pending: PendingValueSlot::empty(),
            lifecycle: LifecycleGuard::new(),
        }
    }

    /// Returns the source-side codec.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the decoder codec owned by this engine.
    #[inline(always)]
    #[must_use]
    pub fn source_codec(&self) -> &D {
        self.decode_engine.codec()
    }

    /// Returns the source-side codec mutably.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the decoder codec owned by this engine.
    #[inline(always)]
    #[must_use]
    pub fn source_codec_mut(&mut self) -> &mut D {
        self.decode_engine.codec_mut()
    }

    /// Returns the target-side codec.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the encoder codec owned by this engine.
    #[inline(always)]
    #[must_use]
    pub fn target_codec(&self) -> &E {
        self.encode_engine.codec()
    }

    /// Returns the target-side codec mutably.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the encoder codec owned by this engine.
    #[inline(always)]
    #[must_use]
    pub fn target_codec_mut(&mut self) -> &mut E {
        self.encode_engine.codec_mut()
    }

    /// Consumes the engine and returns its codecs and hooks.
    ///
    /// Any pending value and lifecycle state owned by the converter are
    /// discarded. Callers should use this only when no further conversion state
    /// needs to be preserved.
    ///
    /// # Returns
    ///
    /// Returns the source codec, target codec, decode hooks, and encode hooks.
    #[inline]
    #[must_use]
    pub fn into_parts(self) -> (D, E, DH, EH) {
        let Self {
            decode_engine,
            encode_engine,
            ..
        } = self;
        let (source, decode_hooks) = decode_engine.into_parts();
        let (target, encode_hooks) = encode_engine.into_parts();
        (source, target, decode_hooks, encode_hooks)
    }

    /// Returns an upper bound for target units produced from `input_len` units.
    ///
    /// The bound sums three parts: any retained pending value, the maximum
    /// decoded values from the source side, and the maximum target units for
    /// those values on the encode side. It covers only the streaming convert
    /// phase. Downstream converters must use this engine-level API for capacity
    /// planning instead of recomputing the bound from the source or target
    /// [`Codec`] constants.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Number of source units the caller plans to convert.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound, or a capacity error on arithmetic
    /// overflow.
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        let pending_units = self.pending_output_len()?;
        let decoded_values =
            self.decode_engine.max_transcode_output_len(input_len)?;
        let converted_units = self
            .encode_engine
            .max_transcode_output_len(decoded_values)?;
        converted_units
            .checked_add(pending_units)
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    /// Returns the maximum target units emitted when resetting stream state.
    ///
    /// Covers decode-side reset values (encoded to target units) plus
    /// encode-side reset units. Most codecs are stateless and return `0`
    /// for [`Codec::MAX_DECODE_RESET_VALUES`]; in that case this equals the
    /// encode reset bound only.
    ///
    /// # Returns
    ///
    /// Returns the combined decode-reset and encode-reset output bound, or a
    /// capacity error on arithmetic overflow.
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        let decode_reset_units = self
            .encode_engine
            .max_transcode_output_len(D::MAX_DECODE_RESET_VALUES)?;
        let encode_reset_units = E::MAX_ENCODE_RESET_UNITS;
        decode_reset_units
            .checked_add(encode_reset_units)
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    /// Returns the maximum target units emitted by finishing retained state.
    ///
    /// The bound covers a retained pending value, decode-side finish values
    /// (encoded to target units), and encode-side finish units.
    ///
    /// # Returns
    ///
    /// Returns the combined pending, decode-finish, and encode-finish output
    /// bound, or a capacity error on arithmetic overflow.
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        let pending_units = self.pending_output_len()?;
        let decoder_finish_values =
            self.decode_engine.max_finish_output_len()?;
        let decoder_finish_units = self
            .encode_engine
            .max_transcode_output_len(decoder_finish_values)?;
        let encoder_finish_units =
            self.encode_engine.max_finish_output_len()?;
        let pending_and_decoder = pending_units
            .checked_add(decoder_finish_units)
            .ok_or(CapacityError::OutputLengthOverflow)?;
        pending_and_decoder
            .checked_add(encoder_finish_units)
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    /// Returns the maximum target units needed by a complete one-shot
    /// conversion.
    ///
    /// The returned bound covers conversion reset output, the streaming convert
    /// phase for `input_len` source units, and finish output. Higher-level
    /// complete conversion helpers should use this engine-level bound instead
    /// of recomputing capacity from the source or target codec constants,
    /// because decode and encode hooks may change streaming or finish
    /// output.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Number of source units in the complete stream.
    ///
    /// # Returns
    ///
    /// Returns the complete-stream target-output bound, or a capacity error on
    /// arithmetic overflow.
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

    /// Clears retained conversion state, runs before-reset hooks, and emits
    /// stream-start encode output.
    ///
    /// Reset clears any retained pending value, drains decode-side reset
    /// values through the target encoder, then emits encode-side reset units.
    /// The caller must provide enough output capacity for
    /// [`Self::max_reset_output_len`].
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output unit slice visible to the converter.
    /// - `output_index`: Absolute output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of target units written while resetting stream state.
    ///
    /// # Errors
    ///
    /// Returns a converter error if reset validation or target reset output
    /// emission fails.
    ///
    /// # Panics
    ///
    /// Panics in debug builds when decode-reset values cannot be encoded
    /// within the capacity reserved by [`Self::max_reset_output_len`].
    pub fn reset(
        &mut self,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<usize, ConvertErrorOf<D, E, DH, EH>>
    where
        D::Value: Default,
    {
        self.lifecycle.on_reset();
        let required = self.max_reset_output_len()?;
        TranscodeError::ensure_output_capacity(
            output.len(),
            output_index,
            required,
        )?;

        self.pending.clear();

        // Source-side reset may emit stream-start values (such as a BOM) that
        // must be piped through the target encoder before any encoder-owned
        // reset output. `max_reset_output_len` already reserves space for both
        // halves of the pipeline, so encode_pending should never report
        // `NeedOutput` here.
        let empty_input: &[D::Unit] = &[];
        let mut state = ConvertState::new(empty_input, 0, output, output_index);
        self.drain_decoder_reset(&mut state)?;

        let output_cursor = state.output_cursor();
        let encoder_written = self
            .encode_engine
            .reset(state.output_mut(), output_cursor)
            .map_err(|error| {
                error.map_domain(TranscodeConvertEngineError::encode)
            })?;
        state.advance_output(encoder_written);
        Ok(state.written())
    }

    /// Converts source units into target units.
    ///
    /// The engine drains any retained pending value before consuming new input.
    /// Each loop iteration decodes one source value and immediately attempts to
    /// encode it. Conversion stops when the input tail is incomplete, when the
    /// output buffer cannot hold the next encoded value, or when the visible
    /// input is exhausted.
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
    /// Returns conversion progress describing input units consumed, target
    /// units written, and why conversion stopped.
    ///
    /// # Errors
    ///
    /// Returns hook errors when indices are invalid or concrete conversion
    /// fails. Invalid output indices are reported through the encode-side
    /// error path.
    pub fn transcode(
        &mut self,
        input: &[D::Unit],
        input_index: usize,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, ConvertErrorOf<D, E, DH, EH>> {
        self.lifecycle.on_transcode();
        TranscodeError::ensure_transcode_indices(
            input.len(),
            input_index,
            output.len(),
            output_index,
        )?;

        let mut state =
            ConvertState::new(input, input_index, output, output_index);

        // A retained decoded value must be written before consuming more input,
        // otherwise callers could observe output reordered across buffer turns.
        if let Some(progress) = self.drain_pending(&mut state)? {
            return Ok(progress);
        }

        let min_input_units = D::MIN_UNITS_PER_VALUE;
        while state.has_input() {
            let available = state.available_input();
            if available < min_input_units.get() {
                return Ok(
                    state.need_input_progress(min_input_units, available)
                );
            }

            let previous_read = state.read();
            // Each hot-path step decodes one source value and immediately tries
            // to encode it, preserving backpressure at the target output.
            if let Some(progress) = self.convert_next(&mut state)? {
                return Ok(progress);
            }
            debug_assert!(
                state.read() > previous_read,
                "TranscodeConvertEngine conversion step must consume input or stop",
            );
        }

        Ok(state.complete_progress())
    }

    /// Finishes retained output after EOF.
    ///
    /// Finalization drains a pending decoded value first, then lets the
    /// source-side decode hooks emit final values, encodes those values through
    /// the target-side encode hooks, and finally finishes target-side encode
    /// hook state. The decode-finish value buffer used for this cold path
    /// requires `D::Value: Default`; the normal [`Self::transcode`] loop does
    /// not.
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output unit slice visible to the converter.
    /// - `output_index`: Absolute output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of target units written during finalization.
    ///
    /// # Errors
    ///
    /// Returns a converter error when output capacity checks fail or when
    /// hook finalization fails.
    ///
    /// # Panics
    ///
    /// Panics in debug builds when a retained pending value or decode-finish
    /// value cannot be encoded within the capacity reserved by
    /// [`Self::max_finish_output_len`].
    pub fn finish(
        &mut self,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<usize, ConvertErrorOf<D, E, DH, EH>>
    where
        D::Value: Default,
    {
        self.lifecycle.on_finish_attempt();
        let required = self.max_finish_output_len()?;
        TranscodeError::ensure_output_capacity(
            output.len(),
            output_index,
            required,
        )?;

        let empty_input: &[D::Unit] = &[];
        let mut state = ConvertState::new(empty_input, 0, output, output_index);
        // Finish keeps the same priority as transcode: output any retained
        // decoded value before asking source-side hooks for final values.
        if self.drain_pending(&mut state)?.is_some() {
            unreachable!(
                "converter finish bound must reserve space for pending values"
            );
        }

        // Source-side finish may emit one or more final values. Drain them into
        // the target encoder before finishing target-side hook state.
        self.drain_decoder_finish(&mut state)?;

        let output_cursor = state.output_cursor();
        let written = self
            .encode_engine
            .finish(state.output_mut(), output_cursor)
            .map_err(|error| {
                error.map_domain(TranscodeConvertEngineError::encode)
            })?;
        state.advance_output(written);
        self.lifecycle.on_finish_success();
        Ok(state.written())
    }

    /// Runs a complete one-shot `reset -> transcode -> finish` conversion.
    ///
    /// The complete input is supplied as `input`, and output starts at index
    /// `0` in `output`. Callers that need subranges should slice their
    /// buffers before calling this method. Downstream one-shot converter
    /// helpers should call this engine method instead of reproducing the
    /// reset, transcode, and finish sequence themselves.
    ///
    /// # Parameters
    ///
    /// - `input`: Complete source unit slice.
    /// - `output`: Target unit slice for the whole converted stream.
    ///
    /// # Returns
    ///
    /// Returns the number of target units written.
    ///
    /// # Errors
    ///
    /// Returns framework errors for insufficient output, capacity overflow, or
    /// an incomplete EOF tail, and domain errors from reset, conversion, or
    /// finish.
    #[inline]
    pub fn transcode_all_into(
        &mut self,
        input: &[D::Unit],
        output: &mut [E::Unit],
    ) -> Result<usize, ConvertErrorOf<D, E, DH, EH>>
    where
        D::Value: Default,
    {
        <Self as Transcoder<D::Unit, E::Unit>>::transcode_all_into(
            self, input, output,
        )
    }

    /// Drains source-side decode reset output and encodes emitted reset
    /// values.
    ///
    /// Stateless decoders still call [`TranscodeDecodeEngine::reset`] so hook
    /// teardown side effects run even when no reset values are emitted.
    ///
    /// # Parameters
    ///
    /// - `state`: Current conversion cursors and output buffer.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after all decode-reset values have been encoded.
    ///
    /// # Errors
    ///
    /// Returns a converter error when decode reset or encode reset handling
    /// fails.
    ///
    /// # Panics
    ///
    /// Panics when a decode-reset value cannot be encoded within the capacity
    /// reserved by [`Self::max_reset_output_len`].
    fn drain_decoder_reset(
        &mut self,
        state: &mut ConvertState<'_, D::Unit, E::Unit>,
    ) -> Result<(), ConvertErrorOf<D, E, DH, EH>>
    where
        D::Value: Default,
    {
        let value_count = D::MAX_DECODE_RESET_VALUES;
        if value_count == 0 {
            // Stateless decoder: still call decode_reset so codecs whose
            // hooks own teardown side effects (e.g. clearing accumulators)
            // run them. The empty slice is safe because the capacity check
            // inside `reset` accepts `required == 0` against any slice.
            self.decode_engine.reset(&mut [], 0).map_err(|error| {
                error.map_domain(TranscodeConvertEngineError::decode)
            })?;
            return Ok(());
        }
        // `D::Value: Default` is only consulted when the decoder declares
        // reset output. Stateless codecs never reach this branch.
        let mut reset_values: Vec<D::Value> =
            (0..value_count).map(|_| D::Value::default()).collect();
        let written = self.decode_engine.reset(&mut reset_values, 0).map_err(
            |error| error.map_domain(TranscodeConvertEngineError::decode),
        )?;
        for value in reset_values.into_iter().take(written) {
            let pending = PendingValue::new(value, 0);
            if self.encode_pending(pending, state)?.is_some() {
                unreachable!(
                    "converter reset bound must reserve space for decode reset values"
                );
            }
        }
        Ok(())
    }

    /// Converts one source value from the current state cursors.
    ///
    /// Decodes one value through the source engine, then immediately attempts
    /// to encode it through the target engine.
    ///
    /// # Parameters
    ///
    /// - `state`: Current conversion cursors and output buffer.
    ///
    /// # Returns
    ///
    /// Returns conversion progress when the step stops early, or `None` when
    /// the value was fully consumed and encoded.
    ///
    /// # Errors
    ///
    /// Returns a converter error when decode or encode handling fails.
    #[inline(always)]
    fn convert_next(
        &mut self,
        state: &mut ConvertState<'_, D::Unit, E::Unit>,
    ) -> Result<Option<TranscodeProgress>, ConvertErrorOf<D, E, DH, EH>> {
        let (outcome, pending) = self
            .decode_engine
            .decode_one(
                state.input(),
                state.decode_context(),
                PendingValue::new,
            )
            .map_err(|error| {
                TranscodeError::domain(TranscodeConvertEngineError::decode(
                    error,
                ))
            })?;
        if let Some(pending) = pending {
            self.pending.put(pending);
        }
        if let Some(progress) = state.apply_decode_outcome(outcome) {
            return Ok(Some(progress));
        }
        let Some(pending) = self.pending.take() else {
            return Ok(None);
        };
        self.encode_pending(pending, state)
    }

    /// Returns the output bound for the retained pending value.
    ///
    /// # Returns
    ///
    /// Returns the maximum target units needed to encode the pending value,
    /// or `0` when no value is retained. Returns a capacity error when hook
    /// planning overflows.
    #[inline(always)]
    fn pending_output_len(&self) -> Result<usize, CapacityError> {
        self.pending.max_transcode_output_len(&self.encode_engine)
    }

    /// Writes a retained decoded value before new input is consumed.
    ///
    /// # Parameters
    ///
    /// - `state`: Current conversion cursors and output buffer.
    ///
    /// # Returns
    ///
    /// Returns conversion progress when the pending value needs more output
    /// capacity, or `None` when the pending value was fully encoded.
    ///
    /// # Errors
    ///
    /// Returns a converter error when encode handling fails.
    #[inline(always)]
    fn drain_pending(
        &mut self,
        state: &mut ConvertState<'_, D::Unit, E::Unit>,
    ) -> Result<Option<TranscodeProgress>, ConvertErrorOf<D, E, DH, EH>> {
        let Some(pending) = self.pending.take() else {
            return Ok(None);
        };
        self.encode_pending(pending, state)
    }

    /// Drains source-side decode finish output and encodes emitted final
    /// values.
    ///
    /// When the decoder declares no finish output, still calls
    /// [`TranscodeDecodeEngine::finish`] so codec flush and hook teardown can
    /// run and fail even when zero values are emitted.
    ///
    /// # Parameters
    ///
    /// - `state`: Current conversion cursors and output buffer.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after all decode-finish values have been encoded.
    ///
    /// # Errors
    ///
    /// Returns a converter error when decode finish or encode handling fails.
    ///
    /// # Panics
    ///
    /// Panics when a decode-finish value cannot be encoded within the
    /// capacity reserved by [`Self::max_finish_output_len`].
    fn drain_decoder_finish(
        &mut self,
        state: &mut ConvertState<'_, D::Unit, E::Unit>,
    ) -> Result<(), ConvertErrorOf<D, E, DH, EH>>
    where
        D::Value: Default,
    {
        let value_count = self.decode_engine.max_finish_output_len()?;
        if value_count == 0 {
            // Skip the Vec allocation when the decoder declares no finish
            // output. We still call finish() so that
            // codec.decode_flush and hooks.finish_hooks both run —
            // hooks may do validation or teardown (e.g. checksum
            // verification) that can fail even when emitting zero
            // values. Passing an empty slice is safe here because the capacity
            // check inside finish() accepts required == 0 against any slice.
            self.decode_engine.finish(&mut [], 0).map_err(|error| {
                error.map_domain(TranscodeConvertEngineError::decode)
            })?;
            return Ok(());
        }
        // D::Value: Default is required only when value_count > 0. The bound
        // remains on the method signature for the general case; stateless
        // codecs never reach this branch.
        let mut decoded: Vec<D::Value> =
            (0..value_count).map(|_| D::Value::default()).collect();
        let written =
            self.decode_engine
                .finish(&mut decoded, 0)
                .map_err(|error| {
                    error.map_domain(TranscodeConvertEngineError::decode)
                })?;
        for value in decoded.into_iter().take(written) {
            let pending = PendingValue::new(value, 0);
            if self.encode_pending(pending, state)?.is_some() {
                unreachable!(
                    "converter finish bound must reserve space for decode finish values"
                );
            }
        }
        Ok(())
    }

    /// Encodes one pending value and applies output/pending state changes.
    ///
    /// When the target buffer lacks capacity, the value is put back into the
    /// pending slot and progress reports
    /// [`crate::TranscodeStatus::NeedOutput`].
    ///
    /// # Parameters
    ///
    /// - `pending`: Decoded value waiting for target output capacity.
    /// - `state`: Current conversion cursors and output buffer.
    ///
    /// # Returns
    ///
    /// Returns conversion progress when the value needs more output capacity,
    /// or `None` when the value was fully encoded.
    ///
    /// # Errors
    ///
    /// Returns a converter error when encode hook handling fails.
    fn encode_pending(
        &mut self,
        pending: PendingValue<D::Value>,
        state: &mut ConvertState<'_, D::Unit, E::Unit>,
    ) -> Result<Option<TranscodeProgress>, ConvertErrorOf<D, E, DH, EH>> {
        let input_index = pending.input_index();
        let output_index = state.output_cursor();
        let context = EncodeContext::new(
            pending.value(),
            input_index,
            state.output_mut(),
            output_index,
        );
        let outcome =
            self.encode_engine.encode_one(context).map_err(|error| {
                TranscodeError::domain(TranscodeConvertEngineError::encode(
                    error,
                ))
            })?;
        let progress = state.apply_encode_outcome(outcome);
        if progress.is_some() {
            self.pending.put(pending);
        }
        Ok(progress)
    }
}

impl<D, E, DH, EH> Default for TranscodeConvertEngine<D, E, DH, EH>
where
    D: Codec + Default,
    E: Codec<Value = D::Value> + Default,
    DH: TranscodeDecodeHooks<D> + Default,
    EH: TranscodeEncodeHooks<E> + Default,
{
    /// Creates a default buffered converter engine.
    ///
    /// # Returns
    ///
    /// Returns a converter engine constructed from default codecs and hooks.
    #[inline(always)]
    fn default() -> Self {
        Self::new(D::default(), E::default(), DH::default(), EH::default())
    }
}

impl<D, E, DH, EH> Transcoder<D::Unit, E::Unit>
    for TranscodeConvertEngine<D, E, DH, EH>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    D::Value: Default,
    DH: TranscodeDecodeHooks<D>,
    EH: TranscodeEncodeHooks<E>,
{
    type Error = TranscodeConvertEngineError<
        TranscodeDecodeEngineError<D::DecodeError, DH::Error>,
        TranscodeEncodeEngineError<E::EncodeError, EH::Error>,
    >;

    /// Returns an upper bound for target units produced from `input_len`
    /// units.
    #[inline(always)]
    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        TranscodeConvertEngine::max_transcode_output_len(self, input_len)
    }

    /// Returns an upper bound for target units emitted by finishing retained
    /// state.
    #[inline(always)]
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        TranscodeConvertEngine::max_finish_output_len(self)
    }

    /// Returns an upper bound for target units emitted when resetting stream
    /// state.
    #[inline(always)]
    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        TranscodeConvertEngine::max_reset_output_len(self)
    }

    /// Clears retained conversion state and emits target reset output.
    #[inline(always)]
    fn reset(
        &mut self,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        TranscodeConvertEngine::reset(self, output, output_index)
    }

    /// Converts source units into target units.
    #[inline(always)]
    fn transcode(
        &mut self,
        input: &[D::Unit],
        input_index: usize,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
        TranscodeConvertEngine::transcode(
            self,
            input,
            input_index,
            output,
            output_index,
        )
    }

    /// Finishes retained converter output after EOF.
    #[inline(always)]
    fn finish(
        &mut self,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        TranscodeConvertEngine::finish(self, output, output_index)
    }
}
