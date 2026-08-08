// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the semantic transcode decoder marker trait.

use qubit_codec as codec;
use qubit_codec::CapacityError;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeDecoder;
use qubit_codec::TranscodeProgress;
use qubit_codec::Transcoder;

#[derive(Default)]
struct ByteToChar;

impl Transcoder for ByteToChar {
    type Input = u8;
    type Output = char;
    type Error = TranscodeDecodeError<core::convert::Infallible>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn reset(
        &mut self,
        output: &mut [char],
        output_index: usize,
    ) -> Result<usize, TranscodeDecodeError<core::convert::Infallible>> {
        codec::TranscodeFailure::ensure_output_index(
            output.len(),
            output_index,
        )?;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [char],
        output_index: usize,
    ) -> Result<
        TranscodeProgress,
        TranscodeDecodeError<core::convert::Infallible>,
    > {
        let readable = input.len().saturating_sub(input_index);
        let writable = output.len().saturating_sub(output_index);
        let count = readable.min(writable);
        for offset in 0..count {
            output[output_index + offset] = input[input_index + offset] as char;
        }
        Ok(TranscodeProgress::complete(count, count))
    }

    fn finish(
        &mut self,
        output: &mut [char],
        output_index: usize,
    ) -> Result<usize, TranscodeDecodeError<core::convert::Infallible>> {
        codec::TranscodeFailure::ensure_output_index(
            output.len(),
            output_index,
        )?;
        Ok(0)
    }
}

impl TranscodeDecoder for ByteToChar {
    type DecodeError = core::convert::Infallible;
}

#[test]
fn test_transcode_decoder_is_a_semantic_transcoder_bound() {
    fn assert_decoder<T: TranscodeDecoder<Input = u8, Output = char>>() {}

    assert_decoder::<ByteToChar>();
}
