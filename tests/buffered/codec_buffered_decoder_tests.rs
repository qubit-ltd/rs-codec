/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for the codec-backed buffered decoder adapter.

use qubit_codec::{
    BufferedDecoder,
    Codec,
    CodecBufferedDecoder,
    CodecDecodeError,
    TranscodeStatus,
    Transcoder,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct VariableByteCodec;

unsafe impl Codec<u8, u8> for VariableByteCodec {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TestDecodeError {
    Incomplete { required: usize, available: usize },
    Invalid { consumed: usize },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FixedPairCodec;

unsafe impl Codec<u8, u8> for FixedPairCodec {
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::new(2).expect("literal is non-zero")
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::new(2).expect("literal is non-zero")
    }

    unsafe fn decode_unchecked(
        &self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index + 1 < input.len());

        Ok((
            input[index].wrapping_add(input[index + 1]),
            core::num::NonZeroUsize::new(2).expect("literal is non-zero"),
        ))
    }

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        debug_assert!(index < output.len());

        output[index] = *value;
        Ok(1)
    }
}

#[test]
fn test_codec_buffered_decoder_decodes_until_output_needs_capacity() {
    fn assert_buffered_decoder<T: BufferedDecoder<u8, u8>>() {}

    assert_buffered_decoder::<CodecBufferedDecoder<VariableByteCodec, u8>>();

    let mut decoder = CodecBufferedDecoder::new(VariableByteCodec);
    let mut output = [0_u8; 2];

    let progress = decoder
        .transcode(&[1, 2, 3], 0, &mut output, 0)
        .expect("input should decode");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 2,
            additional: super::nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(2, progress.read());
    assert_eq!(2, progress.written());
    assert_eq!([1, 2], output);
}

#[test]
fn test_codec_buffered_decoder_reports_bounds_and_resets_state() {
    let mut decoder = CodecBufferedDecoder::new(VariableByteCodec);
    let mut output = [0_u8; 1];

    assert_eq!(Ok(3), decoder.max_output_len(3));
    assert_eq!(Ok(0), decoder.max_finish_output_len());

    decoder.reset();
    let finish = decoder
        .finish(&mut output, 0)
        .expect("codec decoder has no finish output");
    assert_eq!(TranscodeStatus::Complete, finish.status());
    assert_eq!(0, finish.read());
    assert_eq!(0, finish.written());
}

#[test]
fn test_codec_buffered_decoder_wraps_variable_width_incomplete_codec_error() {
    let mut decoder = CodecBufferedDecoder::new(VariableByteCodec);
    let mut output = [0_u8; 1];

    let error = decoder
        .transcode(&[0x80], 0, &mut output, 0)
        .expect_err("strict adapter should not classify codec errors");
    assert_eq!(
        CodecDecodeError::Decode {
            source: TestDecodeError::Incomplete {
                required: 2,
                available: 1,
            },
            input_index: 0,
        },
        error,
    );

    let progress = decoder
        .transcode(&[0x80, 9], 0, &mut output, 0)
        .expect("caller-refilled input should complete");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(2, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([9], output);
}

#[test]
fn test_codec_buffered_decoder_reports_output_index_beyond_buffer() {
    let mut decoder = CodecBufferedDecoder::new(VariableByteCodec);
    let mut output = [];

    let progress = decoder
        .transcode(&[1], 0, &mut output, 1)
        .expect("out-of-range output index should request capacity");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 1,
            additional: super::nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_codec_buffered_decoder_reports_input_index_beyond_buffer() {
    let mut decoder = CodecBufferedDecoder::new(VariableByteCodec);
    let mut output = [0_u8; 1];

    let error = decoder
        .transcode(&[1], 2, &mut output, 0)
        .expect_err("out-of-range input index should fail");

    assert_eq!(CodecDecodeError::InvalidInputIndex { index: 2, len: 1 }, error);
}

#[test]
fn test_codec_buffered_decoder_finish_reports_output_index_beyond_buffer() {
    let mut decoder = CodecBufferedDecoder::new(VariableByteCodec);
    let mut output = [];

    let progress = decoder
        .finish(&mut output, 1)
        .expect("out-of-range finish output index should request capacity");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 1,
            additional: super::nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_codec_buffered_decoder_finish_does_not_handle_input_tail() {
    let mut decoder = CodecBufferedDecoder::new(FixedPairCodec);
    let mut output = [0_u8; 1];

    let progress = decoder
        .transcode(&[7], 0, &mut output, 0)
        .expect("partial input should not be retained");
    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            additional: super::nz(1),
            available: 1,
        },
        progress.status(),
    );

    let finish = decoder
        .finish(&mut output, 0)
        .expect("codec decoder has no finish output");

    assert_eq!(TranscodeStatus::Complete, finish.status());
    assert_eq!(0, finish.read());
    assert_eq!(0, finish.written());
}

#[test]
fn test_codec_buffered_decoder_wraps_invalid_codec_error() {
    let mut decoder = CodecBufferedDecoder::new(VariableByteCodec);
    let mut output = [0_u8; 1];

    let error = decoder
        .transcode(&[0xff], 0, &mut output, 0)
        .expect_err("invalid input should fail");

    assert_eq!(
        CodecDecodeError::Decode {
            source: TestDecodeError::Invalid { consumed: 1 },
            input_index: 0,
        },
        error,
    );
}
