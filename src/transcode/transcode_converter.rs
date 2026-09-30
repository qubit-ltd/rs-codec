// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Semantic marker trait for buffered converters.

use super::TranscodeConvertError;
use super::Transcoder;

/// Converts encoded units of one representation into encoded units of another.
///
/// `TranscodeConverter` refines [`Transcoder`] for implementations whose
/// input and output are both encoded unit streams. Any intermediate logical
/// values are implementation details of the concrete converter.
///
/// The trait adds no methods. It exists to make generic bounds distinguish
/// unit-to-unit conversion from value-to-unit encoding and unit-to-value
/// decoding.
///
/// # Examples
///
/// ```
/// use core::convert::Infallible;
/// use core::num::NonZeroUsize;
///
/// use qubit_codec::CapacityError;
/// use qubit_codec::TranscodeConvertError;
/// use qubit_codec::TranscodeConverter;
/// use qubit_codec::TranscodeProgress;
/// use qubit_codec::Transcoder;
///
/// #[derive(Default)]
/// struct ByteToWord;
/// # impl Transcoder for ByteToWord {
/// #     type Input = u8;
/// #     type Output = u16;
/// #     type Error = TranscodeConvertError<Infallible, Infallible, u8>;
/// #     fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> { Ok(input_len) }
/// #     fn reset(&mut self, _output: &mut [u16], _index: usize) -> Result<usize, Self::Error> { Ok(0) }
/// #     fn finish(&mut self, _output: &mut [u16], _index: usize) -> Result<usize, Self::Error> { Ok(0) }
/// #     fn transcode(&mut self, input: &[u8], input_index: usize, output: &mut [u16], output_index: usize) -> Result<TranscodeProgress, Self::Error> {
/// #         let source = &input[input_index..];
/// #         let target = &mut output[output_index..];
/// #         let count = source.len().min(target.len());
/// #         for (input, output) in source.iter().zip(target.iter_mut()) { *output = u16::from(*input); }
/// #         if count == source.len() { Ok(TranscodeProgress::complete(count, count)) }
/// #         else { Ok(TranscodeProgress::need_output(NonZeroUsize::MIN, count, count)) }
/// #     }
/// # }
/// impl TranscodeConverter for ByteToWord {
///     type DecodeError = Infallible;
///     type EncodeError = Infallible;
///     type Value = u8;
/// }
///
/// let mut converter = ByteToWord::default();
/// let mut output = [0_u16; 2];
/// let progress = converter.transcode(&[7, 9], 0, &mut output, 0).unwrap();
/// assert_eq!(output, [7, 9]);
/// assert_eq!(progress, TranscodeProgress::complete(2, 2));
/// ```
pub trait TranscodeConverter:
    Transcoder<Error = TranscodeConvertError<Self::DecodeError, Self::EncodeError, Self::Value>>
{
    /// Domain error type produced by source decoding.
    type DecodeError;

    /// Domain error type produced by target encoding.
    type EncodeError;

    /// Intermediate logical value type converted between source and target.
    type Value;
}
