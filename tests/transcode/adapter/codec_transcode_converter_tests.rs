// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the codec-backed buffered converter adapter.

use qubit_codec::{
    CapacityError, Codec, CodecTranscodeConverter, TranscodeConvertError, TranscodeConverter,
    TranscodeStatus, Transcoder,
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
struct VariableByteDecoder;

impl Codec for VariableByteDecoder {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index < input.len());

        let first = input[input_index];
        match first {
            0x80 => {
                let available = input.len() - input_index;
                if available < 2 {
                    Err(qubit_codec::DecodeFailure::incomplete(crate::nz(2)))
                } else {
                    Ok((input[input_index + 1], unsafe {
                        core::num::NonZeroUsize::new_unchecked(2)
                    }))
                }
            }
            0xff => Err(qubit_codec::DecodeFailure::invalid(
                TestDecodeError::Invalid { consumed: 1 },
                core::num::NonZeroUsize::MIN,
            )),
            value => Ok((value, core::num::NonZeroUsize::MIN)),
        }
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        debug_assert!(output_index < output.len());

        output[output_index] = *value;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
struct PairByteEncoder;

impl Codec for PairByteEncoder {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = TestEncodeError;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index < input.len());

        Ok((input[input_index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        if *value == 13 {
            return Err(TestEncodeError);
        }
        debug_assert!(output_index + 1 < output.len());

        output[output_index] = *value;
        output[output_index + 1] = value.wrapping_add(1);
        Ok(2)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
struct FlushFailDecoder;

impl Codec for FlushFailDecoder {
    type Value = u8;
    type Unit = u8;
    type DecodeError = &'static str;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_UNITS_PER_VALUE: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        Ok((input[input_index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(1)
    }

    unsafe fn decode_finish(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Err("flush failure")
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
struct ResetFailEncoder;

impl Codec for ResetFailEncoder {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = &'static str;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_RESET_UNITS: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        Ok((input[input_index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(1)
    }

    unsafe fn encode_reset(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Err("reset failure")
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct MinTwoDecoder;

impl Codec for MinTwoDecoder {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 2;

    const MAX_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index + 1 < input.len());

        Ok((
            input[input_index].wrapping_add(input[input_index + 1]),
            unsafe { core::num::NonZeroUsize::new_unchecked(2) },
        ))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        debug_assert!(output_index < output.len());

        output[output_index] = *value;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TestDecodeError {
    Invalid { consumed: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TestEncodeError;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FlushValueDecoder;

impl Codec for FlushValueDecoder {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_FINISH_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index < input.len());

        Ok((input[input_index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        debug_assert!(output_index < output.len());

        output[output_index] = *value;
        Ok(1)
    }

    unsafe fn decode_finish(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        debug_assert!(output_index < output.len());

        output[output_index] = 9;
        Ok(1)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct NonDefaultValue(u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NonDefaultDecoder;

impl Codec for NonDefaultDecoder {
    type Value = NonDefaultValue;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_UNITS_PER_VALUE: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (NonDefaultValue, core::num::NonZeroUsize),
        qubit_codec::DecodeFailure<Self::DecodeError>,
    > {
        debug_assert!(input_index < input.len());

        Ok((
            NonDefaultValue(input[input_index]),
            core::num::NonZeroUsize::MIN,
        ))
    }

    unsafe fn encode(
        &mut self,
        value: &NonDefaultValue,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        debug_assert!(output_index < output.len());

        output[output_index] = value.0;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NonDefaultEncoder;

impl Codec for NonDefaultEncoder {
    type Value = NonDefaultValue;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_UNITS_PER_VALUE: usize = 1;

    fn can_encode_value(&self, value: &NonDefaultValue) -> bool {
        value.0 != 13
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (NonDefaultValue, core::num::NonZeroUsize),
        qubit_codec::DecodeFailure<Self::DecodeError>,
    > {
        debug_assert!(input_index < input.len());

        Ok((
            NonDefaultValue(input[input_index]),
            core::num::NonZeroUsize::MIN,
        ))
    }

    unsafe fn encode(
        &mut self,
        value: &NonDefaultValue,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        debug_assert!(output_index < output.len());

        output[output_index] = value.0.wrapping_add(1);
        Ok(1)
    }
}

#[test]
fn test_codec_transcode_converter_supports_debug_and_default() {
    let converter = CodecTranscodeConverter::<VariableByteDecoder, PairByteEncoder>::default();

    assert!(format!("{converter:?}").contains("CodecTranscodeConverter"));
}

#[test]
fn test_codec_transcode_converter_transcodes_non_clone_values_with_inherent_api() {
    type Converter = CodecTranscodeConverter<NonDefaultDecoder, NonDefaultEncoder>;

    fn assert_transcode_converter<T: TranscodeConverter<Input = u8, Output = u8>>() {}

    assert_transcode_converter::<Converter>();

    let mut converter = CodecTranscodeConverter::new(NonDefaultDecoder, NonDefaultEncoder);
    let mut output = [0_u8; 2];

    assert_eq!(Ok(3), converter.max_transcode_output_len(2));
    assert_eq!(Ok(1), converter.max_finish_output_len());

    let progress = converter
        .transcode(&[3, 4], 0, &mut output, 0)
        .expect("non-clone values should transcode through inherent API");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(2, progress.read());
    assert_eq!(2, progress.written());
    assert_eq!([4, 5], output);

    converter.reset(&mut [], 0).expect("reset");
    assert_eq!(Ok(0), converter.finish(&mut output, 0));
    converter
        .reset(&mut [], 0)
        .expect("reset should start the unencodable-value stream");

    let error = converter
        .transcode(&[13], 0, &mut output, 0)
        .expect_err("unencodable non-clone value should retain owned context");
    assert_eq!(
        TranscodeConvertError::unencodable(0, NonDefaultValue(13)),
        error,
    );
}

#[test]
fn test_codec_transcode_converter_transcoder_trait_methods_forward() {
    type Converter = CodecTranscodeConverter<VariableByteDecoder, PairByteEncoder>;

    let mut converter = Converter::new(VariableByteDecoder, PairByteEncoder);
    let mut output = [0_u8; 2];

    assert_eq!(
        Ok(4),
        <Converter as Transcoder>::max_transcode_output_len(&converter, 1)
    );
    assert_eq!(
        Ok(2),
        <Converter as Transcoder>::max_finish_output_len(&converter),
    );

    let progress = <Converter as Transcoder>::transcode(&mut converter, &[7], 0, &mut output, 0)
        .expect("trait transcoder dispatch should convert through the adapter");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(1, progress.read());
    assert_eq!(2, progress.written());
    assert_eq!([7, 8], output);

    <Converter as Transcoder>::reset(&mut converter, &mut output, 0).expect("reset");
    assert_eq!(
        Ok(0),
        <Converter as Transcoder>::finish(&mut converter, &mut output, 0),
    );
}

#[test]
fn test_codec_transcode_converter_converts_values_until_output_needs_capacity() {
    fn assert_transcode_converter<T: TranscodeConverter<Input = u8, Output = u8>>() {}

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
            required: crate::nz(2),
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

    assert_eq!(Ok(8), converter.max_transcode_output_len(3));
    assert_eq!(Ok(2), converter.max_finish_output_len());
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        converter.max_transcode_output_len(usize::MAX),
    );

    converter.reset(&mut [], 0).expect("reset");
    let written = converter
        .finish(&mut output, 0)
        .expect("codec converter has no finish output");
    assert_eq!(0, written);
}

#[test]
fn test_codec_transcode_converter_finish_encodes_decode_finish_values() {
    let mut converter = CodecTranscodeConverter::<FlushValueDecoder, PairByteEncoder>::new(
        FlushValueDecoder,
        PairByteEncoder,
    );
    let mut output = [0_u8; 2];

    assert_eq!(Ok(4), converter.max_finish_output_len());

    let written = converter
        .finish(&mut output, 0)
        .expect("finish should encode source decode-flush values");

    assert_eq!(2, written);
    assert_eq!([9, 10], output);
}

#[test]
fn test_codec_transcode_converter_reports_variable_width_incomplete_input() {
    let mut converter = CodecTranscodeConverter::<VariableByteDecoder, PairByteEncoder>::new(
        VariableByteDecoder,
        PairByteEncoder,
    );
    let mut output = [0_u8; 2];

    let progress = converter
        .transcode(&[0x80], 0, &mut output, 0)
        .expect("strict converter should classify incomplete input");
    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            required: crate::nz(2),
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
            required: crate::nz(2),
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
            required: crate::nz(2),
            available: 1,
        },
        progress.status(),
    );
    assert_eq!(1, progress.read());
    assert_eq!(0, progress.written());
    assert_eq!([0], output);
    assert_eq!(Ok(8), converter.max_transcode_output_len(3));

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
    assert_eq!(TranscodeConvertError::invalid_input_index(2, 1), error);

    let error = converter
        .transcode(&[1], 0, &mut output, 3)
        .expect_err("out-of-range output index should fail");
    assert_eq!(TranscodeConvertError::invalid_output_index(3, 2), error);
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
        TranscodeConvertError::decode_domain_main_with_consumed(
            TestDecodeError::Invalid { consumed: 1 },
            0,
            Some(crate::nz(1)),
        ),
        error,
    );

    let error = converter
        .transcode(&[13], 0, &mut output, 0)
        .expect_err("unencodable value should fail");
    assert_eq!(
        TranscodeConvertError::encode_domain_main(TestEncodeError, 0,),
        error,
    );
}

#[test]
fn test_codec_transcode_converter_wraps_decode_finish_error() {
    let mut converter = CodecTranscodeConverter::<FlushFailDecoder, PairByteEncoder>::new(
        FlushFailDecoder,
        PairByteEncoder,
    );
    let mut output = [];

    let error = converter
        .finish(&mut output, 0)
        .expect_err("decode flush errors should be flattened");

    assert_eq!(
        TranscodeConvertError::decode_domain_finish("flush failure"),
        error,
    );
}

#[test]
fn test_codec_transcode_converter_wraps_encode_reset_error() {
    let mut converter = CodecTranscodeConverter::<VariableByteDecoder, ResetFailEncoder>::new(
        VariableByteDecoder,
        ResetFailEncoder,
    );
    let mut output = [0_u8; 1];

    let error = converter
        .reset(&mut output, 0)
        .expect_err("encode reset errors should be flattened");

    assert_eq!(
        TranscodeConvertError::encode_domain_reset("reset failure"),
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
            required: crate::nz(2),
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

    assert_eq!(TranscodeConvertError::insufficient_output(4, 2, 0), error,);
}
