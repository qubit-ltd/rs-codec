// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::{Error, ErrorKind};

use qubit_codec::{
    BufferedEncodeOutput, BufferedTranscoder, CapacityError, FinishError, TranscodeProgress,
};
use qubit_io::Output;

use super::nz;

#[derive(Debug, Eq, PartialEq)]
enum PairEncodeError {
    BadInputIndex,
    BadOutputIndex,
}

#[derive(Default)]
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

#[derive(Default)]
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
            return Ok(TranscodeProgress::need_output(output_index, nz(1), 0, 0, 0));
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

#[derive(Default)]
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

#[derive(Default)]
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

fn map_error(error: PairEncodeError) -> Error {
    Error::new(ErrorKind::InvalidData, format!("{error:?}"))
}

#[test]
fn test_buffered_encode_output_encodes_and_flushes_units() {
    let output = UnitOutput::default();
    let encoder = PairEncoder;
    let mut output = BufferedEncodeOutput::with_capacity(output, encoder, 3, map_error);

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
fn test_buffered_encode_output_reports_transcoder_errors_as_io_errors() {
    let output = UnitOutput::default();
    let encoder = PairEncoder;
    let mut output = BufferedEncodeOutput::with_capacity(output, encoder, 3, map_error);
    let input = [u32::MAX];

    // SAFETY: The full input range is valid.
    let error = unsafe { output.write_unchecked(&input, 0, 1) }
        .expect_err("encoder error should be mapped to I/O error");

    assert_eq!(ErrorKind::InvalidData, error.kind());
}

#[test]
fn test_buffered_encode_output_flush_does_not_finish_encoder() {
    let output = UnitOutput::default();
    let encoder = FinishEncoder::default();
    let mut output = BufferedEncodeOutput::with_capacity(output, encoder, 3, map_error);

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
fn test_buffered_encode_output_finish_delegates_zero_width_finish() {
    let output = UnitOutput::default();
    let encoder = ZeroWidthFailingFinishEncoder;
    let mut output = BufferedEncodeOutput::with_capacity(output, encoder, 3, map_error);

    let error = output
        .finish()
        .expect_err("zero-width finish errors should not be skipped");
    assert_eq!(ErrorKind::InvalidData, error.kind());
}
