/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for semantic buffered transcoder marker traits.

use qubit_codec::{
    BufferedConverter,
    BufferedDecoder,
    BufferedEncoder,
    TranscodeProgress,
    Transcoder,
};

#[derive(Default)]
struct CharToByte;

impl Transcoder<char, u8> for CharToByte {
    type Error = core::convert::Infallible;

    fn max_output_len(&self, input_len: usize) -> Option<usize> {
        Some(input_len)
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
}

impl BufferedEncoder<char, u8> for CharToByte {}

#[derive(Default)]
struct ByteToChar;

impl Transcoder<u8, char> for ByteToChar {
    type Error = core::convert::Infallible;

    fn max_output_len(&self, input_len: usize) -> Option<usize> {
        Some(input_len)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [char],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let readable = input.len().saturating_sub(input_index);
        let writable = output.len().saturating_sub(output_index);
        let count = readable.min(writable);
        for offset in 0..count {
            output[output_index + offset] = input[input_index + offset] as char;
        }
        Ok(TranscodeProgress::complete(count, count))
    }
}

impl BufferedDecoder<u8, char> for ByteToChar {}

#[derive(Default)]
struct ByteToWord;

impl Transcoder<u8, u16> for ByteToWord {
    type Error = core::convert::Infallible;

    fn max_output_len(&self, input_len: usize) -> Option<usize> {
        Some(input_len)
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
}

impl BufferedConverter<u8, u16> for ByteToWord {}

#[test]
fn test_buffered_encoder_is_a_semantic_transcoder_bound() {
    fn assert_encoder<T: BufferedEncoder<char, u8>>() {}

    assert_encoder::<CharToByte>();
}

#[test]
fn test_buffered_decoder_is_a_semantic_transcoder_bound() {
    fn assert_decoder<T: BufferedDecoder<u8, char>>() {}

    assert_decoder::<ByteToChar>();
}

#[test]
fn test_buffered_converter_is_a_semantic_transcoder_bound() {
    fn assert_converter<T: BufferedConverter<u8, u16>>() {}

    assert_converter::<ByteToWord>();
}
