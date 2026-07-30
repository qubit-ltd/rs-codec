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
    TranscodeEncodeError,
    TranscodeEncodeOutput,
    TranscodeEncoder,
    TranscodeProgress,
    Transcoder,
};

const RESET: u8 = 0xa1;
const FINISH: u8 = 0xf1;

fuzz_target!(|data: &[u8]| {
    let mut output =
        TranscodeEncodeOutput::with_capacity(Cursor::new(Vec::new()), 1);
    let mut encoder = MarkerEncoder;
    let mut expected = Vec::new();
    let mut mapper = |_error: TranscodeEncodeError<Infallible, u8>| {
        std::io::Error::other("marker encoder cannot fail")
    };
    let mut cursor = 0;

    while cursor < data.len() {
        match data[cursor] % 4 {
            0 => {
                output.reset(&mut encoder, &mut mapper).unwrap();
                expected.push(RESET);
            }
            1 => {
                let end = (cursor + 1 + usize::from(data[cursor] % 8))
                    .min(data.len());
                output
                    .transcode(
                        &mut encoder,
                        &mut mapper,
                        &data[cursor + 1..end],
                        0,
                        end - cursor - 1,
                    )
                    .unwrap();
                expected.extend_from_slice(&data[cursor + 1..end]);
                cursor = end - 1;
            }
            2 => {
                output.finish(&mut encoder, &mut mapper).unwrap();
                expected.push(FINISH);
            }
            _ => output.flush().unwrap(),
        }
        cursor += 1;
    }
    output.flush().unwrap();
    assert_eq!(expected, output.into_parts().0.into_inner());
});

struct MarkerEncoder;

impl Transcoder for MarkerEncoder {
    type Input = u8;
    type Output = u8;
    type Error = TranscodeEncodeError<Infallible, u8>;

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
        let input = &input[input_index..];
        let output = &mut output[output_index..];
        let copied = input.len().min(output.len());
        output[..copied].copy_from_slice(&input[..copied]);

        if copied == input.len() {
            return Ok(TranscodeProgress::complete(copied, copied));
        }
        Ok(TranscodeProgress::need_output(
            output_index + copied,
            qubit_codec::nz(1),
            output.len() - copied,
            copied,
            copied,
        ))
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

impl TranscodeEncoder for MarkerEncoder {
    type EncodeError = Infallible;
}
