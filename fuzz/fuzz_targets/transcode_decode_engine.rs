// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![no_main]

use core::convert::Infallible;
use core::num::NonZeroUsize;

use libfuzzer_sys::fuzz_target;
use qubit_codec::Codec;
use qubit_codec::DecodeFailure;
use qubit_codec::TranscodeDecodeErrorOf;
use qubit_codec::engine::DecodeContext;
use qubit_codec::engine::DecodeInvalidAction;
use qubit_codec::engine::TranscodeDecodeEngine;
use qubit_codec::engine::TranscodeDecodeHooks;

const MAX_INPUT_LEN: usize = 4 * 1024;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_INPUT_LEN)];
    let mut decoder = TranscodeDecodeEngine::new(PrefixCodec, RejectingHooks);
    let mut input_index = 0;
    let mut output = [0_u8; 31];
    let mut decoded = Vec::with_capacity(data.len());
    let mut reset_output = [];
    decoder.reset(&mut reset_output, 0).unwrap_or_else(|error| {
        panic!("prefix codec reset is infallible: {error}")
    });

    while input_index < data.len() {
        let progress = decoder
            .transcode(&data[input_index..], 0, &mut output, 0)
            .unwrap_or_else(|error| {
                panic!("prefix codec is infallible: {error}")
            });
        progress
            .validate(0, data.len() - input_index, 0, output.len())
            .unwrap_or_else(|error| {
                panic!("engine returned invalid progress: {error}")
            });
        input_index += progress.read();
        decoded.extend_from_slice(&output[..progress.written()]);
        assert_eq!(decoded.as_slice(), &data[..input_index]);
        if progress.is_need_input() {
            break;
        }
    }

    assert_eq!(decoded.as_slice(), &data[..input_index]);
    if input_index < data.len() {
        assert_eq!(&[0xfe], &data[input_index..]);
    } else {
        let mut finish_output = [];
        let written =
            decoder
                .finish(&mut finish_output, 0)
                .unwrap_or_else(|error| {
                    panic!("prefix codec finish is infallible: {error}")
                });
        assert_eq!(0, written);
    }
});

#[derive(Default)]
struct PrefixCodec;

impl Codec for PrefixCodec {
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
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        if input[input_index] == 0xfe && input.len() - input_index == 1 {
            return Err(DecodeFailure::incomplete(
                NonZeroUsize::new(2).expect("two is non-zero"),
            ));
        }
        Ok((input[input_index], NonZeroUsize::MIN))
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
}

struct RejectingHooks;

impl TranscodeDecodeHooks<PrefixCodec> for RejectingHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut PrefixCodec,
        error: &Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, TranscodeDecodeErrorOf<PrefixCodec>>
    {
        match *error {}
    }
}
