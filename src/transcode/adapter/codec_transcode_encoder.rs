// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Buffered encoder adapter backed by a low-level codec.

use super::super::engine::TranscodeEncodeEngine;
use super::CodecTranscodeEncodeHooks;
use crate::CapacityError;
use crate::Codec;
use crate::TranscodeEncodeErrorOf;
use crate::TranscodeEncoder;
use crate::TranscodeProgress;
use crate::Transcoder;

/// Encodes values into caller-provided output units by using a [`Codec`].
///
/// `CodecTranscodeEncoder` is the default bridge from the low-level unchecked
/// [`Codec`] contract to the buffered [`Transcoder`] and
/// [`TranscodeEncoder`] contracts. It encodes complete values only; when the
/// remaining output capacity is smaller than `codec.encode_len(value)`, it
/// stops before consuming that input value and reports
/// [`crate::TranscodeStatus::NeedOutput`].
///
/// # Type Parameters
///
/// - `C`: Low-level codec used to encode values.
///
/// # Examples
///
/// ```
/// use core::convert::Infallible;
/// use core::num::NonZeroUsize;
///
/// use qubit_codec::Codec;
/// use qubit_codec::CodecTranscodeEncoder;
/// use qubit_codec::DecodeFailure;
/// use qubit_codec::Transcoder;
///
/// # struct ByteCodec;
/// # impl Codec for ByteCodec {
/// #     type Value = u8;
/// #     type Unit = u8;
/// #     type DecodeError = Infallible;
/// #     type EncodeError = Infallible;
/// #     const MIN_UNITS_PER_VALUE: usize = 1;
/// #     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
/// #     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
/// #     unsafe fn decode(&mut self, input: &[u8], index: usize)
/// #         -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
/// #         Ok((input[index], NonZeroUsize::new(1).unwrap()))
/// #     }
/// #     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
/// #         -> Result<usize, Infallible> {
/// #         output[index] = *value;
/// #         Ok(1)
/// #     }
/// # }
/// let mut encoder = CodecTranscodeEncoder::new(ByteCodec);
/// let mut output = [0; 2];
/// assert_eq!(encoder.reset(&mut output, 0).unwrap(), 0);
/// let progress = encoder.transcode(&[7, 9], 0, &mut output, 0).unwrap();
/// assert_eq!(progress.written(), 2);
/// assert_eq!(output, [7, 9]);
/// ```
#[derive(Debug)]
pub struct CodecTranscodeEncoder<C> {
    /// Common buffered encoding engine.
    engine: TranscodeEncodeEngine<C, CodecTranscodeEncodeHooks>,
}

impl<C> CodecTranscodeEncoder<C>
where
    C: Codec,
{
    /// Creates a buffered encoder backed by `codec`.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec used to encode values.
    ///
    /// # Returns
    ///
    /// Returns a buffered encoder adapter for the supplied codec.
    #[inline]
    #[must_use]
    pub fn new(codec: C) -> Self {
        Self {
            engine: TranscodeEncodeEngine::new(codec, CodecTranscodeEncodeHooks),
        }
    }

    /// Borrows the codec without allocating or changing stream state.
    ///
    /// # Returns
    ///
    /// The wrapped codec, borrowed for the lifetime of `self`.
    #[inline]
    #[must_use]
    pub fn codec(&self) -> &C {
        self.engine.codec()
    }

    /// Returns a mutable reference to the wrapped codec.
    ///
    /// Mutating a codec during an active stream can invalidate that stream's
    /// assumptions; reset the adapter before continuing with the new codec
    /// configuration.
    ///
    /// # Returns
    ///
    /// An exclusive borrow of the wrapped codec tied to `self`.
    #[inline]
    #[must_use]
    pub fn codec_mut(&mut self) -> &mut C {
        self.engine.codec_mut()
    }

    /// Consumes the adapter and returns its wrapped codec.
    ///
    /// Any buffered lifecycle state and internal hooks are discarded.
    ///
    /// # Returns
    ///
    /// The owned codec, without resetting or finishing it.
    #[inline]
    #[must_use]
    pub fn into_codec(self) -> C {
        self.engine.into_parts().0
    }
}

impl<C> Transcoder for CodecTranscodeEncoder<C>
where
    C: Codec,
{
    /// Values accepted by the wrapped codec.
    type Input = C::Value;
    /// Representation units emitted by the wrapped codec.
    type Output = C::Unit;
    /// Domain, contract, and codec failures during buffered encoding.
    type Error = TranscodeEncodeErrorOf<C>;

    /// Gets the maximum number of output units needed for `input_len`
    /// values.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Logical input values the caller plans to encode.
    ///
    /// # Returns
    ///
    /// a conservative upper bound for output units.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityError`] if the engine cannot represent the bound in
    /// `usize`.
    #[inline]
    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        self.engine.max_transcode_output_len(input_len)
    }

    /// Gets the global maximum units emitted when resetting internal state.
    ///
    /// # Returns
    ///
    /// the maximum units emitted when resetting internal state.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityError`] if the engine cannot represent the bound in
    /// `usize`.
    #[inline]
    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        self.engine.max_reset_output_len()
    }

    /// Gets the global maximum units emitted by finishing internal state.
    ///
    /// # Returns
    ///
    /// the number of units that may be emitted by finishing state.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityError`] if the engine cannot represent the bound in
    /// `usize`.
    #[inline]
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        self.engine.max_finish_output_len()
    }

    /// Runs before-reset cleanup and emits stream-start output.
    ///
    /// # Parameters
    ///
    /// - `output`: Destination for codec reset units.
    /// - `output_index`: Start position in `output`.
    ///
    /// # Returns
    ///
    /// The number of reset units written; previous lifecycle state is
    /// discarded.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid index, insufficient reset capacity,
    /// a violated codec contract, or a codec reset failure.
    fn reset(&mut self, output: &mut [C::Unit], output_index: usize) -> Result<usize, TranscodeEncodeErrorOf<C>> {
        self.engine.reset(output, output_index)
    }

    /// Encodes values into the supplied output buffer.
    ///
    /// # Parameters
    ///
    /// - `input`: Input value slice.
    /// - `input_index`: Absolute input index where encoding starts.
    /// - `output`: Destination unit slice.
    /// - `output_index`: Absolute output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns conversion progress for consumed input and produced output
    /// units.
    ///
    /// # Errors
    ///
    /// Returns an encode error when indices are invalid or when encoding cannot
    /// continue under current policy.
    fn transcode(
        &mut self,
        input: &[C::Value],
        input_index: usize,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeEncodeErrorOf<C>> {
        self.engine.transcode(input, input_index, output, output_index)
    }

    /// Finishes internally retained output after EOF.
    ///
    /// # Parameters
    ///
    /// - `output`: Destination unit slice for finalization output.
    /// - `output_index`: Absolute output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of units written by finalization.
    ///
    /// # Errors
    ///
    /// Returns a finish error if retained output cannot be fully emitted.
    fn finish(&mut self, output: &mut [C::Unit], output_index: usize) -> Result<usize, TranscodeEncodeErrorOf<C>> {
        self.engine.finish(output, output_index)
    }
}

impl<C> TranscodeEncoder for CodecTranscodeEncoder<C>
where
    C: Codec,
{
    /// Codec-specific encoding failure carried by the adapter.
    type EncodeError = C::EncodeError;
}

impl<C> Default for CodecTranscodeEncoder<C>
where
    C: Codec + Default,
{
    /// Creates a default codec-backed buffered encoder.
    ///
    /// # Returns
    ///
    /// Returns an encoder backed by `C::default()`.
    fn default() -> Self {
        Self::new(C::default())
    }
}
