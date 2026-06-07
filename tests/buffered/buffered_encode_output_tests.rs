// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::{
    Error,
    ErrorKind,
};

use qubit_codec::{
    BufferedEncodeOutput,
    BufferedTranscoder,
    CapacityError,
    FinishError,
    TranscodeProgress,
};
use qubit_io::Output;

use super::nz;

#[derive(Debug, Eq, PartialEq)]
enum PairEncodeError {
    BadInputIndex,
    BadOutputIndex,
}

#[derive(Debug, Default)]
struct PairEncoder;

impl BufferedTranscoder<u32, u16> for PairEncoder {
    type Error = PairEncodeError;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        input_len
            .checked_mul(2)
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(PairEncodeError::BadInputIndex);
        }
        if output_index > output.len() {
            return Err(PairEncodeError::BadOutputIndex);
        }
        let mut read = 0;
        let mut written = 0;
        while input_index + read < input.len() {
            if input[input_index + read] == u32::MAX {
                return Err(PairEncodeError::BadInputIndex);
            }
            if output_index + written + 2 > output.len() {
                let available = output.len() - (output_index + written);
                return Ok(TranscodeProgress::need_output(
                    output_index + written,
                    nz(2 - available),
                    available,
                    read,
                    written,
                ));
            }
            let value = input[input_index + read];
            output[output_index + written] = (value >> 16) as u16;
            output[output_index + written + 1] = value as u16;
            read += 1;
            written += 2;
        }
        Ok(TranscodeProgress::complete(read, written))
    }

    fn finish(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, FinishError<Self::Error>> {
        Ok(0)
    }
}

#[derive(Debug, Default)]
struct FinishEncoder {
    finished: bool,
}

impl BufferedTranscoder<u32, u16> for FinishEncoder {
    type Error = PairEncodeError;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(usize::from(!self.finished))
    }

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(PairEncodeError::BadInputIndex);
        }
        if output_index > output.len() {
            return Err(PairEncodeError::BadOutputIndex);
        }
        if input_index == input.len() {
            return Ok(TranscodeProgress::complete(0, 0));
        }
        if output_index == output.len() {
            return Ok(TranscodeProgress::need_output(
                output_index,
                nz(1),
                0,
                0,
                0,
            ));
        }
        output[output_index] = input[input_index] as u16;
        Ok(TranscodeProgress::complete(1, 1))
    }

    fn finish(
        &mut self,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<usize, FinishError<Self::Error>> {
        if self.finished {
            return Ok(0);
        }
        if output_index >= output.len() {
            return Err(FinishError::insufficient_output(output_index, 1, 0));
        }
        output[output_index] = 0xeeee;
        self.finished = true;
        Ok(1)
    }
}

#[derive(Debug, Default)]
struct ZeroWidthFailingFinishEncoder;

impl BufferedTranscoder<u32, u16> for ZeroWidthFailingFinishEncoder {
    type Error = PairEncodeError;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(PairEncodeError::BadInputIndex);
        }
        if output_index > output.len() {
            return Err(PairEncodeError::BadOutputIndex);
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, FinishError<Self::Error>> {
        Err(FinishError::source(PairEncodeError::BadInputIndex))
    }
}

#[derive(Debug, Default)]
struct UnitOutput {
    units: Vec<u16>,
    flushed: bool,
}

impl Output for UnitOutput {
    type Item = u16;

    unsafe fn write_unchecked(
        &mut self,
        input: &[u16],
        index: usize,
        count: usize,
    ) -> std::io::Result<usize> {
        self.units.extend_from_slice(&input[index..index + count]);
        Ok(count)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flushed = true;
        Ok(())
    }
}

#[derive(Debug, Default)]
struct CapacityBoundEncoder;

impl BufferedTranscoder<u32, u16> for CapacityBoundEncoder {
    type Error = PairEncodeError;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Err(CapacityError::OutputLengthOverflow)
    }

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(PairEncodeError::BadInputIndex);
        }
        Ok(TranscodeProgress::complete(0, 0))
    }
}

#[derive(Clone, Copy, Debug)]
enum FinishFailure {
    Capacity,
    InvalidIndex,
    InsufficientOutput,
}

#[derive(Debug)]
struct FailingFinishEncoder {
    failure: FinishFailure,
}

impl BufferedTranscoder<u32, u16> for FailingFinishEncoder {
    type Error = PairEncodeError;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(PairEncodeError::BadInputIndex);
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, FinishError<Self::Error>> {
        match self.failure {
            FinishFailure::Capacity => {
                Err(FinishError::capacity(CapacityError::OutputLengthOverflow))
            }
            FinishFailure::InvalidIndex => {
                Err(FinishError::invalid_output_index(4, 1))
            }
            FinishFailure::InsufficientOutput => {
                Err(FinishError::insufficient_output(0, 2, 1))
            }
        }
    }
}

#[derive(Debug, Default)]
struct NeedInputEncoder;

impl BufferedTranscoder<u32, u16> for NeedInputEncoder {
    type Error = PairEncodeError;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(PairEncodeError::BadInputIndex);
        }
        Ok(TranscodeProgress::need_input(input_index, nz(1), 0, 0, 0))
    }
}

#[derive(Debug, Default)]
struct NeedOutputAfterReadEncoder;

impl BufferedTranscoder<u32, u16> for NeedOutputAfterReadEncoder {
    type Error = PairEncodeError;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        _output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(PairEncodeError::BadInputIndex);
        }
        Ok(TranscodeProgress::need_output(output_index, nz(1), 0, 1, 0))
    }
}

fn map_error(error: PairEncodeError) -> Error {
    Error::new(ErrorKind::InvalidData, format!("{error:?}"))
}

#[test]
fn test_buffered_encode_output_exposes_parts_and_debug() {
    let mapper: fn(PairEncodeError) -> Error = map_error;
    let output = UnitOutput::default();
    let output: BufferedEncodeOutput<_, _, _, u32> =
        BufferedEncodeOutput::with_capacity(output, PairEncoder, 3, mapper);

    let debug = format!("{output:?}");
    assert!(debug.contains("BufferedEncodeOutput"));
    assert!(output.inner().units.is_empty());
    let _encoder = output.encoder();

    let (inner, pending, _encoder, _mapper) = output.into_parts();
    assert!(inner.units.is_empty());
    assert!(pending.is_empty());
}

#[test]
fn test_buffered_encode_output_returns_zero_for_zero_count() {
    let output = UnitOutput::default();
    let encoder = PairEncoder;
    let mut output =
        BufferedEncodeOutput::with_capacity(output, encoder, 3, map_error);

    // SAFETY: The empty input range at index zero is valid.
    let written = unsafe {
        output
            .write_unchecked(&[0x0001_0002], 0, 0)
            .expect("zero-count write should be a no-op")
    };

    assert_eq!(0, written);
    assert!(output.inner().units.is_empty());
}

#[test]
fn test_buffered_encode_output_encodes_and_flushes_units() {
    let output = UnitOutput::default();
    let encoder = PairEncoder;
    let mut output =
        BufferedEncodeOutput::with_capacity(output, encoder, 3, map_error);

    // SAFETY: The full input range is valid.
    let written = unsafe {
        output
            .write_unchecked(&[0x0001_0002, 0x0003_0004], 0, 2)
            .expect("encoding should accept both values")
    };
    assert_eq!(2, written);

    output.flush().expect("flush should drain buffered units");

    assert_eq!(&[1, 2, 3, 4], output.inner().units.as_slice());
    assert!(output.inner().flushed);
}

#[test]
fn test_buffered_encode_output_flushes_full_buffer_before_next_write() {
    let output = UnitOutput::default();
    let encoder = PairEncoder;
    let mut output =
        BufferedEncodeOutput::with_capacity(output, encoder, 2, map_error);

    // SAFETY: The full input range is valid.
    let written = unsafe {
        output
            .write_unchecked(&[0x0001_0002], 0, 1)
            .expect("first value should fill the unit buffer")
    };
    assert_eq!(1, written);
    assert!(output.inner().units.is_empty());

    // SAFETY: The full input range is valid.
    let written = unsafe {
        output
            .write_unchecked(&[0x0003_0004], 0, 1)
            .expect("second value should flush the full buffer first")
    };
    assert_eq!(1, written);

    output.flush().expect("flush should drain buffered units");
    assert_eq!(&[1, 2, 3, 4], output.inner().units.as_slice());
}

#[test]
fn test_buffered_encode_output_reports_no_progress_need_output_capacity() {
    let output = UnitOutput::default();
    let encoder = PairEncoder;
    let mut output =
        BufferedEncodeOutput::with_capacity(output, encoder, 1, map_error);

    // SAFETY: The full input range is valid.
    let error = unsafe { output.write_unchecked(&[0x0001_0002], 0, 1) }
        .expect_err("insufficient fixed buffer capacity should be reported");

    assert_eq!(ErrorKind::InvalidInput, error.kind());
    assert!(error.to_string().contains("spare capacity"));
}

#[test]
fn test_buffered_encode_output_returns_after_need_output_consumes_input() {
    let output = UnitOutput::default();
    let encoder = NeedOutputAfterReadEncoder;
    let mut output =
        BufferedEncodeOutput::with_capacity(output, encoder, 1, map_error);

    // SAFETY: The full input range is valid.
    let written = unsafe {
        output
            .write_unchecked(&[0x1234], 0, 1)
            .expect("need-output after consuming input should return progress")
    };

    assert_eq!(1, written);
}

#[test]
fn test_buffered_encode_output_reports_transcoder_errors_as_io_errors() {
    let output = UnitOutput::default();
    let encoder = PairEncoder;
    let mut output =
        BufferedEncodeOutput::with_capacity(output, encoder, 3, map_error);
    let input = [u32::MAX];

    // SAFETY: The full input range is valid.
    let error = unsafe { output.write_unchecked(&input, 0, 1) }
        .expect_err("encoder error should be mapped to I/O error");

    assert_eq!(ErrorKind::InvalidData, error.kind());
}

#[test]
fn test_buffered_encode_output_rejects_need_input_status() {
    let output = UnitOutput::default();
    let encoder = NeedInputEncoder;
    let mut output =
        BufferedEncodeOutput::with_capacity(output, encoder, 3, map_error);

    // SAFETY: The full input range is valid.
    let error = unsafe { output.write_unchecked(&[0x1234], 0, 1) }
        .expect_err("encoder NeedInput status should be rejected");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(
        error
            .to_string()
            .contains("unexpectedly requested more input")
    );
}

#[test]
fn test_buffered_encode_output_flush_does_not_finish_encoder() {
    let output = UnitOutput::default();
    let encoder = FinishEncoder::default();
    let mut output =
        BufferedEncodeOutput::with_capacity(output, encoder, 3, map_error);

    // SAFETY: The full input range is valid.
    let written = unsafe {
        output
            .write_unchecked(&[0x1234], 0, 1)
            .expect("encoding should accept the value")
    };
    assert_eq!(1, written);

    output
        .flush()
        .expect("flush should only drain buffered units");
    assert_eq!(&[0x1234], output.inner().units.as_slice());

    output
        .finish()
        .expect("finish should write encoder trailer");
    assert_eq!(&[0x1234, 0xeeee], output.inner().units.as_slice());
}

#[test]
fn test_buffered_encode_output_maps_finish_capacity_bound_error() {
    let output = UnitOutput::default();
    let encoder = CapacityBoundEncoder;
    let mut output =
        BufferedEncodeOutput::with_capacity(output, encoder, 3, map_error);

    let error = output
        .finish()
        .expect_err("finish bound overflow should be mapped to I/O error");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("output length overflow"));
}

#[test]
fn test_buffered_encode_output_maps_finish_failure_variants() {
    for failure in [
        FinishFailure::Capacity,
        FinishFailure::InvalidIndex,
        FinishFailure::InsufficientOutput,
    ] {
        let output = UnitOutput::default();
        let encoder = FailingFinishEncoder { failure };
        let mut output =
            BufferedEncodeOutput::with_capacity(output, encoder, 3, map_error);

        let error = output
            .finish()
            .expect_err("finish failure should be mapped to I/O error");

        assert_eq!(ErrorKind::InvalidData, error.kind());
    }
}

#[test]
fn test_buffered_encode_output_finish_delegates_zero_width_finish() {
    let output = UnitOutput::default();
    let encoder = ZeroWidthFailingFinishEncoder;
    let mut output =
        BufferedEncodeOutput::with_capacity(output, encoder, 3, map_error);

    let error = output
        .finish()
        .expect_err("zero-width finish errors should not be skipped");
    assert_eq!(ErrorKind::InvalidData, error.kind());
}
