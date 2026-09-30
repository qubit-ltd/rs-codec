// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared single-byte lifecycle codec whose reset and finish phases are
//! observable.

use core::convert::Infallible;
use core::num::NonZeroUsize;

use qubit_codec as codec;
use qubit_codec::Codec;

/// One-byte codec that emits a reset marker, shifts decoded values, and reports
/// the shift count when the finish phase runs.
///
/// It is shared by the decoder adapter tests and the two decode lifecycle
/// output mirrors, which all observe the reset and finish phases.
#[derive(Default)]
pub(crate) struct ResetSensitiveLifecycleCodec {
    /// Number of values decoded since the last reset phase.
    decode_state: usize,
}

impl Codec for ResetSensitiveLifecycleCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_RESET_VALUES: usize = 1;

    const MAX_DECODE_FINISH_VALUES: usize = 1;

    unsafe fn decode_reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::DecodeError> {
        output[output_index] = 0xfe;
        self.decode_state = 1;
        Ok(1)
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        let decoded = input[input_index].wrapping_sub(self.decode_state as u8);
        self.decode_state += 1;
        Ok((decoded, NonZeroUsize::MIN))
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

    unsafe fn decode_finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::DecodeError> {
        output[output_index] = self.decode_state as u8;
        self.decode_state = 0;
        Ok(1)
    }
}
