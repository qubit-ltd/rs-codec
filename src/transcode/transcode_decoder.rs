// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Semantic marker trait for buffered decoders.

use super::TranscodeDecodeError;
use super::Transcoder;

/// Decodes encoded units into logical values over caller-provided buffers.
///
/// `TranscodeDecoder` refines [`Transcoder`] for implementations whose
/// input is the encoded unit stream and whose output is the logical value
/// stream. The trait adds no methods; it exists to make generic bounds
/// distinguish decoding direction from encoding and unit-to-unit conversion.
///
/// The word "buffered" describes the caller-managed buffer and progress model.
/// It does not require the implementor to own an internal buffer.
///
/// # Examples
///
/// ```
/// use core::convert::Infallible;
/// use core::num::NonZeroUsize;
///
/// use qubit_codec::Codec;
/// use qubit_codec::CodecTranscodeDecoder;
/// use qubit_codec::DecodeFailure;
/// use qubit_codec::TranscodeDecoder;
/// use qubit_codec::Transcoder;
///
/// # struct IdentityCodec;
/// # impl Codec for IdentityCodec {
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
/// fn accepts_decoder<D: TranscodeDecoder>(_: &D) {}
/// let mut decoder = CodecTranscodeDecoder::new(IdentityCodec);
/// accepts_decoder(&decoder);
/// let mut values = [0_u8; 1];
/// decoder.reset(&mut values, 0).unwrap();
/// let progress = decoder.transcode(&[7_u8], 0, &mut values, 0).unwrap();
/// assert_eq!(progress.read(), 1);
/// assert_eq!(progress.written(), 1);
/// assert_eq!(values, [7]);
/// ```
pub trait TranscodeDecoder: Transcoder<Error = TranscodeDecodeError<Self::DecodeError>> {
    /// Domain error type produced by decode internals.
    type DecodeError;
}
