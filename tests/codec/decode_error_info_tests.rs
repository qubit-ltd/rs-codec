/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for decode error metadata extraction.

use qubit_codec::{
    Codec,
    DecodeErrorInfo,
    DecodeFailure,
};

#[derive(Debug, Eq, PartialEq)]
struct RejectingDecodeError;

#[derive(Default)]
struct RejectingCodec;

unsafe impl Codec<u8, u8> for RejectingCodec {
    type DecodeError = RejectingDecodeError;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> usize {
        1
    }

    fn max_units_per_value(&self) -> usize {
        2
    }

    unsafe fn decode_unchecked(&self, _input: &[u8], _index: usize) -> Result<(u8, usize), Self::DecodeError> {
        Err(RejectingDecodeError)
    }

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        debug_assert!(index < output.len());

        output[index] = *value;
        Ok(1)
    }
}

impl DecodeErrorInfo for RejectingDecodeError {
    fn failure(&self) -> DecodeFailure {
        DecodeFailure::Invalid { consumed: 1 }
    }
}

#[test]
fn test_decode_error_info_reports_invalid_input_consumption() {
    let codec = RejectingCodec;
    let input = [1_u8, 2_u8];

    let error = unsafe { codec.decode_unchecked(&input, 0) }.expect_err("decode should reject input");

    assert_eq!(DecodeFailure::Invalid { consumed: 1 }, error.failure());
}
