// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{
    Codec,
    CodecEncodeError,
    CodecTranscodeEncoder,
    TranscodeError,
    Transcoder,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetFailCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("reset failed")]
struct ResetFailError;

impl Codec for ResetFailCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = ResetFailError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    fn can_encode_value(&self, value: &u8) -> bool {
        value.is_multiple_of(2)
    }

    const MAX_ENCODE_RESET_UNITS: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::DecodeFailure<Self::DecodeError>,
    > {
        Ok((input[input_index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(qubit_io::nz!(1))
    }

    unsafe fn encode_reset(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Err(ResetFailError)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RejectOddCodec;

impl Codec for RejectOddCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = &'static str;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    fn can_encode_value(&self, value: &u8) -> bool {
        value.is_multiple_of(2)
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::DecodeFailure<Self::DecodeError>,
    > {
        Ok((input[input_index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(self.can_encode_value(value));
        output[output_index] = *value;
        Ok(qubit_io::nz!(1))
    }
}

#[test]
fn test_codec_transcode_encode_hooks_wraps_encode_errors() {
    let mut encoder = CodecTranscodeEncoder::new(RejectOddCodec);
    let mut output = [0_u8; 1];

    let error = encoder
        .transcode(&[7], 0, &mut output, 0)
        .expect_err("strict encode hooks should reject unencodable values");

    assert_eq!(
        TranscodeError::Domain(CodecEncodeError::UnencodableValue {
            input_index: 0
        }),
        error,
    );
}

#[test]
fn test_codec_transcode_encode_hooks_wraps_encode_reset_errors() {
    let mut encoder = CodecTranscodeEncoder::new(ResetFailCodec);
    let mut output = [0_u8; 1];

    let error = encoder
        .reset(&mut output, 0)
        .expect_err("reset errors should be wrapped");

    assert_eq!(
        TranscodeError::Domain(CodecEncodeError::EncodeReset {
            source: ResetFailError,
        }),
        error,
    );
}
