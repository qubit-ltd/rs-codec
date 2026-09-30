// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Semantic marker trait for buffered encoders.

use super::TranscodeEncodeError;
use super::Transcoder;

/// Encodes logical values into encoded units over caller-provided buffers.
///
/// `TranscodeEncoder` refines [`Transcoder`] for implementations whose
/// input is the logical value stream and whose output is the encoded unit
/// stream. The trait adds no methods; it exists to make generic bounds
/// distinguish encoding direction from decoding and unit-to-unit conversion.
///
/// The word "buffered" describes the caller-managed buffer and progress model.
/// It does not require the implementor to own an internal buffer. An encoder
/// may retain consumed input internally, but [`Transcoder::transcode`] must
/// consume every visible input value before returning
/// [`crate::TranscodeStatus::Complete`] and must never return
/// [`crate::TranscodeStatus::NeedInput`]. It returns
/// [`crate::TranscodeStatus::NeedOutput`] when more output capacity is needed;
/// [`Transcoder::finish`] emits any retained output after the caller has
/// supplied the complete logical input stream.
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
/// use qubit_codec::TranscodeEncoder;
/// use qubit_codec::Transcoder;
///
/// struct ByteCodec;
/// impl Codec for ByteCodec {
///     type Value = u8;
///     type Unit = u8;
///     type DecodeError = Infallible;
///     type EncodeError = Infallible;
///     const MIN_UNITS_PER_VALUE: usize = 1;
///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
///
///     unsafe fn decode(&mut self, input: &[u8], index: usize)
///         -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>>
///     {
///         Ok((input[index], NonZeroUsize::MIN))
///     }
///
///     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
///         -> Result<usize, Infallible>
///     {
///         output[index] = *value;
///         Ok(1)
///     }
/// }
///
/// fn accepts_encoder<E: TranscodeEncoder>(_: &E) {}
/// let mut encoder = CodecTranscodeEncoder::new(ByteCodec);
/// accepts_encoder(&encoder);
/// let mut output = [0; 2];
/// let written = encoder.transcode_complete_into(&[1, 2], &mut output).unwrap();
/// assert_eq!(written, 2);
/// assert_eq!(output, [1, 2]);
/// ```
pub trait TranscodeEncoder:
    Transcoder<Error = TranscodeEncodeError<Self::EncodeError, <Self as Transcoder>::Input>>
{
    /// Domain error type produced by encode internals.
    type EncodeError;
}
