// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shows why codecs with decode lifecycle output need a lifecycle-aware facade.

use core::{
    convert::Infallible,
    num::NonZeroUsize,
};

use qubit_codec::{
    Codec,
    CodecValueDecoder,
    DecodeFailure,
};

const RESET_MARKER: u8 = 0xfe;
const FINISH_MARKER: u8 = 0xff;

/// Demonstrates strict and lifecycle-aware whole-value decode behavior.
fn main() {
    let mut strict = CodecValueDecoder::new(MarkerCodec);
    assert!(strict.decode(&[0x42]).is_err());

    let mut lifecycle_decoder = CodecValueDecoder::new(MarkerCodec);
    let lifecycle = lifecycle_decoder
        .decode_lifecycle(&[0x42])
        .expect("lifecycle-aware decoding should preserve every emitted value");
    let (reset, value, finish) = lifecycle.into_parts();

    assert_eq!(vec![RESET_MARKER], reset);
    assert_eq!(0x42, value);
    assert_eq!(vec![FINISH_MARKER], finish);
}

/// A one-byte codec that emits values when decode state opens and closes.
struct MarkerCodec;

impl Codec for MarkerCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_RESET_VALUES: usize = 1;
    const MAX_DECODE_FINISH_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index < input.len());
        Ok((input[input_index], NonZeroUsize::MIN))
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

    unsafe fn decode_reset(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        output[output_index] = RESET_MARKER;
        Ok(1)
    }

    unsafe fn decode_finish(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        output[output_index] = FINISH_MARKER;
        Ok(1)
    }
}
