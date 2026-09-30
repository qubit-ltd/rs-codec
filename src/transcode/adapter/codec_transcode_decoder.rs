// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Buffered decoder adapter backed by a low-level codec.

use super::super::engine::TranscodeDecodeEngine;
use super::CodecTranscodeDecodeHooks;
use crate::CapacityError;
use crate::Codec;
use crate::TranscodeDecodeErrorOf;
use crate::TranscodeDecoder;
use crate::TranscodeProgress;
use crate::Transcoder;

/// Decodes encoded units into caller-provided value buffers by using a
/// [`Codec`].
///
/// `CodecTranscodeDecoder` is a policy-free bridge from the low-level unchecked
/// [`Codec`] contract to [`Transcoder`] and [`TranscodeDecoder`]. It
/// leaves incomplete input tails in the caller-provided input slice while the
/// stream remains open. Call [`Transcoder::transcode_eof`] after upstream EOF
/// so codecs can apply their [`Codec::decode_eof`] policy to a trailing prefix.
///
/// # Type Parameters
///
/// - `C`: Low-level codec used to decode values.
///
/// # Examples
///
/// ```
/// use std::convert::Infallible;
/// use std::num::NonZeroUsize;
///
/// use qubit_codec::Codec;
/// use qubit_codec::CodecTranscodeDecoder;
/// use qubit_codec::DecodeFailure;
/// use qubit_codec::Transcoder;
///
/// struct Identity;
/// impl Codec for Identity {
///     type Value = u8;
///     type Unit = u8;
///     type DecodeError = Infallible;
///     type EncodeError = Infallible;
///     const MIN_UNITS_PER_VALUE: usize = 1;
///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
///     unsafe fn decode(&mut self, input: &[u8], index: usize)
///         -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
///         Ok((input[index], NonZeroUsize::new(1).unwrap()))
///     }
///     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
///         -> Result<usize, Infallible> {
///         output[index] = *value;
///         Ok(1)
///     }
/// }
/// let mut decoder = CodecTranscodeDecoder::new(Identity);
/// let mut output = [0; 2];
/// decoder.reset(&mut output, 0).unwrap();
/// decoder.transcode(&[4, 7], 0, &mut output, 0).unwrap();
/// assert_eq!(output, [4, 7]);
/// ```
#[derive(Debug, Default)]
pub struct CodecTranscodeDecoder<C> {
    /// Common buffered decoding engine.
    engine: TranscodeDecodeEngine<C, CodecTranscodeDecodeHooks>,
}

impl<C> CodecTranscodeDecoder<C>
where
    C: Codec,
{
    /// Creates a buffered decoder backed by `codec`.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec used to decode values.
    ///
    /// # Returns
    ///
    /// Returns a buffered decoder adapter for the supplied codec.
    #[inline]
    #[must_use]
    pub fn new(codec: C) -> Self {
        Self {
            engine: TranscodeDecodeEngine::new(codec, CodecTranscodeDecodeHooks),
        }
    }

    /// Returns a shared reference to the wrapped codec.
    ///
    /// # Returns
    ///
    /// A borrow of the codec that remains valid while this adapter is borrowed.
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
    /// An exclusive borrow of the wrapped codec; no allocation is performed.
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
    /// The owned codec, with its codec-specific state preserved.
    #[inline]
    #[must_use]
    pub fn into_codec(self) -> C {
        self.engine.into_parts().0
    }
}

impl<C> Transcoder for CodecTranscodeDecoder<C>
where
    C: Codec,
{
    /// Encoded units borrowed from the caller input.
    type Input = C::Unit;
    /// Decoded values written into caller-provided storage.
    type Output = C::Value;
    /// Contract or codec failures reported by the decoding engine.
    type Error = TranscodeDecodeErrorOf<C>;

    /// Returns an upper bound for decoded values produced from `input_len`
    /// units.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Source units the caller plans to decode.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound for decoded values.
    ///
    /// # Errors
    ///
    /// Returns a capacity error if the requested bound cannot fit in `usize`.
    #[inline]
    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        self.engine.max_transcode_output_len(input_len)
    }

    /// Returns the global maximum values emitted by finishing internal state.
    ///
    /// # Returns
    ///
    /// Returns the number of values that may still be emitted by finishing
    /// state.
    ///
    /// # Errors
    ///
    /// Returns a capacity error if the combined finish bound overflows.
    #[inline]
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        self.engine.max_finish_output_len()
    }

    /// Returns the global maximum values emitted when resetting internal state.
    ///
    /// # Returns
    ///
    /// The maximum reset output capacity required by the engine.
    ///
    /// # Errors
    ///
    /// Returns a capacity error if the combined reset bound overflows.
    #[inline]
    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        self.engine.max_reset_output_len()
    }

    /// Runs before-reset cleanup for decoder state.
    ///
    /// # Parameters
    ///
    /// - `output`: Destination storage for reset values.
    /// - `output_index`: First destination position to write.
    ///
    /// # Returns
    ///
    /// The number of reset values written.
    ///
    /// # Errors
    ///
    /// Returns a contract error for invalid indices or insufficient capacity,
    /// or a codec error when resetting codec state fails.
    fn reset(&mut self, output: &mut [C::Value], output_index: usize) -> Result<usize, TranscodeDecodeErrorOf<C>> {
        self.engine.reset(output, output_index)
    }

    /// Decodes source units into logical values.
    ///
    /// # Parameters
    ///
    /// - `input`: Source unit slice.
    /// - `input_index`: Absolute source index where decoding starts.
    /// - `output`: Destination value slice.
    /// - `output_index`: Absolute output value index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns conversion progress for consumed and written counters.
    ///
    /// # Errors
    ///
    /// Returns a decode error when indices are invalid or when conversion fails
    /// under hook policy.
    fn transcode(
        &mut self,
        input: &[C::Unit],
        input_index: usize,
        output: &mut [C::Value],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeDecodeErrorOf<C>> {
        self.engine.transcode(input, input_index, output, output_index)
    }

    /// Transcodes an input segment while marking it as the end of the stream.
    ///
    /// This forwarding implementation lets the engine apply EOF-specific
    /// decoder behavior before returning progress to the caller.
    ///
    /// # Parameters
    ///
    /// - `input`: Final source unit slice.
    /// - `input_index`: First source position to consume.
    /// - `output`: Destination value storage.
    /// - `output_index`: First destination position to write.
    ///
    /// # Returns
    ///
    /// Consumed and written counts and the engine progress status.
    ///
    /// # Errors
    ///
    /// Returns contract errors for invalid indices or lifecycle transitions,
    /// or decode errors for malformed or incomplete final input.
    fn transcode_eof(
        &mut self,
        input: &[C::Unit],
        input_index: usize,
        output: &mut [C::Value],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeDecodeErrorOf<C>> {
        self.engine.transcode_eof(input, input_index, output, output_index)
    }

    /// Finishes internally retained output after EOF.
    ///
    /// # Parameters
    ///
    /// - `output`: Destination value slice for final retained values.
    /// - `output_index`: Absolute output value index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of values written by finalization.
    ///
    /// # Errors
    ///
    /// Returns contract errors for invalid indices, insufficient capacity, or
    /// invalid lifecycle transitions, or a codec error for invalid EOF state.
    fn finish(&mut self, output: &mut [C::Value], output_index: usize) -> Result<usize, TranscodeDecodeErrorOf<C>> {
        self.engine.finish(output, output_index)
    }
}

impl<C> TranscodeDecoder for CodecTranscodeDecoder<C>
where
    C: Codec,
{
    /// Codec-specific failure retained by the decoder error wrapper.
    type DecodeError = C::DecodeError;
}
