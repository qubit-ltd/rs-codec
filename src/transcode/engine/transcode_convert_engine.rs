// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Reusable buffered converter engine for codec-backed transcoding.
//!
//! Bridges a source [`crate::engine::TranscodeDecodeEngine`] and a target
//! [`crate::engine::TranscodeEncodeEngine`] into one unit-to-unit conversion
//! pipeline.

use core::num::NonZeroUsize;

use super::super::internal::convert_state::ConvertState;
use super::super::internal::encode_attempt::EncodeAttempt;
use super::super::internal::lifecycle_guard::LifecycleGuard;
use super::super::internal::pending_value::PendingValue;
use super::super::internal::pending_value_slot::PendingValueSlot;
use super::TranscodeDecodeHooks;
use super::TranscodeEncodeHooks;
use super::transcode_decode_engine::TranscodeDecodeEngine;
use super::transcode_encode_engine::TranscodeEncodeEngine;
use crate::CapacityError;
use crate::Codec;
use crate::TranscodeConvertError;
use crate::TranscodeConvertErrorOf;
use crate::TranscodeConverter;
use crate::TranscodeFailure;
use crate::TranscodeProgress;
use crate::Transcoder;
use crate::codec::assert_unit_bounds;

/// Adds two independent target-output capacity bounds.
fn add_convert_output_bounds(first: usize, second: usize) -> Result<usize, CapacityError> {
    first.checked_add(second).ok_or(CapacityError::OutputLengthOverflow)
}

/// Adds three independent target-output capacity bounds.
fn sum_convert_output_bounds(first: usize, second: usize, third: usize) -> Result<usize, CapacityError> {
    let partial = add_convert_output_bounds(first, second)?;
    add_convert_output_bounds(partial, third)
}

/// Asserts that a pre-reserved conversion phase did not need more output.
fn assert_reserved_output_drained(progress: Option<TranscodeProgress>, message: &'static str) {
    assert!(progress.is_none(), "{message}");
}

/// Reusable buffered conversion engine for codec-backed converters.
///
/// The engine owns reusable buffered decode and encode engines. It keeps
/// common converter control flow private: index validation, pending-value
/// retention, finish draining, decode-error policy dispatch, encode attempts,
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
/// same unchecked codec and hook primitives as
/// [`crate::engine::TranscodeDecodeEngine`] and
/// [`crate::engine::TranscodeEncodeEngine`]. It does not call one-value public
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
///     TranscodeDecodeErrorOf,
///     TranscodeEncodeErrorOf,
///     DecodeFailure,
///     TranscodeStatus,
/// };
/// use qubit_codec::engine::{
///     DecodeContext,
///     EncodeContext,
///     EncodeUnencodableAction,
///     TranscodeConvertEngine,
///     TranscodeDecodeHooks,
///     TranscodeEncodeHooks,
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
///     const MIN_UNITS_PER_VALUE: usize = 1;
///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
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
///     ) -> Result<usize, Self::EncodeError> {
///         output[index] = *value;
///         Ok(1)
///     }
/// }
///
/// impl Codec for TargetCodec {
///     type Value = u8;
///     type Unit = u8;
///     type DecodeError = Infallible;
///     type EncodeError = Infallible;
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
///         Ok((input[index], NonZeroUsize::MIN))
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
/// struct StrictDecodeHooks;
///
/// impl TranscodeDecodeHooks<SourceCodec> for StrictDecodeHooks {
///     fn handle_invalid_decode(
///         &mut self,
///         _codec: &mut SourceCodec,
///         error: &Infallible,
///         _consumed: Option<NonZeroUsize>,
///         _context: DecodeContext,
///     ) -> Result<qubit_codec::engine::DecodeInvalidAction<u8>, TranscodeDecodeErrorOf<SourceCodec>> {
///         match *error {}
///     }
/// }
///
/// struct StrictEncodeHooks;
///
/// impl TranscodeEncodeHooks<TargetCodec> for StrictEncodeHooks {
///     fn handle_unencodable_encode(
///         &mut self,
///         _codec: &mut TargetCodec,
///         _context: &EncodeContext<'_, u8>,
///     ) -> Result<EncodeUnencodableAction<u8>, TranscodeEncodeErrorOf<TargetCodec>> {
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
/// let mut reset_output = [];
/// engine.reset(&mut reset_output, 0)?;
///
/// let progress = engine.transcode(&input, 0, &mut output, 0)?;
/// match progress.status() {
///     TranscodeStatus::NeedOutput { .. } => {
///         assert_eq!(2, progress.written());
///         assert_eq!([2, 3], output);
///         // Drain `output[..progress.written()]`, then resume at
///         // `progress.read()` with fresh output capacity.
///     }
///     TranscodeStatus::Complete => unreachable!("output is intentionally short"),
///     TranscodeStatus::NeedInput { .. } => unreachable!("input is complete"),
/// }
/// # Ok::<(), qubit_codec::TranscodeConvertErrorOf<SourceCodec, TargetCodec>>(())
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
    /// Guard for the `reset → transcode* → finish` lifecycle in every build
    /// profile. The converter owns its own guard rather than delegating to the
    /// inner decode/encode engines, because lifecycle events here describe the
    /// converter as a whole.
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
    /// # Compile-Time Checks
    ///
    /// Fails to compile when either codec declares a zero decode unit bound or
    /// when [`Codec::MIN_UNITS_PER_VALUE`] exceeds
    /// [`Codec::MAX_DECODE_UNITS_PER_VALUE`].
    #[inline]
    #[must_use]
    pub fn new(decoder: D, encoder: E, decode_hooks: DH, encode_hooks: EH) -> Self {
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

    /// Returns the decode hooks used by this engine.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the decode hooks owned by this engine.
    #[inline(always)]
    #[must_use]
    pub const fn decode_hooks(&self) -> &DH {
        self.decode_engine.hooks()
    }

    /// Returns the decode hooks mutably.
    ///
    /// Mutating the returned hooks does not reset the converter or clear a
    /// pending value. The replacement hooks must continue to satisfy their
    /// global output-capacity contract for every reachable converter state.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the decode hooks owned by this engine.
    #[inline(always)]
    #[must_use]
    pub fn decode_hooks_mut(&mut self) -> &mut DH {
        self.decode_engine.hooks_mut()
    }

    /// Returns the encode hooks used by this engine.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the encode hooks owned by this engine.
    #[inline(always)]
    #[must_use]
    pub const fn encode_hooks(&self) -> &EH {
        self.encode_engine.hooks()
    }

    /// Returns the encode hooks mutably.
    ///
    /// Mutating the returned hooks does not reset the converter or clear a
    /// pending value. The replacement hooks must continue to satisfy their
    /// global output-capacity contract for every reachable converter state.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the encode hooks owned by this engine.
    #[inline(always)]
    #[must_use]
    pub fn encode_hooks_mut(&mut self) -> &mut EH {
        self.encode_engine.hooks_mut()
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
    /// The bound sums three parts: one possible retained pending value, the
    /// maximum decoded values from the source side, and the maximum target
    /// units for those values on the encode side. It covers only the streaming
    /// convert phase and remains valid even when no value is currently pending.
    /// Downstream converters must use this engine-level API for capacity
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
    pub fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        let pending_units = self.encode_engine.max_transcode_output_len(1)?;
        let decoded_values = self.decode_engine.max_transcode_output_len(input_len)?;
        let converted_units = self.encode_engine.max_transcode_output_len(decoded_values)?;
        add_convert_output_bounds(converted_units, pending_units)
    }

    /// Returns the global maximum target units emitted when resetting stream
    /// state.
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
        add_convert_output_bounds(decode_reset_units, encode_reset_units)
    }

    /// Returns the maximum target units emitted by finishing retained state.
    ///
    /// The bound covers one possible retained pending value, the global
    /// decode-side finish-value bound (encoded to target units), and the
    /// global encode-side finish-unit bound. It is independent of whether a
    /// pending value currently exists.
    ///
    /// # Returns
    ///
    /// Returns the combined pending, decode-finish, and encode-finish output
    /// bound, or a capacity error on arithmetic overflow.
    #[must_use = "capacity planning can fail on overflow"]
    pub fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        let pending_units = self.encode_engine.max_transcode_output_len(1)?;
        let decoder_finish_values = self.decode_engine.max_finish_output_len()?;
        let decoder_finish_units = self.encode_engine.max_transcode_output_len(decoder_finish_values)?;
        let encoder_finish_units = self.encode_engine.max_finish_output_len()?;
        sum_convert_output_bounds(pending_units, decoder_finish_units, encoder_finish_units)
    }

    /// Returns the finish-output bound for the converter's current pending
    /// state.
    ///
    /// Public capacity methods expose global bounds across every transient
    /// state. A concrete finish call can use this narrower checked bound while
    /// still relying on global component bounds for decoder and encoder
    /// finalization.
    ///
    /// # Returns
    ///
    /// Returns the current finish-output bound, or a capacity error on
    /// arithmetic overflow.
    fn current_finish_output_len(&self) -> Result<usize, CapacityError> {
        let pending_units = self.pending.current_output_len(&self.encode_engine)?;
        let decoder_finish_values = self.decode_engine.max_finish_output_len()?;
        let decoder_finish_units = self.encode_engine.max_transcode_output_len(decoder_finish_values)?;
        let encoder_finish_units = self.encode_engine.max_finish_output_len()?;
        sum_convert_output_bounds(pending_units, decoder_finish_units, encoder_finish_units)
    }

    /// Returns the maximum target units needed by a complete one-shot
    /// conversion.
    ///
    /// The returned bound covers conversion reset output, the streaming convert
    /// phase for `input_len` source units, and finish output. Its components
    /// are global across transient state. Higher-level complete conversion
    /// helpers should use this engine-level bound instead of recomputing
    /// capacity from the source or target codec constants, because decode
    /// and encode hooks may change streaming or finish output.
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
    pub fn max_total_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        let reset = self.max_reset_output_len()?;
        let transcode = self.max_transcode_output_len(input_len)?;
        let finish = self.max_finish_output_len()?;
        sum_convert_output_bounds(reset, transcode, finish)
    }

    /// Clears retained conversion state, runs before-reset hooks, and emits
    /// stream-start encode output.
    ///
    /// Reset clears any retained pending value, resets the target encoder, then
    /// drains decode-side reset values through that reset encoder state. The
    /// caller must provide enough output capacity for
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
    /// emission fails. Capacity and index failures occur before converter state
    /// is changed. Once reset execution starts, an error poisons the converter
    /// until a later reset succeeds.
    ///
    /// # Panics
    ///
    /// Panics when decode-reset values cannot be encoded within the capacity
    /// reserved by [`Self::max_reset_output_len`].
    pub fn reset(&mut self, output: &mut [E::Unit], output_index: usize) -> Result<usize, TranscodeConvertErrorOf<D, E>>
    where
        D::Value: Default,
    {
        let required = self.max_reset_output_len()?;
        TranscodeFailure::ensure_output_capacity(output.len(), output_index, required)?;
        self.lifecycle.on_reset_start();

        self.pending.clear();

        // Reset the target first because source-side reset values must be
        // encoded under the target's new-stream state. The reset bound reserves
        // space for both target-owned output and the encoded source-reset
        // values, so encode_pending should never report `NeedOutput` here.
        let empty_input: &[D::Unit] = &[];
        let mut state = ConvertState::new(empty_input, 0, output, output_index);
        let output_cursor = state.output_cursor();
        let encoder_written = self.encode_engine.reset(state.output_mut(), output_cursor)?;
        state.advance_output(encoder_written);
        self.drain_decoder_reset(&mut state)?;
        self.lifecycle.on_reset_success();
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
    /// error path. Returns [`TranscodeFailure::TranscodeBeforeReset`] when the
    /// engine has not completed its first reset. Returns
    /// [`TranscodeFailure::TranscodeAfterFinish`] when the logical stream was
    /// already finished and has not been reset, or
    /// [`TranscodeFailure::LifecyclePoisoned`] when an earlier reset or finish
    /// failed after execution started.
    pub fn transcode(
        &mut self,
        input: &[D::Unit],
        input_index: usize,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
        self.transcode_with_eof(input, input_index, output, output_index, false)
    }

    /// Converts source units after the caller has established end of input.
    #[inline(always)]
    pub fn transcode_eof(
        &mut self,
        input: &[D::Unit],
        input_index: usize,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
        self.transcode_with_eof(input, input_index, output, output_index, true)
    }

    fn transcode_with_eof(
        &mut self,
        input: &[D::Unit],
        input_index: usize,
        output: &mut [E::Unit],
        output_index: usize,
        end_of_input: bool,
    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
        self.lifecycle.on_transcode()?;
        TranscodeFailure::ensure_transcode_indices(input.len(), input_index, output.len(), output_index)?;

        let mut state = ConvertState::new(input, input_index, output, output_index);

        // A retained decoded value must be written before consuming more input,
        // otherwise callers could observe output reordered across buffer turns.
        if let Some(progress) = self.drain_pending(&mut state)? {
            return Ok(progress);
        }

        let min_input_units =
            NonZeroUsize::new(D::MIN_UNITS_PER_VALUE).expect("Codec::MIN_UNITS_PER_VALUE is non-zero");
        let min_input_len = min_input_units.get();
        while state.has_input() {
            let available = state.available_input();
            if available < min_input_len && !end_of_input {
                return Ok(state.need_input_progress(min_input_units));
            }

            let previous_read = state.read();
            // Each hot-path step decodes one source value and immediately tries
            // to encode it, preserving backpressure at the target output.
            if let Some(progress) = self.convert_next(&mut state, end_of_input)? {
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
    /// hook finalization fails. Returns
    /// [`TranscodeFailure::FinishBeforeReset`] when the engine has not
    /// completed its first reset. Returns
    /// [`TranscodeFailure::FinishAfterFinish`] when the logical stream was
    /// already finished and has not been reset, or
    /// [`TranscodeFailure::LifecyclePoisoned`] when an earlier reset or finish
    /// failed after execution started. Capacity and index failures occur before
    /// finish execution and remain retryable; later failures poison the
    /// converter until reset succeeds.
    ///
    /// # Panics
    ///
    /// Panics when a retained pending value or decode-finish value cannot be
    /// encoded within the planned finish capacity.
    pub fn finish(
        &mut self,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<usize, TranscodeConvertErrorOf<D, E>>
    where
        D::Value: Default,
    {
        self.lifecycle.on_finish_attempt()?;
        let required = self.current_finish_output_len()?;
        TranscodeFailure::ensure_output_capacity(output.len(), output_index, required)?;
        self.lifecycle.on_finish_start();

        let empty_input: &[D::Unit] = &[];
        let mut state = ConvertState::new(empty_input, 0, output, output_index);
        // Finish keeps the same priority as transcode: output any retained
        // decoded value before asking source-side hooks for final values.
        let progress = self.drain_pending(&mut state)?;
        assert_reserved_output_drained(progress, "converter finish bound must reserve space for pending values");

        // Source-side finish may emit one or more final values. Drain them into
        // the target encoder before finishing target-side hook state.
        self.drain_decoder_finish(&mut state)?;

        let output_cursor = state.output_cursor();
        let written = self.encode_engine.finish(state.output_mut(), output_cursor)?;
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
    pub fn transcode_complete_into(
        &mut self,
        input: &[D::Unit],
        output: &mut [E::Unit],
    ) -> Result<usize, TranscodeConvertErrorOf<D, E>>
    where
        D::Value: Default,
    {
        <Self as Transcoder>::transcode_complete_into(self, input, output)
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
    ) -> Result<(), TranscodeConvertErrorOf<D, E>>
    where
        D::Value: Default,
    {
        let value_count = D::MAX_DECODE_RESET_VALUES;
        if value_count == 0 {
            // Stateless decoder: still call decode_reset so codecs whose
            // hooks own teardown side effects (e.g. clearing accumulators)
            // run them. The empty slice is safe because the capacity check
            // inside `reset` accepts `required == 0` against any slice.
            self.decode_engine.reset(&mut [], 0)?;
            return Ok(());
        }
        // `D::Value: Default` is only consulted when the decoder declares
        // reset output. Stateless codecs never reach this branch.
        let mut reset_values: Vec<D::Value> = (0..value_count).map(|_| D::Value::default()).collect();
        let written = self.decode_engine.reset(&mut reset_values, 0)?;
        for value in reset_values.into_iter().take(written) {
            let pending = PendingValue::new(value, 0);
            let progress = self.encode_pending(pending, state)?;
            assert_reserved_output_drained(
                progress,
                "converter reset bound must reserve space for decode reset values",
            );
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
        end_of_input: bool,
    ) -> Result<Option<TranscodeProgress>, TranscodeConvertErrorOf<D, E>> {
        let (outcome, pending) =
            self.decode_engine
                .decode_one(state.input(), state.decode_context(), end_of_input, PendingValue::new)?;
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
    ) -> Result<Option<TranscodeProgress>, TranscodeConvertErrorOf<D, E>> {
        let Some(pending) = self.pending.take() else {
            return Ok(None);
        };
        self.encode_pending(pending, state)
    }

    /// Drains source-side decode finish output and encodes emitted final
    /// values.
    ///
    /// When the decoder declares no finish output, still calls
    /// [`TranscodeDecodeEngine::finish`] so codec finish and hook teardown can
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
    ) -> Result<(), TranscodeConvertErrorOf<D, E>>
    where
        D::Value: Default,
    {
        let value_count = self.decode_engine.max_finish_output_len()?;
        if value_count == 0 {
            // Skip the Vec allocation when the decoder declares no finish
            // output. We still call finish() so that
            // codec.decode_finish and hooks.finish_hooks both run —
            // hooks may do validation or teardown (e.g. checksum
            // verification) that can fail even when emitting zero
            // values. Passing an empty slice is safe here because the capacity
            // check inside finish() accepts required == 0 against any slice.
            self.decode_engine.finish(&mut [], 0)?;
            return Ok(());
        }
        // D::Value: Default is required only when value_count > 0. The bound
        // remains on the method signature for the general case; stateless
        // codecs never reach this branch.
        let mut decoded: Vec<D::Value> = (0..value_count).map(|_| D::Value::default()).collect();
        let written = self.decode_engine.finish(&mut decoded, 0)?;
        for value in decoded.into_iter().take(written) {
            let pending = PendingValue::new(value, 0);
            let progress = self.encode_pending(pending, state)?;
            assert_reserved_output_drained(
                progress,
                "converter finish bound must reserve space for decode finish values",
            );
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
    ) -> Result<Option<TranscodeProgress>, TranscodeConvertErrorOf<D, E>> {
        let input_index = pending.input_index();
        let output_index = state.output_cursor();
        let attempt = EncodeAttempt::new(pending.value(), input_index, state.output_mut(), output_index);
        let outcome = match self.encode_engine.encode_one(attempt) {
            Ok(outcome) => outcome,
            Err(error) => {
                return Err(TranscodeConvertError::from_encode_error_with_value(
                    error,
                    pending.into_value(),
                ));
            }
        };
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

impl<D, E, DH, EH> Transcoder for TranscodeConvertEngine<D, E, DH, EH>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    D::Value: Default,
    DH: TranscodeDecodeHooks<D>,
    EH: TranscodeEncodeHooks<E>,
{
    type Input = D::Unit;
    type Output = E::Unit;
    type Error = TranscodeConvertErrorOf<D, E>;

    /// Returns an upper bound for target units produced from `input_len`
    /// units.
    #[inline(always)]
    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
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
    fn reset(&mut self, output: &mut [E::Unit], output_index: usize) -> Result<usize, TranscodeConvertErrorOf<D, E>> {
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
    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
        TranscodeConvertEngine::transcode(self, input, input_index, output, output_index)
    }

    #[inline(always)]
    fn transcode_eof(
        &mut self,
        input: &[D::Unit],
        input_index: usize,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
        TranscodeConvertEngine::transcode_eof(self, input, input_index, output, output_index)
    }

    /// Finishes retained converter output after EOF.
    #[inline(always)]
    fn finish(&mut self, output: &mut [E::Unit], output_index: usize) -> Result<usize, TranscodeConvertErrorOf<D, E>> {
        TranscodeConvertEngine::finish(self, output, output_index)
    }
}

impl<D, E, DH, EH> TranscodeConverter for TranscodeConvertEngine<D, E, DH, EH>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    D::Value: Default,
    DH: TranscodeDecodeHooks<D>,
    EH: TranscodeEncodeHooks<E>,
{
    type DecodeError = D::DecodeError;
    type EncodeError = E::EncodeError;
    type Value = D::Value;
}
