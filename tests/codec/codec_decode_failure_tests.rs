// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::num::NonZeroUsize;

use qubit_codec::{
    Codec,
    CodecDecodeFailure,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DomainDecodeError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PlainCodec;

impl Codec for PlainCodec {
    type Value = Vec<u8>;
    type Unit = u8;
    type DecodeError = DomainDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: NonZeroUsize = qubit_io::nz!(2);

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(Vec<u8>, NonZeroUsize), CodecDecodeFailure<Self::DecodeError>>
    {
        if input[index] == 0xff {
            return Err(CodecDecodeFailure::invalid(
                DomainDecodeError,
                NonZeroUsize::MIN,
            ));
        }
        Ok((vec![input[index]], NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &Vec<u8>,
        output: &mut [u8],
        index: usize,
    ) -> Result<NonZeroUsize, Self::EncodeError> {
        output[index] = value[0];
        Ok(NonZeroUsize::MIN)
    }
}

#[test]
fn test_codec_decode_failure_reports_incomplete_control_flow() {
    let required_total = qubit_io::nz!(3);
    let failure = CodecDecodeFailure::<DomainDecodeError>::incomplete(required_total);

    assert_eq!(
        CodecDecodeFailure::Incomplete { required_total },
        failure
    );
    assert_eq!(Some(required_total), failure.required_total());
    assert_eq!(None, failure.invalid_source());
    assert_eq!(None, failure.consumed_units());
}

#[test]
fn test_codec_decode_failure_reports_invalid_domain_error() {
    let consumed = qubit_io::nz!(2);
    let failure = CodecDecodeFailure::invalid(DomainDecodeError, consumed);

    assert_eq!(
        CodecDecodeFailure::Invalid {
            source: DomainDecodeError,
            consumed: Some(consumed),
        },
        failure
    );
    assert_eq!(None, failure.required_total());
    assert_eq!(Some(&DomainDecodeError), failure.invalid_source());
    assert_eq!(Some(consumed), failure.consumed_units());
}

#[test]
fn test_codec_trait_is_safe_and_accepts_non_copy_non_default_values() {
    let mut codec = PlainCodec;
    let mut output = [0_u8; 1];

    let decoded = unsafe { codec.decode(&[0x41], 0) }
        .expect("plain codec should decode a value");
    assert_eq!(vec![0x41], decoded.0);

    let written = unsafe { codec.encode(&vec![0x42], &mut output, 0) }
        .expect("plain codec encode is infallible");
    assert_eq!(NonZeroUsize::MIN, written);
    assert_eq!([0x42], output);
}
