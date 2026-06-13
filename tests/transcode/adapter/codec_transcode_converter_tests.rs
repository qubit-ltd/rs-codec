// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the codec-backed buffered converter adapter.

use qubit_codec::{
    CapacityError, Codec, CodecConvertError, CodecDecodeError, CodecEncodeError,
    CodecTranscodeConverter, TranscodeConverter, TranscodeError, TranscodeStatus, Transcoder,
};

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
struct VariableByteDecoder;

unsafe impl Codec for VariableByteDecoder {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        unsafe { core::num::NonZeroUsize::new_unchecked(2) }
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        let first = input[index];
        match first {
            0x80 => {
                let available = input.len() - index;
                if available < 2 {
                    Err(TestDecodeError::Incomplete {
                        required: 2,
                        available,
                    })
                } else {
                    Ok((input[index + 1], unsafe {
                        core::num::NonZeroUsize::new_unchecked(2)
                    }))
                }
            }
            0xff => Err(TestDecodeError::Invalid { consumed: 1 }),
            value => Ok((value, core::num::NonZeroUsize::MIN)),
        }
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index < output.len());

        output[index] = *value;
        Ok(qubit_codec::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
struct PairByteEncoder;

unsafe impl Codec for PairByteEncoder {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = TestEncodeError;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        unsafe { core::num::NonZeroUsize::new_unchecked(2) }
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        Ok((input[index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        if *value == 13 {
            return Err(TestEncodeError);
        }
        debug_assert!(index + 1 < output.len());

        output[index] = *value;
        output[index + 1] = value.wrapping_add(1);
        Ok(qubit_codec::nz!(2))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct MinTwoDecoder;

unsafe impl Codec for MinTwoDecoder {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::new(2).expect("literal is non-zero")
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        unsafe { core::num::NonZeroUsize::new_unchecked(2) }
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index + 1 < input.len());

        Ok((input[index].wrapping_add(input[index + 1]), unsafe {
            core::num::NonZeroUsize::new_unchecked(2)
        }))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index < output.len());

        output[index] = *value;
        Ok(qubit_codec::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TestDecodeError {
    Incomplete { required: usize, available: usize },
    Invalid { consumed: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TestEncodeError;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FlushValueDecoder;

unsafe impl Codec for FlushValueDecoder {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_decode_flush_values(&self) -> usize {
        1
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        Ok((input[index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index < output.len());

        output[index] = *value;
        Ok(qubit_codec::nz!(1))
    }

    unsafe fn decode_flush(
        &mut self,
        output: &mut [u8],
        index: usize,
    ) -> Result<usize, Self::DecodeError> {
        debug_assert!(index < output.len());

        output[index] = 9;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct NonDefaultValue(u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NonDefaultDecoder;

unsafe impl Codec for NonDefaultDecoder {
    type Value = NonDefaultValue;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(NonDefaultValue, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        Ok((NonDefaultValue(input[index]), core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &NonDefaultValue,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index < output.len());

        output[index] = value.0;
        Ok(qubit_codec::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NonDefaultEncoder;

unsafe impl Codec for NonDefaultEncoder {
    type Value = NonDefaultValue;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(NonDefaultValue, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        Ok((NonDefaultValue(input[index]), core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &NonDefaultValue,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index < output.len());

        output[index] = value.0.wrapping_add(1);
        Ok(qubit_codec::nz!(1))
    }
}

#[test]
fn test_codec_transcode_converter_supports_standard_traits() {
    let converter = CodecTranscodeConverter::<VariableByteDecoder, PairByteEncoder>::default();
    let cloned = converter.clone();

    assert_eq!(converter, cloned);
    assert_eq!(format!("{converter:?}"), format!("{cloned:?}"));

    let mut converter_hash = DefaultHasher::new();
    converter.hash(&mut converter_hash);
    let mut cloned_hash = DefaultHasher::new();
    cloned.hash(&mut cloned_hash);
    assert_eq!(converter_hash.finish(), cloned_hash.finish());
}

#[test]
fn test_codec_transcode_converter_transcodes_non_default_values_with_inherent_api() {
    type Converter = CodecTranscodeConverter<NonDefaultDecoder, NonDefaultEncoder>;

    fn assert_transcode_converter<T: TranscodeConverter<u8, u8>>() {}

    assert_transcode_converter::<Converter>();

    let mut converter = CodecTranscodeConverter::new(NonDefaultDecoder, NonDefaultEncoder);
    let mut output = [0_u8; 2];

    assert_eq!(Ok(2), converter.max_output_len(2));
    assert_eq!(Ok(0), converter.max_finish_output_len());

    let progress = converter
        .transcode(&[3, 4], 0, &mut output, 0)
        .expect("non-default values should transcode through inherent API");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(2, progress.read());
    assert_eq!(2, progress.written());
    assert_eq!([4, 5], output);

    converter.reset(&mut [], 0).expect("reset");
    assert_eq!(Ok(0), converter.finish(&mut output, 0));
}

#[test]
fn test_codec_transcode_converter_transcoder_trait_methods_forward() {
    type Converter = CodecTranscodeConverter<VariableByteDecoder, PairByteEncoder>;

    let mut converter = Converter::new(VariableByteDecoder, PairByteEncoder);
    let mut output = [0_u8; 2];

    assert_eq!(
        Ok(2),
        <Converter as Transcoder<u8, u8>>::max_output_len(&converter, 1)
    );
    assert_eq!(
        Ok(0),
        <Converter as Transcoder<u8, u8>>::max_finish_output_len(&converter),
    );

    let progress =
        <Converter as Transcoder<u8, u8>>::transcode(&mut converter, &[7], 0, &mut output, 0)
            .expect("trait transcoder dispatch should convert through the adapter");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(1, progress.read());
    assert_eq!(2, progress.written());
    assert_eq!([7, 8], output);

    <Converter as Transcoder<u8, u8>>::reset(&mut converter, &mut output, 0).expect("reset");
    assert_eq!(
        Ok(0),
        <Converter as Transcoder<u8, u8>>::finish(&mut converter, &mut output, 0),
    );
}

#[test]
fn test_codec_transcode_converter_converts_values_until_output_needs_capacity() {
    fn assert_transcode_converter<T: TranscodeConverter<u8, u8>>() {}

    assert_transcode_converter::<CodecTranscodeConverter<VariableByteDecoder, PairByteEncoder>>();

    let mut converter = CodecTranscodeConverter::<VariableByteDecoder, PairByteEncoder>::new(
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
            additional: crate::nz(2),
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
fn test_codec_transcode_converter_reports_bounds_and_finishes_noop() {
    let mut converter = CodecTranscodeConverter::<VariableByteDecoder, PairByteEncoder>::new(
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

    converter.reset(&mut [], 0).expect("reset");
    let written = converter
        .finish(&mut output, 0)
        .expect("codec converter has no finish output");
    assert_eq!(0, written);
}

#[test]
fn test_codec_transcode_converter_finish_encodes_decode_flush_values() {
    let mut converter = CodecTranscodeConverter::<FlushValueDecoder, PairByteEncoder>::new(
        FlushValueDecoder,
        PairByteEncoder,
    );
    let mut output = [0_u8; 2];

    assert_eq!(Ok(2), converter.max_finish_output_len());

    let written = converter
        .finish(&mut output, 0)
        .expect("finish should encode source decode-flush values");

    assert_eq!(2, written);
    assert_eq!([9, 10], output);
}

#[test]
fn test_codec_transcode_converter_wraps_variable_width_incomplete_decode_error() {
    let mut converter = CodecTranscodeConverter::<VariableByteDecoder, PairByteEncoder>::new(
        VariableByteDecoder,
        PairByteEncoder,
    );
    let mut output = [0_u8; 2];

    let error = converter
        .transcode(&[0x80], 0, &mut output, 0)
        .expect_err("strict converter should not classify decoder errors");
    assert_eq!(
        TranscodeError::Domain(CodecConvertError::Decode {
            source: CodecDecodeError::Decode {
                source: TestDecodeError::Incomplete {
                    required: 2,
                    available: 1,
                },
                input_index: 0,
            },
        }),
        error,
    );

    let progress = converter
        .transcode(&[0x80, 9], 0, &mut output, 0)
        .expect("caller-refilled input should complete conversion");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(2, progress.read());
    assert_eq!(2, progress.written());
    assert_eq!([9, 10], output);
}

#[test]
fn test_codec_transcode_converter_reports_short_minimum_input_without_consuming_tail() {
    let mut converter = CodecTranscodeConverter::<MinTwoDecoder, PairByteEncoder>::new(
        MinTwoDecoder,
        PairByteEncoder,
    );
    let mut output = [0_u8; 2];

    let progress = converter
        .transcode(&[7], 0, &mut output, 0)
        .expect("short input should request another unit");

    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            additional: crate::nz(1),
            available: 1,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_codec_transcode_converter_keeps_decoded_value_pending_when_output_is_short() {
    let mut converter = CodecTranscodeConverter::<VariableByteDecoder, PairByteEncoder>::new(
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
            additional: crate::nz(1),
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
fn test_codec_transcode_converter_finish_drains_pending_decoded_value() {
    let mut converter = CodecTranscodeConverter::<VariableByteDecoder, PairByteEncoder>::new(
        VariableByteDecoder,
        PairByteEncoder,
    );
    let mut short_output = [0_u8; 1];

    let progress = converter
        .transcode(&[7], 0, &mut short_output, 0)
        .expect("short output should retain the decoded value");
    assert!(matches!(
        progress.status(),
        TranscodeStatus::NeedOutput { .. }
    ));
    assert_eq!(1, progress.read());
    assert_eq!(0, progress.written());

    let mut output = [0_u8; 2];
    let written = converter
        .finish(&mut output, 0)
        .expect("finish should write the retained decoded value");

    assert_eq!(2, written);
    assert_eq!([7, 8], output);
}

#[test]
fn test_codec_transcode_converter_reports_invalid_indices() {
    let mut converter = CodecTranscodeConverter::<VariableByteDecoder, PairByteEncoder>::new(
        VariableByteDecoder,
        PairByteEncoder,
    );
    let mut output = [0_u8; 2];

    let error = converter
        .transcode(&[1], 2, &mut output, 0)
        .expect_err("invalid input index should fail");
    assert_eq!(
        TranscodeError::InvalidInputIndex { index: 2, len: 1 },
        error
    );

    let error = converter
        .transcode(&[1], 0, &mut output, 3)
        .expect_err("out-of-range output index should fail");
    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 3, len: 2 },
        error
    );
}

#[test]
fn test_codec_transcode_converter_wraps_decode_and_encode_errors() {
    let mut converter = CodecTranscodeConverter::<VariableByteDecoder, PairByteEncoder>::new(
        VariableByteDecoder,
        PairByteEncoder,
    );
    let mut output = [0_u8; 2];

    let error = converter
        .transcode(&[0xff], 0, &mut output, 0)
        .expect_err("invalid decode input should fail");
    assert_eq!(
        TranscodeError::Domain(CodecConvertError::Decode {
            source: CodecDecodeError::Decode {
                source: TestDecodeError::Invalid { consumed: 1 },
                input_index: 0,
            },
        }),
        error,
    );

    let error = converter
        .transcode(&[13], 0, &mut output, 0)
        .expect_err("unencodable value should fail");
    assert_eq!(
        TranscodeError::Domain(CodecConvertError::Encode {
            source: CodecEncodeError::Encode {
                source: TestEncodeError,
                input_index: 0,
            },
        }),
        error,
    );
}

#[test]
fn test_codec_transcode_converter_finish_does_not_handle_input_tail() {
    let mut converter = CodecTranscodeConverter::<MinTwoDecoder, PairByteEncoder>::new(
        MinTwoDecoder,
        PairByteEncoder,
    );
    let mut output = [0_u8; 2];

    let progress = converter
        .transcode(&[7], 0, &mut output, 0)
        .expect("partial value should not be retained");
    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            additional: crate::nz(1),
            available: 1,
        },
        progress.status(),
    );

    let written = converter
        .finish(&mut output, 0)
        .expect("codec converter has no finish output");

    assert_eq!(0, written);
}

#[test]
fn test_codec_transcode_converter_reports_max_reset_output_len() {
    let converter = CodecTranscodeConverter::<VariableByteDecoder, PairByteEncoder>::new(
        VariableByteDecoder,
        PairByteEncoder,
    );

    assert_eq!(Ok(0), converter.max_reset_output_len());
    assert_eq!(Ok(0), Transcoder::max_reset_output_len(&converter));
}

#[test]
fn test_codec_transcode_converter_finish_rejects_insufficient_output() {
    let mut converter = CodecTranscodeConverter::<VariableByteDecoder, PairByteEncoder>::new(
        VariableByteDecoder,
        PairByteEncoder,
    );
    let mut output = [0_u8; 4];

    converter
        .transcode(&[3, 5, 7], 0, &mut output, 0)
        .expect("conversion should fill output");

    let error = converter
        .finish(&mut output, 4)
        .expect_err("finish should reject insufficient output");

    assert_eq!(
        TranscodeError::InsufficientOutput {
            output_index: 4,
            required: 2,
            available: 0,
        },
        error,
    );
}
