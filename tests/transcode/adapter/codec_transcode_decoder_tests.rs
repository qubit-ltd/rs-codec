// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the codec-backed buffered decoder adapter.

use qubit_codec::{
    Codec, CodecTranscodeDecoder, DecodeFailure, TranscodeDecodeError, TranscodeDecoder,
    TranscodeStatus, Transcoder,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct VariableByteCodec;

impl Codec for VariableByteCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index < input.len());

        let first = input[input_index];
        match first {
            0x80 => {
                let available = input.len() - input_index;
                if available < 2 {
                    Err(DecodeFailure::incomplete(crate::nz(2)))
                } else {
                    Ok((input[input_index + 1], unsafe {
                        core::num::NonZeroUsize::new_unchecked(2)
                    }))
                }
            }
            0xff => Err(DecodeFailure::invalid(
                TestDecodeError::Invalid,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TestDecodeError {
    Invalid,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FixedPairCodec;

impl Codec for FixedPairCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 2;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index + 1 < input.len());

        Ok((
            input[input_index].wrapping_add(input[input_index + 1]),
            crate::nz(2),
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FlushFailCodec;

impl Codec for FlushFailCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = &'static str;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>> {
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetFailCodec;

impl Codec for ResetFailCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = &'static str;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>> {
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

    unsafe fn decode_reset(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Err("reset failure")
    }
}

#[test]
fn test_codec_transcode_decoder_decodes_until_output_needs_capacity() {
    fn assert_transcode_decoder<T: TranscodeDecoder<Input = u8, Output = u8>>() {}

    assert_transcode_decoder::<CodecTranscodeDecoder<VariableByteCodec>>();

    let mut decoder = CodecTranscodeDecoder::new(VariableByteCodec);

    let mut reset_output = [];

    decoder
        .reset(&mut reset_output, 0)
        .expect("initialize stream");

    let mut output = [0_u8; 2];

    let progress = decoder
        .transcode(&[1, 2, 3], 0, &mut output, 0)
        .expect("input should decode");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            required: crate::nz(1),
        },
        progress.status(),
    );
    assert_eq!(2, progress.read());
    assert_eq!(2, progress.written());
    assert_eq!([1, 2], output);
}

#[test]
fn test_codec_transcode_decoder_does_not_decode_after_output_is_full() {
    let mut decoder = CodecTranscodeDecoder::new(VariableByteCodec);
    let mut reset_output = [];
    decoder
        .reset(&mut reset_output, 0)
        .expect("initialize stream");

    let mut output = [0_u8; 1];

    let progress = decoder
        .transcode(&[1, 0xff], 0, &mut output, 0)
        .expect("full output should stop before invalid trailing input");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            required: crate::nz(1),
        },
        progress.status(),
    );
    assert_eq!(1, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([1], output);
}

#[test]
fn test_codec_transcode_decoder_reports_bounds_and_resets_state() {
    let mut decoder = CodecTranscodeDecoder::new(VariableByteCodec);
    let mut output = [0_u8; 1];

    assert_eq!(Ok(3), decoder.max_transcode_output_len(3));
    assert_eq!(Ok(0), decoder.max_finish_output_len());

    decoder.reset(&mut [], 0).expect("reset");
    let written = decoder
        .finish(&mut output, 0)
        .expect("codec decoder has no finish output");
    assert_eq!(0, written);
}

#[test]
fn test_codec_transcode_decoder_reports_variable_width_incomplete_input() {
    let mut decoder = CodecTranscodeDecoder::new(VariableByteCodec);
    let mut reset_output = [];
    decoder
        .reset(&mut reset_output, 0)
        .expect("initialize stream");

    let mut output = [0_u8; 1];

    let progress = decoder
        .transcode(&[0x80], 0, &mut output, 0)
        .expect("incomplete codec failure should request more input");
    assert_eq!(
        TranscodeStatus::NeedInput {
            required: crate::nz(2),
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());

    let progress = decoder
        .transcode(&[0x80, 9], 0, &mut output, 0)
        .expect("caller-refilled input should complete");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(2, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([9], output);
}

#[test]
fn test_codec_transcode_decoder_reports_output_index_beyond_buffer() {
    let mut decoder = CodecTranscodeDecoder::new(VariableByteCodec);
    let mut reset_output = [];
    decoder
        .reset(&mut reset_output, 0)
        .expect("initialize stream");

    let mut output = [];

    let error = decoder
        .transcode(&[1], 0, &mut output, 1)
        .expect_err("out-of-range output index should fail");

    assert_eq!(TranscodeDecodeError::invalid_output_index(1, 0), error);
}

#[test]
fn test_codec_transcode_decoder_reports_input_index_beyond_buffer() {
    let mut decoder = CodecTranscodeDecoder::new(VariableByteCodec);
    let mut reset_output = [];
    decoder
        .reset(&mut reset_output, 0)
        .expect("initialize stream");

    let mut output = [0_u8; 1];

    let error = decoder
        .transcode(&[1], 2, &mut output, 0)
        .expect_err("out-of-range input index should fail");

    assert_eq!(TranscodeDecodeError::invalid_input_index(2, 1), error);
}

#[test]
fn test_codec_transcode_decoder_finish_reports_output_index_beyond_buffer() {
    let mut decoder = CodecTranscodeDecoder::new(VariableByteCodec);
    let mut reset_output = [];
    decoder
        .reset(&mut reset_output, 0)
        .expect("initialize stream");

    let mut output = [];

    let error = decoder
        .finish(&mut output, 1)
        .expect_err("out-of-range finish output index should be rejected");

    assert_eq!(TranscodeDecodeError::invalid_output_index(1, 0), error);
}

#[test]
fn test_codec_transcode_decoder_finish_does_not_handle_input_tail() {
    let mut decoder = CodecTranscodeDecoder::new(FixedPairCodec);
    let mut reset_output = [];
    decoder
        .reset(&mut reset_output, 0)
        .expect("initialize stream");

    let mut output = [0_u8; 1];

    let progress = decoder
        .transcode(&[7], 0, &mut output, 0)
        .expect("partial input should not be retained");
    assert_eq!(
        TranscodeStatus::NeedInput {
            required: crate::nz(2),
        },
        progress.status(),
    );

    let written = decoder
        .finish(&mut output, 0)
        .expect("codec decoder has no finish output");

    assert_eq!(0, written);
}

#[test]
fn test_codec_transcode_decoder_wraps_invalid_codec_error() {
    let mut decoder = CodecTranscodeDecoder::new(VariableByteCodec);
    let mut reset_output = [];
    decoder
        .reset(&mut reset_output, 0)
        .expect("initialize stream");

    let mut output = [0_u8; 1];

    let error = decoder
        .transcode(&[0xff], 0, &mut output, 0)
        .expect_err("invalid input should fail");

    assert_eq!(
        TranscodeDecodeError::domain_main_with_consumed(
            TestDecodeError::Invalid,
            0,
            Some(crate::nz(1)),
        ),
        error,
    );
}

#[test]
fn test_codec_transcode_decoder_wraps_decode_finish_error() {
    let mut decoder = CodecTranscodeDecoder::new(FlushFailCodec);
    let mut reset_output = [];
    decoder
        .reset(&mut reset_output, 0)
        .expect("initialize stream");

    let mut output = [];

    let error = decoder
        .finish(&mut output, 0)
        .expect_err("decode flush errors should be flattened");

    assert_eq!(TranscodeDecodeError::domain_finish("flush failure"), error,);
}

#[test]
fn test_codec_transcode_decoder_wraps_decode_reset_error() {
    let mut decoder = CodecTranscodeDecoder::new(ResetFailCodec);

    let error = decoder
        .reset(&mut [], 0)
        .expect_err("decode reset errors should be flattened");

    assert_eq!(TranscodeDecodeError::domain_reset("reset failure"), error,);
}

#[test]
fn test_codec_transcode_decoder_reports_max_reset_output_len() {
    let decoder = CodecTranscodeDecoder::<FixedPairCodec>::new(FixedPairCodec);

    assert_eq!(Ok(0), Transcoder::max_reset_output_len(&decoder));
}
