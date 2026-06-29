// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the semantic transcode converter marker trait.

use qubit_codec::{
    CapacityError, TranscodeConverter, TranscodeError, TranscodeProgress, Transcoder,
};

#[derive(Default)]
struct ByteToWord;

impl Transcoder<u8, u16> for ByteToWord {
    type Error = TranscodeError<core::convert::Infallible>;
    type DomainError = core::convert::Infallible;

    fn map_error(&self, error: TranscodeError<Self::DomainError>) -> Self::Error {
        error
    }

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn reset(&mut self, output: &mut [u16], output_index: usize) -> Result<usize, Self::Error> {
        TranscodeError::<Self::DomainError>::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let readable = input.len().saturating_sub(input_index);
        let writable = output.len().saturating_sub(output_index);
        let count = readable.min(writable);
        for offset in 0..count {
            output[output_index + offset] = input[input_index + offset] as u16;
        }
        Ok(TranscodeProgress::complete(count, count))
    }

    fn finish(&mut self, output: &mut [u16], output_index: usize) -> Result<usize, Self::Error> {
        TranscodeError::<Self::DomainError>::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }
}

impl TranscodeConverter<u8, u16> for ByteToWord {}

#[test]
fn test_transcode_converter_is_a_semantic_transcoder_bound() {
    fn assert_converter<T: TranscodeConverter<u8, u16>>() {}

    assert_converter::<ByteToWord>();
}
