// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::convert::Infallible;
use core::num::NonZeroUsize;

use qubit_codec::Codec;
use qubit_codec::DecodeFailure;

/// Leaves a lone marker pending until EOF, then emits a question mark.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct EofAwareCodec;

impl Codec for EofAwareCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
        debug_assert!(input_index < input.len());
        if input[input_index] == 0xfe && input.len() - input_index == 1 {
            return Err(DecodeFailure::incomplete(NonZeroUsize::new(2).unwrap()));
        }
        Ok((input[input_index], NonZeroUsize::MIN))
    }

    unsafe fn decode_eof(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
        debug_assert!(input_index < input.len());
        if input[input_index] == 0xfe && input.len() - input_index == 1 {
            return Ok((b'?', NonZeroUsize::MIN));
        }
        // SAFETY: The caller supplies the same readable range as decode.
        unsafe { self.decode(input, input_index) }
    }

    unsafe fn encode(&mut self, value: &u8, output: &mut [u8], output_index: usize) -> Result<usize, Infallible> {
        debug_assert!(output_index < output.len());
        output[output_index] = *value;
        Ok(1)
    }
}
