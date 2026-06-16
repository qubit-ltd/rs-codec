// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{Codec, CodecDecodeError, nz, CodecTranscodeDecoder, TranscodeError, Transcoder};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FlushFailCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("flush failed")]
struct FlushFailError;

unsafe impl Codec for FlushFailCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = FlushFailError;
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
        Ok((input[index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[index] = *value;
        Ok(nz!(1))
    }

    unsafe fn decode_flush(
        &mut self,
        _output: &mut [u8],
        _index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Err(FlushFailError)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct InvalidByteCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("invalid byte")]
struct InvalidByteError;

unsafe impl Codec for InvalidByteCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = InvalidByteError;
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
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        if input[index] == 0xff {
            Err(InvalidByteError)
        } else {
            Ok((input[index], core::num::NonZeroUsize::MIN))
        }
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[index] = *value;
        Ok(nz!(1))
    }
}

#[test]
fn test_codec_transcode_decode_hooks_wraps_decode_errors() {
    let mut decoder = CodecTranscodeDecoder::new(InvalidByteCodec);
    let mut output = [0_u8; 1];

    let error = decoder
        .transcode(&[0xff], 0, &mut output, 0)
        .expect_err("strict decode hooks should wrap codec errors");

    assert_eq!(
        TranscodeError::Domain(CodecDecodeError::Decode {
            source: InvalidByteError,
            input_index: 0,
        }),
        error,
    );
}

#[test]
fn test_codec_transcode_decode_hooks_wraps_decode_flush_errors() {
    let mut decoder = CodecTranscodeDecoder::new(FlushFailCodec);
    let mut output = [0_u8; 1];

    let error = decoder
        .finish(&mut output, 0)
        .expect_err("flush errors should be wrapped");

    assert_eq!(
        TranscodeError::Domain(CodecDecodeError::Decode {
            source: FlushFailError,
            input_index: 0,
        }),
        error,
    );
}
