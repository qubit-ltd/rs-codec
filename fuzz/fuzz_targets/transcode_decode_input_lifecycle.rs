// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![no_main]

use core::convert::Infallible;
use std::io::Cursor;

use libfuzzer_sys::fuzz_target;
use qubit_codec::{
    CapacityError,
    TranscodeDecodeError,
    TranscodeDecodeInput,
    TranscodeDecoder,
    TranscodeProgress,
    Transcoder,
};

const RESET: u8 = 0xa2;
const FINISH: u8 = 0xf2;
/// Keeps standalone fuzz runs aligned with the CI input-size budget.
const MAX_INPUT_LEN: usize = 4 * 1024;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_INPUT_LEN)];
    let mut input =
        TranscodeDecodeInput::with_capacity(Cursor::new(data.to_vec()), 1);
    let mut decoder = MarkerDecoder;
    let mut mapper = |_error: TranscodeDecodeError<Infallible>| {
        std::io::Error::other("marker decoder cannot fail")
    };
    let mut lifecycle = Vec::new();
    let mut decoded = Vec::new();

    for action in data.iter().copied().chain(core::iter::once(2)) {
        match action % 3 {
            0 => {
                let mut output = [0_u8; 1];
                let written = input
                    .reset(&mut decoder, &mut mapper, &mut output, 0, 1)
                    .unwrap();
                lifecycle.extend_from_slice(&output[..written]);
            }
            1 => {
                let mut output = [0_u8; 1];
                let written = input
                    .transcode(&mut decoder, &mut mapper, &mut output, 0, 1)
                    .unwrap();
                decoded.extend_from_slice(&output[..written]);
            }
            _ => {
                let mut output = [0_u8; 1];
                let written = input
                    .finish(&mut decoder, &mut mapper, &mut output, 0, 1)
                    .unwrap();
                lifecycle.extend_from_slice(&output[..written]);
            }
        }
    }

    assert!(decoded.len() <= data.len());
    assert_eq!(&data[..decoded.len()], decoded.as_slice());
    assert!(
        lifecycle
            .iter()
            .all(|unit| *unit == RESET || *unit == FINISH)
    );
    let (_inner, unread) = input.into_parts();
    assert!(unread.available() <= 1);
});

struct MarkerDecoder;

impl Transcoder for MarkerDecoder {
    type Input = u8;
    type Output = u8;
    type Error = TranscodeDecodeError<Infallible>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }
    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }
    fn reset(
        &mut self,
        output: &mut [u8],
        index: usize,
    ) -> Result<usize, Self::Error> {
        output[index] = RESET;
        Ok(1)
    }
    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index == input.len() {
            return Ok(TranscodeProgress::complete(0, 0));
        }
        if output_index == output.len() {
            return Ok(TranscodeProgress::need_output(
                output_index,
                qubit_codec::nz(1),
                0,
                0,
                0,
            ));
        }
        output[output_index] = input[input_index];
        Ok(TranscodeProgress::complete(1, 1))
    }
    fn finish(
        &mut self,
        output: &mut [u8],
        index: usize,
    ) -> Result<usize, Self::Error> {
        output[index] = FINISH;
        Ok(1)
    }
}

impl TranscodeDecoder for MarkerDecoder {
    type DecodeError = Infallible;
}
