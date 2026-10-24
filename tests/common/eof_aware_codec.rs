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

    unsafe fn decode(&mut self, input: &[u8], index: usize) -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
        if input[index] == 0xfe && input.len() - index == 1 {
            return Err(DecodeFailure::incomplete(NonZeroUsize::new(2).unwrap()));
        }
        Ok((input[index], NonZeroUsize::MIN))
    }

    unsafe fn decode_eof(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
        if input[index] == 0xfe && input.len() - index == 1 {
            return Ok((b'?', NonZeroUsize::MIN));
        }
        // SAFETY: The caller supplies the same readable range as decode.
        unsafe { self.decode(input, index) }
    }

    unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Infallible> {
        output[index] = *value;
        Ok(1)
    }
}
