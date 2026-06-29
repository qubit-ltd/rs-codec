// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the semantic transcode encoder marker trait.

use qubit_codec::{
    CapacityError,
    TranscodeEncoder,
    TranscodeError,
    TranscodeProgress,
    Transcoder,
};

#[derive(Default)]
struct CharToByte;

impl Transcoder<char, u8> for CharToByte {
    type Error = TranscodeError<core::convert::Infallible>;
    type DomainError = core::convert::Infallible;

    fn map_error(
        &self,
        error: TranscodeError<Self::DomainError>,
    ) -> Self::Error {
        error
    }

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn reset(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        TranscodeError::<Self::DomainError>::ensure_output_index(
            output.len(),
            output_index,
        )?;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[char],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let readable = input.len().saturating_sub(input_index);
        let writable = output.len().saturating_sub(output_index);
        let count = readable.min(writable);
        for offset in 0..count {
            output[output_index + offset] = input[input_index + offset] as u8;
        }
        Ok(TranscodeProgress::complete(count, count))
    }

    fn finish(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        TranscodeError::<Self::DomainError>::ensure_output_index(
            output.len(),
            output_index,
        )?;
        Ok(0)
    }
}

impl TranscodeEncoder<char, u8> for CharToByte {}

#[test]
fn test_transcode_encoder_is_a_semantic_transcoder_bound() {
    fn assert_encoder<T: TranscodeEncoder<char, u8>>() {}

    assert_encoder::<CharToByte>();
}
