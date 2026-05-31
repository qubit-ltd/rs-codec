/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for the codec-backed buffered converter adapter.

use qubit_codec::{
    BufferedConverter,
    CapacityError,
    Codec,
    CodecBufferedConverter,
    CodecConvertError,
    CodecDecodeError,
    DecodeErrorInfo,
    DecodeFailure,
    TranscodeStatus,
    Transcoder,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct VariableByteDecoder;

unsafe impl Codec<u8, u8> for VariableByteDecoder {
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        unsafe { core::num::NonZeroUsize::new_unchecked(2) }
    }

    unsafe fn decode_unchecked(
        &self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        let first = input[index];
        match first {
            0x80 => {
                let available = input.len() - index;
                if available < 2 {
                    Err(TestDecodeError::Incomplete { required: 2, available })
                } else {
                    Ok((input[index + 1], unsafe { core::num::NonZeroUsize::new_unchecked(2) }))
                }
            }
            0xff => Err(TestDecodeError::Invalid { consumed: 1 }),
            value => Ok((value, core::num::NonZeroUsize::MIN)),
        }
    }

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        debug_assert!(index < output.len());

        output[index] = *value;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct PairByteEncoder;

unsafe impl Codec<u8, u8> for PairByteEncoder {
    type DecodeError = core::convert::Infallible;
    type EncodeError = TestEncodeError;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        unsafe { core::num::NonZeroUsize::new_unchecked(2) }
    }

    unsafe fn decode_unchecked(
        &self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        Ok((input[index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        if *value == 13 {
            return Err(TestEncodeError);
        }
        debug_assert!(index + 1 < output.len());

        output[index] = *value;
        output[index + 1] = value.wrapping_add(1);
        Ok(2)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct MinTwoDecoder;

unsafe impl Codec<u8, u8> for MinTwoDecoder {
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::new(2).expect("literal is non-zero")
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        unsafe { core::num::NonZeroUsize::new_unchecked(2) }
    }

    unsafe fn decode_unchecked(
        &self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index + 1 < input.len());

        Ok((input[index].wrapping_add(input[index + 1]), unsafe {
            core::num::NonZeroUsize::new_unchecked(2)
        }))
    }

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        debug_assert!(index < output.len());

        output[index] = *value;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TestDecodeError {
    Incomplete { required: usize, available: usize },
    Invalid { consumed: usize },
}

impl DecodeErrorInfo for TestDecodeError {
    fn failure(&self) -> DecodeFailure {
        match self {
            Self::Incomplete { required, available } => DecodeFailure::Incomplete {
                required_total: *required,
                available: *available,
            },
            Self::Invalid { consumed } => DecodeFailure::Invalid { consumed: *consumed },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TestEncodeError;

#[test]
fn test_codec_buffered_converter_converts_values_until_output_needs_capacity() {
    fn assert_buffered_converter<T: BufferedConverter<u8, u8>>() {}

    assert_buffered_converter::<CodecBufferedConverter<VariableByteDecoder, PairByteEncoder, u8, u8>>();

    let mut converter = CodecBufferedConverter::<VariableByteDecoder, PairByteEncoder, u8, u8>::new(
        VariableByteDecoder,
        PairByteEncoder,
    );
    let mut output = [0_u8; 4];

    let progress = converter
        .transcode(&[3, 5, 7], 0, &mut output, 0)
        .expect("conversion should succeed until output fills");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 4,
            additional: 2,
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(3, progress.read());
    assert_eq!(4, progress.written());
    assert_eq!([3, 4, 5, 6], output);
    assert_eq!(Ok(2), converter.max_finish_output_len());
}

#[test]
fn test_codec_buffered_converter_reports_bounds_and_finishes_noop() {
    let mut converter = CodecBufferedConverter::<VariableByteDecoder, PairByteEncoder, u8, u8>::new(
        VariableByteDecoder,
        PairByteEncoder,
    );
    let mut output = [0_u8; 2];

    assert_eq!(Ok(6), converter.max_output_len(3));
    assert_eq!(Ok(0), converter.max_finish_output_len());
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        converter.max_output_len(usize::MAX),
    );

    converter.reset();
    let finish = converter
        .finish(&mut output, 0)
        .expect("codec converter has no finish output");
    assert_eq!(TranscodeStatus::Complete, finish.status());
    assert_eq!(0, finish.read());
    assert_eq!(0, finish.written());
}

#[test]
fn test_codec_buffered_converter_leaves_incomplete_input_to_caller() {
    let mut converter = CodecBufferedConverter::<VariableByteDecoder, PairByteEncoder, u8, u8>::new(
        VariableByteDecoder,
        PairByteEncoder,
    );
    let mut output = [0_u8; 2];

    let progress = converter
        .transcode(&[0x80], 0, &mut output, 0)
        .expect("partial value should request input");
    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            additional: 1,
            available: 1,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());

    let progress = converter
        .transcode(&[0x80, 9], 0, &mut output, 0)
        .expect("caller-refilled input should complete conversion");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(2, progress.read());
    assert_eq!(2, progress.written());
    assert_eq!([9, 10], output);
}

#[test]
fn test_codec_buffered_converter_reports_short_minimum_input_without_consuming_tail() {
    let mut converter =
        CodecBufferedConverter::<MinTwoDecoder, PairByteEncoder, u8, u8>::new(MinTwoDecoder, PairByteEncoder);
    let mut output = [0_u8; 2];

    let progress = converter
        .transcode(&[7], 0, &mut output, 0)
        .expect("short input should request another unit");

    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            additional: 1,
            available: 1,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_codec_buffered_converter_keeps_decoded_value_pending_when_output_is_short() {
    let mut converter = CodecBufferedConverter::<VariableByteDecoder, PairByteEncoder, u8, u8>::new(
        VariableByteDecoder,
        PairByteEncoder,
    );
    let mut output = [0_u8; 1];

    let progress = converter
        .transcode(&[3], 0, &mut output, 0)
        .expect("short output should retain the decoded value");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: 1,
            available: 1,
        },
        progress.status(),
    );
    assert_eq!(1, progress.read());
    assert_eq!(0, progress.written());
    assert_eq!([0], output);
    assert_eq!(Ok(8), converter.max_output_len(3));

    let mut output = [0_u8; 2];
    let progress = converter
        .transcode(&[], 0, &mut output, 0)
        .expect("pending value should be written before new input");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(0, progress.read());
    assert_eq!(2, progress.written());
    assert_eq!([3, 4], output);
}

#[test]
fn test_codec_buffered_converter_finish_drains_pending_decoded_value() {
    let mut converter = CodecBufferedConverter::<VariableByteDecoder, PairByteEncoder, u8, u8>::new(
        VariableByteDecoder,
        PairByteEncoder,
    );
    let mut short_output = [0_u8; 1];

    let progress = converter
        .transcode(&[7], 0, &mut short_output, 0)
        .expect("short output should retain the decoded value");
    assert!(matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));
    assert_eq!(1, progress.read());
    assert_eq!(0, progress.written());

    let mut output = [0_u8; 2];
    let finish = converter
        .finish(&mut output, 0)
        .expect("finish should write the retained decoded value");

    assert_eq!(TranscodeStatus::Complete, finish.status());
    assert_eq!(0, finish.read());
    assert_eq!(2, finish.written());
    assert_eq!([7, 8], output);
}

#[test]
fn test_codec_buffered_converter_reports_invalid_indices() {
    let mut converter = CodecBufferedConverter::<VariableByteDecoder, PairByteEncoder, u8, u8>::new(
        VariableByteDecoder,
        PairByteEncoder,
    );
    let mut output = [0_u8; 2];

    let error = converter
        .transcode(&[1], 2, &mut output, 0)
        .expect_err("invalid input index should fail");
    assert_eq!(
        CodecConvertError::Decode {
            source: CodecDecodeError::InvalidInputIndex { index: 2, len: 1 },
        },
        error,
    );

    let progress = converter
        .transcode(&[1], 0, &mut output, 3)
        .expect("out-of-range output index should request capacity");
    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 3,
            additional: 2,
            available: 0,
        },
        progress.status(),
    );
}

#[test]
fn test_codec_buffered_converter_wraps_decode_and_encode_errors() {
    let mut converter = CodecBufferedConverter::<VariableByteDecoder, PairByteEncoder, u8, u8>::new(
        VariableByteDecoder,
        PairByteEncoder,
    );
    let mut output = [0_u8; 2];

    let error = converter
        .transcode(&[0xff], 0, &mut output, 0)
        .expect_err("invalid decode input should fail");
    assert_eq!(
        CodecConvertError::Decode {
            source: CodecDecodeError::Decode {
                source: TestDecodeError::Invalid { consumed: 1 },
                input_index: 0,
            },
        },
        error,
    );

    let error = converter
        .transcode(&[13], 0, &mut output, 0)
        .expect_err("unencodable value should fail");
    assert_eq!(
        CodecConvertError::Encode {
            source: TestEncodeError,
        },
        error,
    );
}

#[test]
fn test_codec_buffered_converter_finish_does_not_handle_input_tail() {
    let mut converter = CodecBufferedConverter::<VariableByteDecoder, PairByteEncoder, u8, u8>::new(
        VariableByteDecoder,
        PairByteEncoder,
    );
    let mut output = [0_u8; 2];

    let progress = converter
        .transcode(&[0x80], 0, &mut output, 0)
        .expect("partial value should not be retained");
    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            additional: 1,
            available: 1,
        },
        progress.status(),
    );

    let finish = converter
        .finish(&mut output, 0)
        .expect("codec converter has no finish output");

    assert_eq!(TranscodeStatus::Complete, finish.status());
    assert_eq!(0, finish.read());
    assert_eq!(0, finish.written());
}
