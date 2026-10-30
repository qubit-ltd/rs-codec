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
use std::io::Error;

use libfuzzer_sys::fuzz_target;
use qubit_codec::CapacityError;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeDecodeInput;
use qubit_codec::TranscodeDecoder;
use qubit_codec::TranscodeFailure;
use qubit_codec::TranscodeProgress;
use qubit_codec::Transcoder;
use qubit_utils::nonzero;

const RESET: u8 = 0xa2;
const FINISH: u8 = 0xf2;
const TAIL: u8 = 0xfe;
/// Keeps standalone fuzz runs aligned with the CI input-size budget.
const MAX_INPUT_LEN: usize = 4 * 1024;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_INPUT_LEN)];
    let (payload, actions) = data.split_at(data.len() / 2);
    run_case(payload, actions);
    run_case(&[], &[]);
    let mut short_tail = payload.to_vec();
    short_tail.push(TAIL);
    run_case(&short_tail, actions);
});

fn run_case(payload: &[u8], actions: &[u8]) {
    let mut input = TranscodeDecodeInput::with_capacity(Cursor::new(payload.to_vec()), 1);
    let mut decoder = MarkerDecoder::default();
    let mut mapper = |_error: TranscodeDecodeError<Infallible>| Error::other("marker decoder cannot fail");
    let mut lifecycle = Vec::new();
    let mut expected_lifecycle = Vec::new();
    let mut decoded = Vec::new();
    let expected_reads = actions
        .iter()
        .filter(|action| **action % 3 == 1)
        .count()
        .min(payload.len());

    for action in actions.iter().copied() {
        match action % 3 {
            0 => {
                expected_lifecycle.push(RESET);
                let mut output = [0_u8; 1];
                let written = input.reset(&mut decoder, &mut mapper, &mut output, 0, 1).unwrap();
                lifecycle.extend_from_slice(&output[..written]);
            }
            1 => {
                let mut output = [0_u8; 1];
                let written = input.transcode(&mut decoder, &mut mapper, &mut output, 0, 1).unwrap();
                decoded.extend_from_slice(&output[..written]);
            }
            _ => {
                expected_lifecycle.push(FINISH);
                let mut output = [0_u8; 1];
                let written = input.finish(&mut decoder, &mut mapper, &mut output, 0, 1).unwrap();
                lifecycle.extend_from_slice(&output[..written]);
            }
        }
    }

    let mut expected_output = payload.to_vec();
    if let Some(last) = expected_output.last_mut() {
        if *last == TAIL {
            *last = b'?';
        }
    }
    assert_eq!(&expected_output[..expected_reads], decoded.as_slice());
    assert_eq!(expected_lifecycle, lifecycle);
    let position = input.inner().position() as usize;
    let mut remaining = input.unread().to_vec();
    remaining.extend_from_slice(&input.inner().get_ref()[position..]);
    assert_eq!(&payload[expected_reads..], remaining.as_slice());

    let empty_calls_before = decoder.eof_empty_calls;
    let mut observed_zero = false;
    for _ in 0..=payload.len() {
        let mut output = [0_u8; 1];
        let written = input.transcode(&mut decoder, &mut mapper, &mut output, 0, 1).unwrap();
        decoded.extend_from_slice(&output[..written]);
        if written == 0 {
            observed_zero = true;
            break;
        }
    }
    assert!(observed_zero);
    assert!(decoder.eof_empty_calls > empty_calls_before);
    if payload.last() == Some(&TAIL) {
        assert!(decoder.eof_tail_calls > 0);
    }
    assert_eq!(expected_output, decoded);

    let mut output = [0_u8; 1];
    let written = input.finish(&mut decoder, &mut mapper, &mut output, 0, 1).unwrap();
    lifecycle.extend_from_slice(&output[..written]);
    expected_lifecycle.push(FINISH);
    assert_eq!(expected_lifecycle, lifecycle);
    let (inner, unread) = input.into_parts();
    assert!(unread.readable().is_empty());
    assert_eq!(inner.position() as usize, payload.len());
}

#[derive(Default)]
struct MarkerDecoder {
    eof_empty_calls: usize,
    eof_tail_calls: usize,
}

impl Transcoder for MarkerDecoder {
    type Input = u8;
    type Output = u8;
    type Error = TranscodeDecodeError<Infallible>;

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }
    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }
    fn reset(&mut self, output: &mut [u8], index: usize) -> Result<usize, Self::Error> {
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
        transcode_marker(input, input_index, output, output_index, false)
    }
    fn transcode_eof(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index == input.len() {
            self.eof_empty_calls += 1;
        } else if input.last() == Some(&TAIL) {
            self.eof_tail_calls += 1;
        }
        transcode_marker(input, input_index, output, output_index, true)
    }
    fn finish(&mut self, output: &mut [u8], index: usize) -> Result<usize, Self::Error> {
        output[index] = FINISH;
        Ok(1)
    }
}

impl TranscodeDecoder for MarkerDecoder {
    type DecodeError = Infallible;
}

/// Implements an EOF-sensitive marker while obeying the transcode progress
/// contract. A final `TAIL` is retained until EOF, then mapped to `?`;
/// intermediate markers pass through.
fn transcode_marker(
    input: &[u8],
    input_index: usize,
    output: &mut [u8],
    output_index: usize,
    end_of_input: bool,
) -> Result<TranscodeProgress, TranscodeDecodeError<Infallible>> {
    TranscodeFailure::ensure_transcode_indices(input.len(), input_index, output.len(), output_index)?;
    let mut read = 0;
    let mut written = 0;
    while input_index + read < input.len() {
        let index = input_index + read;
        let last_marker = input[index] == TAIL && index + 1 == input.len();
        if last_marker && !end_of_input {
            return Ok(TranscodeProgress::need_input(nonzero(2), read, written));
        }
        if output_index + written == output.len() {
            return Ok(TranscodeProgress::need_output(nonzero(1), read, written));
        }
        output[output_index + written] = if last_marker { b'?' } else { input[index] };
        read += 1;
        written += 1;
    }
    Ok(TranscodeProgress::complete(read, written))
}
