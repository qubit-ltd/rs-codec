// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec as codec;
use qubit_codec::Codec;
use qubit_codec::CodecTranscodeDecoder;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::Transcoder;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FlushFailCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("flush failed")]
struct FlushFailError;

impl Codec for FlushFailCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = FlushFailError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_FINISH_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
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

    unsafe fn decode_finish(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::DecodeError> {
        Err(FlushFailError)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct InvalidByteCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("invalid byte")]
struct InvalidByteError;

impl Codec for InvalidByteCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = InvalidByteError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        if input[input_index] == 0xff {
            Err(codec::DecodeFailure::invalid(
                InvalidByteError,
                core::num::NonZeroUsize::MIN,
            ))
        } else {
            Ok((input[input_index], core::num::NonZeroUsize::MIN))
        }
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
}

#[test]
fn test_codec_transcode_decode_hooks_wraps_decode_errors() {
    let mut decoder = CodecTranscodeDecoder::new(InvalidByteCodec);
    let mut reset_output = [];
    decoder.reset(&mut reset_output, 0).expect("initialize stream");

    let mut output = [0_u8; 1];

    let error = decoder
        .transcode(&[0xff], 0, &mut output, 0)
        .expect_err("strict decode hooks should wrap codec errors");

    assert_eq!(
        TranscodeDecodeError::domain_main_with_consumed(InvalidByteError, 0, Some(crate::nonzero(1)),),
        error,
    );
}

#[test]
fn test_codec_transcode_decode_hooks_wraps_decode_finish_errors() {
    let mut decoder = CodecTranscodeDecoder::new(FlushFailCodec);
    let mut reset_output = [];
    decoder.reset(&mut reset_output, 0).expect("initialize stream");

    let mut output = [0_u8; 1];

    let error = decoder
        .finish(&mut output, 0)
        .expect_err("flush errors should be wrapped");

    assert_eq!(TranscodeDecodeError::domain_finish(FlushFailError), error,);
}
