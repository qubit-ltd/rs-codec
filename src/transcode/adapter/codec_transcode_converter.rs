// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Buffered converter adapter backed by two low-level codecs.

use core::fmt;

use super::super::engine::TranscodeConvertEngine;
use super::CodecTranscodeDecodeHooks;
use super::CodecTranscodeEncodeHooks;
use crate::CapacityError;
use crate::Codec;
use crate::TranscodeConvertErrorOf;
use crate::TranscodeConverter;
use crate::TranscodeProgress;
use crate::Transcoder;

/// Converts source units to target units through a decoded value by using
/// codecs.
///
/// The converter decodes one source value with the decoder codec, then encodes
/// that value with the encoder codec. If the current output buffer cannot hold
/// the encoded value, the already decoded value is retained by the common
/// converter engine and must be drained before more source input is consumed.
/// Incomplete source tails are left in the caller-provided input slice; callers
/// own input-buffer refill and EOF incomplete-tail policy.
///
/// Because finalization receives no source input, the source codec should have
/// locally decidable decode boundaries for the default converter bridge. Source
/// formats that require EOF-aware maximal-munch parsing or delayed boundary
/// decisions should implement that source-side policy in a custom transcoder or
/// facade before conversion.
///
/// # Type Parameters
///
/// - `D`: Low-level codec used to decode source units.
/// - `E`: Low-level codec used to encode target units.
///
/// # Examples
///
/// A byte-preserving codec can be adapted to the buffered streaming interface.
///
/// ```
/// use core::convert::Infallible;
/// use core::num::NonZeroUsize;
///
/// use qubit_codec::Codec;
/// use qubit_codec::CodecTranscodeConverter;
/// use qubit_codec::DecodeFailure;
///
/// # struct ByteCodec;
/// # impl Codec for ByteCodec {
/// #     type Value = u8;
/// #     type Unit = u8;
/// #     type DecodeError = Infallible;
/// #     type EncodeError = Infallible;
/// #     const MIN_UNITS_PER_VALUE: usize = 1;
/// #     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
/// #     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
/// #     unsafe fn decode(&mut self, input: &[u8], index: usize)
/// #         -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
/// #         Ok((input[index], NonZeroUsize::MIN))
/// #     }
/// #     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
/// #         -> Result<usize, Infallible> {
/// #         output[index] = *value;
/// #         Ok(1)
/// #     }
/// # }
/// let mut converter = CodecTranscodeConverter::new(ByteCodec, ByteCodec);
/// let mut output = [0_u8; 2];
/// converter.reset(&mut output, 0).unwrap();
/// converter.transcode(&[7, 9], 0, &mut output, 0).unwrap();
/// assert_eq!(output, [7, 9]);
/// ```
pub struct CodecTranscodeConverter<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
{
    /// Common buffered converter engine.
    engine: TranscodeConvertEngine<D, E, CodecTranscodeDecodeHooks, CodecTranscodeEncodeHooks>,
}

impl<D, E> CodecTranscodeConverter<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
{
    /// Creates a buffered converter backed by decoder and encoder codecs.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Low-level codec used to decode source units.
    /// - `encoder`: Low-level codec used to encode target units.
    ///
    /// # Returns
    ///
    /// Returns a buffered converter adapter for the supplied codecs.
    #[inline]
    #[must_use]
    pub fn new(decoder: D, encoder: E) -> Self {
        Self {
            engine: TranscodeConvertEngine::new(decoder, encoder, CodecTranscodeDecodeHooks, CodecTranscodeEncodeHooks),
        }
    }

    /// Returns a shared reference to the source codec.
    ///
    /// # Returns
    ///
    /// Borrows the source codec without allocating or changing stream state.
    #[inline]
    #[must_use]
    pub fn source_codec(&self) -> &D {
        self.engine.source_codec()
    }

    /// Returns a mutable reference to the source codec.
    ///
    /// Mutating a codec during an active stream can invalidate that stream's
    /// assumptions; reset the adapter before continuing with the new codec
    /// configuration.
    ///
    /// # Returns
    ///
    /// Exclusively borrows the codec while the adapter remains borrowed.
    #[inline]
    #[must_use]
    pub fn source_codec_mut(&mut self) -> &mut D {
        self.engine.source_codec_mut()
    }

    /// Returns a shared reference to the target codec.
    ///
    /// # Returns
    ///
    /// Borrows the target codec without allocating or changing stream state.
    #[inline]
    #[must_use]
    pub fn target_codec(&self) -> &E {
        self.engine.target_codec()
    }

    /// Returns a mutable reference to the target codec.
    ///
    /// Mutating a codec during an active stream can invalidate that stream's
    /// assumptions; reset the adapter before continuing with the new codec
    /// configuration.
    ///
    /// # Returns
    ///
    /// Exclusively borrows the codec while the adapter remains borrowed.
    #[inline]
    #[must_use]
    pub fn target_codec_mut(&mut self) -> &mut E {
        self.engine.target_codec_mut()
    }

    /// Consumes the adapter and returns its source and target codecs.
    ///
    /// Any buffered lifecycle state, pending value, and internal hooks are
    /// discarded.
    ///
    /// # Returns
    ///
    /// Returns the owned source codec followed by the owned target codec.
    #[inline]
    #[must_use]
    pub fn into_codecs(self) -> (D, E) {
        let (source, target, _, _) = self.engine.into_parts();
        (source, target)
    }

    /// Returns an upper bound for target units produced from `input_len` units.
    ///
    /// This concrete adapter method is available even when `D::Value` does not
    /// implement [`Default`].
    ///
    /// # Parameters
    ///
    /// - `input_len`: Source units the caller plans to convert.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound for produced target units.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityError::OutputLengthOverflow`] when component-bound
    /// arithmetic overflows.
    #[must_use = "capacity planning can fail on overflow"]
    #[inline]
    pub fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        self.engine.max_transcode_output_len(input_len)
    }

    /// Returns the global maximum target units emitted by finishing internal
    /// state.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound valid for every reachable converter
    /// state.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityError::OutputLengthOverflow`] when component-bound
    /// arithmetic overflows.
    #[must_use = "capacity planning can fail on overflow"]
    #[inline]
    pub fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        self.engine.max_finish_output_len()
    }

    /// Returns the maximum target units emitted when resetting stream state.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound valid for every reachable converter
    /// state.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityError::OutputLengthOverflow`] when component-bound
    /// arithmetic overflows.
    #[must_use = "capacity planning can fail on overflow"]
    #[inline]
    pub fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        self.engine.max_reset_output_len()
    }

    /// Clears retained pending output, resets target state, and then encodes
    /// source-side reset values.
    ///
    /// `D::Value: Default` is required so the engine can allocate scratch
    /// storage for any stream-start values the source decoder emits through
    /// [`Codec::decode_reset`] before they are
    /// piped through the target encoder. Stateless decoders never reach the
    /// allocating path; the bound is consulted only when
    /// [`Codec::MAX_DECODE_RESET_VALUES`]
    /// is non-zero.
    ///
    /// # Parameters
    ///
    /// - `output`: Target unit slice for stream-start output.
    /// - `output_index`: Absolute target output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of target units written while resetting component
    /// state.
    ///
    /// # Errors
    ///
    /// Returns a converter error when the output range is invalid or too
    /// small, or when decoder or encoder reset processing fails.
    pub fn reset(&mut self, output: &mut [E::Unit], output_index: usize) -> Result<usize, TranscodeConvertErrorOf<D, E>>
    where
        D::Value: Default,
    {
        self.engine.reset(output, output_index)
    }

    /// Converts source units into target units.
    ///
    /// This is the main streaming operation and does not require `D::Value` to
    /// implement [`Default`].
    ///
    /// # Parameters
    ///
    /// - `input`: Source unit slice.
    /// - `input_index`: Absolute source index where conversion starts.
    /// - `output`: Target unit slice.
    /// - `output_index`: Absolute target index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns conversion progress for consumed/produced counters and stop
    /// reason.
    ///
    /// # Errors
    ///
    /// Returns converter error when source or target indices are invalid, or
    /// when decoding/encoding fails under current policy.
    pub fn transcode(
        &mut self,
        input: &[D::Unit],
        input_index: usize,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
        self.engine.transcode(input, input_index, output, output_index)
    }

    /// Converts source units after the caller has established end of input.
    ///
    /// # Parameters
    ///
    /// - `input`: Final source unit slice; the caller must not supply more
    ///   input later.
    /// - `input_index`: Absolute index at which unread source units begin.
    /// - `output`: Target unit slice receiving converted values.
    /// - `output_index`: Absolute index at which target output begins.
    ///
    /// # Returns
    ///
    /// Returns consumed and produced counts with the conversion stop reason.
    ///
    /// # Errors
    ///
    /// Returns a converter error for invalid indices, invalid or incomplete
    /// final source input, or a decoder/encoder failure under the current
    /// policy.
    pub fn transcode_eof(
        &mut self,
        input: &[D::Unit],
        input_index: usize,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
        self.engine.transcode_eof(input, input_index, output, output_index)
    }

    /// Finishes internally retained output after EOF.
    ///
    /// Finalization delegates to the reusable converter engine. It drains
    /// retained pending output, encodes source-side decode flush values, and
    /// then finishes target-side encode hook state.
    ///
    /// # Parameters
    ///
    /// - `output`: Target unit slice for finalization output.
    /// - `output_index`: Absolute target output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of target units written by finalization.
    ///
    /// # Errors
    ///
    /// Returns a converter error for invalid output indices, insufficient
    /// output capacity, or failures while draining pending values or
    /// finishing either codec.
    pub fn finish(
        &mut self,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<usize, TranscodeConvertErrorOf<D, E>>
    where
        D::Value: Default,
    {
        self.engine.finish(output, output_index)
    }
}

impl<D, E> Transcoder for CodecTranscodeConverter<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    D::Value: Default,
{
    /// Source units accepted by the decoder codec.
    type Input = D::Unit;
    /// Target units produced by the encoder codec.
    type Output = E::Unit;
    /// Conversion failures retaining decoder, encoder, and pending-value
    /// context.
    type Error = TranscodeConvertErrorOf<D, E>;

    /// Returns an upper bound for target units produced from `input_len` units.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Source units the caller plans to convert.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound for produced target units.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityError::OutputLengthOverflow`] when component-bound
    /// arithmetic overflows.
    #[inline]
    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        CodecTranscodeConverter::max_transcode_output_len(self, input_len)
    }

    /// Returns the maximum target units emitted by finishing internal state.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound valid for every reachable converter
    /// state.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityError::OutputLengthOverflow`] when component-bound
    /// arithmetic overflows.
    #[inline]
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        CodecTranscodeConverter::max_finish_output_len(self)
    }

    /// Returns the maximum target units emitted when resetting stream state.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound valid for every reachable converter
    /// state.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityError::OutputLengthOverflow`] when component-bound
    /// arithmetic overflows.
    #[inline]
    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        CodecTranscodeConverter::max_reset_output_len(self)
    }

    /// Clears retained pending output, resets component state, and emits
    /// stream-start encode output.
    ///
    /// # Parameters
    ///
    /// - `output`: Target unit slice for stream-start output.
    /// - `output_index`: Absolute target output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of target units written while resetting component
    /// state.
    ///
    /// # Errors
    ///
    /// Returns a converter error when the output range is invalid or too
    /// small, or when decoder or encoder reset processing fails.
    fn reset(&mut self, output: &mut [E::Unit], output_index: usize) -> Result<usize, TranscodeConvertErrorOf<D, E>> {
        CodecTranscodeConverter::reset(self, output, output_index)
    }

    /// Converts source units into target units.
    ///
    /// # Parameters
    ///
    /// - `input`: Source unit slice.
    /// - `input_index`: Absolute source index where conversion starts.
    /// - `output`: Target unit slice.
    /// - `output_index`: Absolute target index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns conversion progress for consumed/produced counters and stop
    /// reason.
    ///
    /// # Errors
    ///
    /// Returns converter error when source or target indices are invalid, or
    /// when decoding/encoding fails under current policy.
    fn transcode(
        &mut self,
        input: &[D::Unit],
        input_index: usize,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
        CodecTranscodeConverter::transcode(self, input, input_index, output, output_index)
    }

    /// Transcodes an input segment while marking it as the end of the stream.
    ///
    /// This forwarding implementation delegates to the converter's EOF-aware
    /// operation so buffered decoder state can be finalized before encoding.
    ///
    /// # Parameters
    ///
    /// - `input`: Final source unit slice; the caller must not supply more
    ///   input later.
    /// - `input_index`: Absolute index at which unread source units begin.
    /// - `output`: Target unit slice receiving converted values.
    /// - `output_index`: Absolute index at which target output begins.
    ///
    /// # Returns
    ///
    /// Returns consumed and produced counts with the conversion stop reason.
    ///
    /// # Errors
    ///
    /// Returns a converter error for invalid indices, invalid or incomplete
    /// final source input, or a decoder/encoder failure under the current
    /// policy.
    fn transcode_eof(
        &mut self,
        input: &[D::Unit],
        input_index: usize,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
        CodecTranscodeConverter::transcode_eof(self, input, input_index, output, output_index)
    }

    /// Finishes internally retained output after EOF.
    ///
    /// # Parameters
    ///
    /// - `output`: Target unit slice for finalization output.
    /// - `output_index`: Absolute target output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of target units written by finalization.
    ///
    /// # Errors
    ///
    /// Returns a converter error for invalid output indices, insufficient
    /// output capacity, or failures while draining pending values or
    /// finishing either codec.
    fn finish(&mut self, output: &mut [E::Unit], output_index: usize) -> Result<usize, TranscodeConvertErrorOf<D, E>> {
        CodecTranscodeConverter::finish(self, output, output_index)
    }
}

impl<D, E> TranscodeConverter for CodecTranscodeConverter<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    D::Value: Default,
{
    /// Domain error reported by the source codec.
    type DecodeError = D::DecodeError;
    /// Domain error reported by the target codec.
    type EncodeError = E::EncodeError;
    /// Decoded value transferred from source codec to target codec.
    type Value = D::Value;
}

impl<D, E> Default for CodecTranscodeConverter<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    TranscodeConvertEngine<D, E, CodecTranscodeDecodeHooks, CodecTranscodeEncodeHooks>: Default,
{
    /// Creates a default codec-backed buffered converter.
    ///
    /// # Returns
    ///
    /// Returns a converter with default codecs and hooks.
    #[inline]
    fn default() -> Self {
        Self {
            engine: TranscodeConvertEngine::default(),
        }
    }
}

impl<D, E> fmt::Debug for CodecTranscodeConverter<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    TranscodeConvertEngine<D, E, CodecTranscodeDecodeHooks, CodecTranscodeEncodeHooks>: fmt::Debug,
{
    /// Formats the wrapped converter engine for debugging.
    ///
    /// # Parameters
    ///
    /// - `f`: Destination formatter.
    ///
    /// # Returns
    ///
    /// Returns `fmt::Result` from the formatter.
    ///
    /// # Errors
    ///
    /// Returns the formatter error if writing the debug representation fails.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodecTranscodeConverter")
            .field("engine", &self.engine)
            .finish()
    }
}
