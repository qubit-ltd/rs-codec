// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::{convert::Infallible, num::NonZeroUsize};

use qubit_codec::{Codec, DecodeFailure};

/// Stateless one-byte codec shared by transcode integration tests.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct IdentityCodec;

impl Codec for IdentityCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

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
}
