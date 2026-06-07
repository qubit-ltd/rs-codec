// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::collections::VecDeque;
use std::io::{
    Error,
    ErrorKind,
};

use qubit_codec::{
    BufferedDecodeInput,
    BufferedTranscoder,
    CapacityError,
    FinishError,
    TranscodeProgress,
};
use qubit_io::Input;

use super::nz;

#[derive(Debug, Eq, PartialEq)]
enum PairDecodeError {
    BadInputIndex,
    BadOutputIndex,
}

#[derive(Debug, Default)]
struct PairDecoder;

impl BufferedTranscoder<u16, u32> for PairDecoder {
    type Error = PairDecodeError;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len / 2)
    }

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        output: &mut [u32],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(PairDecodeError::BadInputIndex);
        }
        if output_index > output.len() {
            return Err(PairDecodeError::BadOutputIndex);
        }
        let mut read = 0;
        let mut written = 0;
        while input_index + read + 1 < input.len() {
            if output_index + written == output.len() {
                return Ok(TranscodeProgress::need_output(
                    output_index + written,
                    nz(1),
                    0,
                    read,
                    written,
                ));
            }
            let high = input[input_index + read] as u32;
            let low = input[input_index + read + 1] as u32;
            output[output_index + written] = (high << 16) | low;
            read += 2;
            written += 1;
        }
        let available = input.len() - (input_index + read);
        if available == 0 {
            Ok(TranscodeProgress::complete(read, written))
        } else {
            Ok(TranscodeProgress::need_input(
                input_index + read,
                nz(1),
                available,
                read,
                written,
            ))
        }
    }

    fn finish(
        &mut self,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<usize, FinishError<Self::Error>> {
        Ok(0)
    }
}

#[derive(Debug, Default)]
struct FinishDecoder {
    finished: bool,
}

impl BufferedTranscoder<u16, u32> for FinishDecoder {
    type Error = PairDecodeError;

    fn max_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(usize::from(!self.finished))
    }

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(PairDecodeError::BadInputIndex);
        }
        if output_index > 0 {
            return Err(PairDecodeError::BadOutputIndex);
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(
        &mut self,
        output: &mut [u32],
        output_index: usize,
    ) -> Result<usize, FinishError<Self::Error>> {
        if self.finished {
            return Ok(0);
        }
        if output_index >= output.len() {
            return Err(FinishError::insufficient_output(output_index, 1, 0));
        }
        output[output_index] = 0xfeed_beef;
        self.finished = true;
        Ok(1)
    }
}

#[derive(Debug, Default)]
struct ZeroWidthFailingFinishDecoder;

impl BufferedTranscoder<u16, u32> for ZeroWidthFailingFinishDecoder {
    type Error = PairDecodeError;

    fn max_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(PairDecodeError::BadInputIndex);
        }
        if output_index > 0 {
            return Err(PairDecodeError::BadOutputIndex);
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(
        &mut self,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<usize, FinishError<Self::Error>> {
        Err(FinishError::source(PairDecodeError::BadInputIndex))
    }
}

#[derive(Debug)]
struct ChunkedInput {
    chunks: VecDeque<Vec<u16>>,
}

impl ChunkedInput {
    fn new(chunks: Vec<Vec<u16>>) -> Self {
        Self {
            chunks: VecDeque::from(chunks),
        }
    }
}

impl Input for ChunkedInput {
    type Item = u16;

    unsafe fn read_unchecked(
        &mut self,
        output: &mut [u16],
        index: usize,
        count: usize,
    ) -> std::io::Result<usize> {
        let Some(chunk) = self.chunks.pop_front() else {
            return Ok(0);
        };
        let read = count.min(chunk.len());
        output[index..index + read].copy_from_slice(&chunk[..read]);
        if read < chunk.len() {
            self.chunks.push_front(chunk[read..].to_vec());
        }
        Ok(read)
    }
}

#[derive(Debug, Default)]
struct TwoUnitFinishDecoder;

impl BufferedTranscoder<u16, u32> for TwoUnitFinishDecoder {
    type Error = PairDecodeError;

    fn max_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(2)
    }

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(PairDecodeError::BadInputIndex);
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(
        &mut self,
        output: &mut [u32],
        output_index: usize,
    ) -> Result<usize, FinishError<Self::Error>> {
        FinishError::ensure_output_capacity(output.len(), output_index, 2)?;
        output[output_index] = 0xaaaa;
        output[output_index + 1] = 0xbbbb;
        Ok(2)
    }
}

#[derive(Debug, Default)]
struct CapacityBoundDecoder;

impl BufferedTranscoder<u16, u32> for CapacityBoundDecoder {
    type Error = PairDecodeError;

    fn max_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Err(CapacityError::OutputLengthOverflow)
    }

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(PairDecodeError::BadInputIndex);
        }
        Ok(TranscodeProgress::complete(0, 0))
    }
}

#[derive(Debug, Default)]
struct FailingTranscodeDecoder;

impl BufferedTranscoder<u16, u32> for FailingTranscodeDecoder {
    type Error = PairDecodeError;

    fn max_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(PairDecodeError::BadInputIndex);
        }
        Err(PairDecodeError::BadInputIndex)
    }
}

#[derive(Clone, Copy, Debug)]
enum FinishFailure {
    Capacity,
    InvalidIndex,
}

#[derive(Debug)]
struct FailingFinishDecoder {
    failure: FinishFailure,
}

impl BufferedTranscoder<u16, u32> for FailingFinishDecoder {
    type Error = PairDecodeError;

    fn max_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(PairDecodeError::BadInputIndex);
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(
        &mut self,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<usize, FinishError<Self::Error>> {
        match self.failure {
            FinishFailure::Capacity => {
                Err(FinishError::capacity(CapacityError::OutputLengthOverflow))
            }
            FinishFailure::InvalidIndex => {
                Err(FinishError::invalid_output_index(4, 1))
            }
        }
    }
}

#[derive(Debug)]
struct FailingInput;

impl Input for FailingInput {
    type Item = u16;

    unsafe fn read_unchecked(
        &mut self,
        _output: &mut [u16],
        _index: usize,
        _count: usize,
    ) -> std::io::Result<usize> {
        Err(Error::new(ErrorKind::BrokenPipe, "input failure"))
    }
}

#[derive(Debug)]
struct ErrorAfterFirstReadInput {
    first_read: bool,
}

impl Default for ErrorAfterFirstReadInput {
    fn default() -> Self {
        Self { first_read: true }
    }
}

impl Input for ErrorAfterFirstReadInput {
    type Item = u16;

    unsafe fn read_unchecked(
        &mut self,
        output: &mut [u16],
        index: usize,
        _count: usize,
    ) -> std::io::Result<usize> {
        if self.first_read {
            self.first_read = false;
            output[index] = 0x0001;
            Ok(1)
        } else {
            Err(Error::new(ErrorKind::BrokenPipe, "refill failure"))
        }
    }
}

fn map_error(error: PairDecodeError) -> Error {
    Error::new(ErrorKind::InvalidData, format!("{error:?}"))
}

#[test]
fn test_buffered_decode_input_exposes_parts_and_debug() {
    let mapper: fn(PairDecodeError) -> Error = map_error;
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let input: BufferedDecodeInput<_, _, _, u32> =
        BufferedDecodeInput::with_capacity(input, PairDecoder, 3, mapper);

    let debug = format!("{input:?}");
    assert!(debug.contains("BufferedDecodeInput"));
    assert_eq!(1, input.inner().chunks.len());
    let _decoder = input.decoder();

    let (inner, unread, _decoder, _mapper) = input.into_parts();
    assert_eq!(1, inner.chunks.len());
    assert!(unread.is_empty());
}

#[test]
fn test_buffered_decode_input_returns_zero_for_zero_count() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let decoder = PairDecoder;
    let mut input =
        BufferedDecodeInput::with_capacity(input, decoder, 3, map_error);
    let mut output = [0_u32; 1];

    // SAFETY: The empty output range at index zero is valid.
    let read = unsafe {
        input
            .read_unchecked(&mut output, 0, 0)
            .expect("zero-count read should be a no-op")
    };

    assert_eq!(0, read);
    assert_eq!([0], output);
}

#[test]
fn test_buffered_decode_input_decodes_across_refills() {
    let input =
        ChunkedInput::new(vec![vec![0x0001], vec![0x0002, 0x0003, 0x0004]]);
    let decoder = PairDecoder;
    let mut input =
        BufferedDecodeInput::with_capacity(input, decoder, 3, map_error);
    let mut output = [0_u32; 2];

    // SAFETY: The full output range is valid.
    let read = unsafe {
        input
            .read_unchecked(&mut output, 0, 2)
            .expect("decode input should produce values")
    };

    assert_eq!(2, read);
    assert_eq!([0x0001_0002, 0x0003_0004], output);
}

#[test]
fn test_buffered_decode_input_returns_partial_at_clean_eof_before_finish() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let decoder = PairDecoder;
    let mut input =
        BufferedDecodeInput::with_capacity(input, decoder, 3, map_error);
    let mut output = [0_u32; 2];

    // SAFETY: The full output range is valid.
    let read = unsafe {
        input
            .read_unchecked(&mut output, 0, 2)
            .expect("complete value should be returned before final EOF")
    };

    assert_eq!(1, read);
    assert_eq!(0x0001_0002, output[0]);
}

#[test]
fn test_buffered_decode_input_stops_when_output_is_full() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003, 0x0004]]);
    let decoder = PairDecoder;
    let mut input =
        BufferedDecodeInput::with_capacity(input, decoder, 4, map_error);
    let mut output = [0_u32; 1];

    // SAFETY: The full output range is valid.
    let read = unsafe {
        input
            .read_unchecked(&mut output, 0, 1)
            .expect("full output should stop decoding")
    };

    assert_eq!(1, read);
    assert_eq!([0x0001_0002], output);
}

#[test]
fn test_buffered_decode_input_reports_initial_refill_errors() {
    let input = FailingInput;
    let decoder = PairDecoder;
    let mut input =
        BufferedDecodeInput::with_capacity(input, decoder, 3, map_error);
    let mut output = [0_u32; 1];

    // SAFETY: The full output range is valid.
    let error = unsafe { input.read_unchecked(&mut output, 0, 1) }
        .expect_err("input refill error should be returned");

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}

#[test]
fn test_buffered_decode_input_reports_transcoder_errors() {
    let input = ChunkedInput::new(vec![vec![0x0001]]);
    let decoder = FailingTranscodeDecoder;
    let mut input =
        BufferedDecodeInput::with_capacity(input, decoder, 3, map_error);
    let mut output = [0_u32; 1];

    // SAFETY: The full output range is valid.
    let error = unsafe { input.read_unchecked(&mut output, 0, 1) }
        .expect_err("decoder error should be mapped to I/O error");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("BadInputIndex"));
}

#[test]
fn test_buffered_decode_input_reports_refill_errors_after_need_input() {
    let input = ErrorAfterFirstReadInput::default();
    let decoder = PairDecoder;
    let mut input =
        BufferedDecodeInput::with_capacity(input, decoder, 3, map_error);
    let mut output = [0_u32; 1];

    // SAFETY: The full output range is valid.
    let error = unsafe { input.read_unchecked(&mut output, 0, 1) }
        .expect_err("NeedInput refill error should be returned");

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}

#[test]
fn test_buffered_decode_input_returns_partial_values_before_incomplete_eof() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003]]);
    let decoder = PairDecoder;
    let mut input =
        BufferedDecodeInput::with_capacity(input, decoder, 3, map_error);
    let mut output = [0_u32; 2];

    // SAFETY: The full output range is valid.
    let read = unsafe {
        input
            .read_unchecked(&mut output, 0, 2)
            .expect("partial value should be returned before EOF error")
    };
    assert_eq!(1, read);
    assert_eq!(0x0001_0002, output[0]);

    // SAFETY: The full output range is valid.
    let error = unsafe { input.read_unchecked(&mut output, 0, 2) }
        .expect_err("incomplete tail at EOF should be an I/O error");
    assert_eq!(ErrorKind::UnexpectedEof, error.kind());
}

#[test]
fn test_buffered_decode_input_reports_insufficient_finish_output() {
    let input = ChunkedInput::new(Vec::new());
    let decoder = TwoUnitFinishDecoder;
    let mut input =
        BufferedDecodeInput::with_capacity(input, decoder, 3, map_error);
    let mut output = [0_u32; 1];

    // SAFETY: The full output range is valid.
    let error = unsafe { input.read_unchecked(&mut output, 0, 1) }
        .expect_err("one-shot finish should require the full finish bound");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("insufficient finish output"));
}

#[test]
fn test_buffered_decode_input_maps_finish_capacity_bound_error() {
    let input = ChunkedInput::new(Vec::new());
    let decoder = CapacityBoundDecoder;
    let mut input =
        BufferedDecodeInput::with_capacity(input, decoder, 3, map_error);
    let mut output = [0_u32; 1];

    // SAFETY: The full output range is valid.
    let error = unsafe { input.read_unchecked(&mut output, 0, 1) }
        .expect_err("finish bound overflow should be mapped to I/O error");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("output length overflow"));
}

#[test]
fn test_buffered_decode_input_maps_finish_failure_variants() {
    for failure in [FinishFailure::Capacity, FinishFailure::InvalidIndex] {
        let input = ChunkedInput::new(Vec::new());
        let decoder = FailingFinishDecoder { failure };
        let mut input =
            BufferedDecodeInput::with_capacity(input, decoder, 3, map_error);
        let mut output = [0_u32; 1];

        // SAFETY: The full output range is valid.
        let error = unsafe { input.read_unchecked(&mut output, 0, 1) }
            .expect_err("finish failure should be mapped to I/O error");

        assert_eq!(ErrorKind::InvalidData, error.kind());
    }
}

#[test]
fn test_buffered_decode_input_finishes_decoder_at_clean_eof() {
    let input = ChunkedInput::new(Vec::new());
    let decoder = FinishDecoder::default();
    let mut input =
        BufferedDecodeInput::with_capacity(input, decoder, 3, map_error);
    let mut output = [0_u32; 1];

    // SAFETY: The full output range is valid.
    let read = unsafe {
        input
            .read_unchecked(&mut output, 0, 1)
            .expect("clean EOF should finish decoder")
    };
    assert_eq!(1, read);
    assert_eq!([0xfeed_beef], output);

    // SAFETY: The full output range is valid.
    let read = unsafe {
        input
            .read_unchecked(&mut output, 0, 1)
            .expect("finished decoder should report EOF")
    };
    assert_eq!(0, read);
}

#[test]
fn test_buffered_decode_input_delegates_zero_width_finish_at_clean_eof() {
    let input = ChunkedInput::new(Vec::new());
    let decoder = ZeroWidthFailingFinishDecoder;
    let mut input =
        BufferedDecodeInput::with_capacity(input, decoder, 3, map_error);
    let mut output = [0_u32; 1];

    // SAFETY: The full output range is valid.
    let error = unsafe { input.read_unchecked(&mut output, 0, 1) }
        .expect_err("zero-width finish errors should not be skipped");
    assert_eq!(ErrorKind::InvalidData, error.kind());
}
