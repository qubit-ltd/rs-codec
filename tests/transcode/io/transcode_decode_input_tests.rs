// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::collections::VecDeque;
use std::io::{
    Cursor,
    Error,
    ErrorKind,
    Read,
    Seek,
    SeekFrom,
};

use qubit_codec::{
    CapacityError,
    Codec,
    TranscodeDecodeInput,
    TranscodeError,
    TranscodeProgress,
    Transcoder,
};
use qubit_io::Input;

use crate::nz;

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
enum PairDecodeError {
    #[error("bad input index")]
    BadInputIndex,
    #[error("bad output index")]
    BadOutputIndex,
    #[error("invalid output index {index} for output length {len}")]
    InvalidOutputIndex { index: usize, len: usize },
    #[error(
        "insufficient output at index {output_index}: required {required}, available {available}"
    )]
    InsufficientOutput {
        output_index: usize,
        required: usize,
        available: usize,
    },
    #[error("capacity overflow")]
    CapacityOverflow,
}

impl TranscodeError for PairDecodeError {
    fn invalid_input_index(_context: (), _index: usize, _len: usize) -> Self {
        Self::BadInputIndex
    }

    fn invalid_output_index(_context: (), index: usize, len: usize) -> Self {
        Self::InvalidOutputIndex { index, len }
    }

    fn insufficient_output(
        _context: (),
        output_index: usize,
        required: usize,
        available: usize,
    ) -> Self {
        Self::InsufficientOutput {
            output_index,
            required,
            available,
        }
    }
}

#[derive(Debug, Default)]
struct PairDecoder;

#[derive(Debug, Default)]
struct PairCodec;

unsafe impl Codec for PairCodec {
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;
    type DecodeState = ();
    type EncodeState = ();
    type Unit = u16;
    type Value = u32;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        nz(2)
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        nz(2)
    }

    unsafe fn decode(
        &mut self,
        input: &[u16],
        index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), Self::DecodeError> {
        if index + 1 >= input.len() {
            return Err(PairDecodeError::BadInputIndex);
        }
        let high = input[index] as u32;
        let low = input[index + 1] as u32;
        Ok(((high << 16) | low, nz(2)))
    }

    unsafe fn encode(
        &mut self,
        value: &u32,
        output: &mut [u16],
        index: usize,
    ) -> Result<usize, Self::EncodeError> {
        if index + 1 >= output.len() {
            return Err(PairDecodeError::BadOutputIndex);
        }
        output[index] = (value >> 16) as u16;
        output[index + 1] = *value as u16;
        Ok(2)
    }
}

impl Transcoder<u16, u32> for PairDecoder {
    type Error = PairDecodeError;
    type ErrorContext = ();

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
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

#[derive(Debug, Default)]
struct FinishDecoder {
    finished: bool,
}

impl Transcoder<u16, u32> for FinishDecoder {
    type Error = PairDecodeError;
    type ErrorContext = ();

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
    ) -> Result<usize, Self::Error> {
        if self.finished {
            return Ok(0);
        }
        if output_index >= output.len() {
            return Err(PairDecodeError::InsufficientOutput {
                output_index,
                required: 1,
                available: 0,
            });
        }
        output[output_index] = 0xfeed_beef;
        self.finished = true;
        Ok(1)
    }
}

#[derive(Debug, Default)]
struct ZeroWidthFailingFinishDecoder;

impl Transcoder<u16, u32> for ZeroWidthFailingFinishDecoder {
    type Error = PairDecodeError;
    type ErrorContext = ();

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
    ) -> Result<usize, Self::Error> {
        Err(PairDecodeError::BadInputIndex)
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

impl Transcoder<u16, u32> for TwoUnitFinishDecoder {
    type Error = PairDecodeError;
    type ErrorContext = ();

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
    ) -> Result<usize, Self::Error> {
        Self::Error::ensure_output_capacity((), output.len(), output_index, 2)?;
        output[output_index] = 0xaaaa;
        output[output_index + 1] = 0xbbbb;
        Ok(2)
    }
}

#[derive(Debug, Default)]
struct CapacityBoundDecoder;

impl Transcoder<u16, u32> for CapacityBoundDecoder {
    type Error = PairDecodeError;
    type ErrorContext = ();

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

impl Transcoder<u16, u32> for FailingTranscodeDecoder {
    type Error = PairDecodeError;
    type ErrorContext = ();

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

impl Transcoder<u16, u32> for FailingFinishDecoder {
    type Error = PairDecodeError;
    type ErrorContext = ();

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
    ) -> Result<usize, Self::Error> {
        match self.failure {
            FinishFailure::Capacity => Err(PairDecodeError::CapacityOverflow),
            FinishFailure::InvalidIndex => {
                Err(PairDecodeError::InvalidOutputIndex { index: 4, len: 1 })
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

unsafe fn decode_with<I, D>(
    input: &mut TranscodeDecodeInput<I>,
    decoder: &mut D,
    output: &mut [u32],
    output_index: usize,
    count: usize,
) -> std::io::Result<usize>
where
    I: Input<Item = u16>,
    D: Transcoder<u16, u32, Error = PairDecodeError>,
{
    let mut mapper: fn(PairDecodeError) -> Error = map_error;
    // SAFETY: The caller upholds the requested output range contract.
    unsafe {
        input.transcode_into(decoder, &mut mapper, output, output_index, count)
    }
}

unsafe fn finish_with<I, D>(
    input: &mut TranscodeDecodeInput<I>,
    decoder: &mut D,
    output: &mut [u32],
    output_index: usize,
    count: usize,
) -> std::io::Result<usize>
where
    I: Input<Item = u16>,
    D: Transcoder<u16, u32, Error = PairDecodeError>,
{
    let mut mapper: fn(PairDecodeError) -> Error = map_error;
    // SAFETY: The caller upholds the requested output range contract.
    unsafe {
        input.finish_transcode_into(
            decoder,
            &mut mapper,
            output,
            output_index,
            count,
        )
    }
}

#[test]
fn test_buffered_decode_input_exposes_parts_and_debug() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let input = TranscodeDecodeInput::with_capacity(input, 3);

    let debug = format!("{input:?}");
    assert!(debug.contains("TranscodeDecodeInput"));
    assert_eq!(1, input.inner().chunks.len());

    let (inner, unread) = input.into_parts();
    assert_eq!(1, inner.chunks.len());
    assert!(unread.is_empty());
}

#[test]
fn test_buffered_decode_input_exposes_raw_byte_read_and_seek_adapters() {
    let mut input = TranscodeDecodeInput::new(Cursor::new(vec![1, 2, 3, 4, 5]));
    input.inner_mut().set_position(0);

    let mut first = [0_u8; 1];
    let read = Read::read(&mut input, &mut first)
        .expect("raw unit read should succeed");
    assert_eq!(1, read);
    assert_eq!([1], first);

    let mut middle = [0_u8; 4];
    let read = Read::read(&mut input, &mut middle[1..3])
        .expect("raw unit read should succeed");
    assert_eq!(2, read);
    assert_eq!([0, 2, 3, 0], middle);

    let mut next = [0_u8; 1];
    assert_eq!(
        1,
        Read::read(&mut input, &mut next)
            .expect("std::io::Read should delegate to raw unit reads")
    );
    assert_eq!([4], next);

    assert_eq!(
        0,
        Seek::seek(&mut input, SeekFrom::Start(0))
            .expect("std::io::Seek should delegate to the buffered input")
    );
    let mut after_seek = [0_u8; 1];
    let read = Read::read(&mut input, &mut after_seek)
        .expect("seek should discard buffered bytes");
    assert_eq!(1, read);
    assert_eq!([1], after_seek);
}

#[test]
fn test_buffered_decode_input_returns_zero_for_zero_count() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];

    // SAFETY: The empty output range at index zero is valid.
    let read = unsafe {
        decode_with(&mut input, &mut decoder, &mut output, 0, 0)
            .expect("zero-count read should be a no-op")
    };

    assert_eq!(0, read);
    assert_eq!([0], output);
}

#[test]
fn test_buffered_decode_input_decode_into_respects_output_range() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut mapper: fn(PairDecodeError) -> Error = map_error;
    let mut output = [0_u32; 1];

    let read = unsafe {
        input.transcode_into(&mut decoder, &mut mapper, &mut output, 0, 1)
    }
    .expect("checked decode should accept a valid output range");

    assert_eq!(1, read);
    assert_eq!([0x0001_0002], output);
}

#[test]
fn test_buffered_decode_input_decode_into_accepts_codec_decoder() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003, 0x0004]]);
    let mut decoder = PairCodec;
    let mut input = TranscodeDecodeInput::with_capacity(input, 4);
    let mut mapper: fn(PairDecodeError) -> Error = map_error;
    let mut output = [0_u32; 3];

    let read = unsafe {
        input.decode_into(&mut decoder, &mut mapper, &mut output, 1, 2)
    }
    .expect("codec decoder should decode directly through the input buffer");

    assert_eq!(2, read);
    assert_eq!([0, 0x0001_0002, 0x0003_0004], output);
}

#[test]
fn test_buffered_decode_input_finish_into_accepts_codec_decoder_as_noop() {
    let input = ChunkedInput::new(Vec::new());
    let decoder = PairCodec;
    let mut input = TranscodeDecodeInput::with_capacity(input, 4);
    let mut mapper: fn(PairDecodeError) -> Error = map_error;
    let mut output = [0x1111_u32, 0x2222, 0x3333];

    let written =
        unsafe { input.finish_into(&decoder, &mut mapper, &mut output, 1, 1) }
            .expect("codec finish should be a no-op");

    assert_eq!(0, written);
    assert_eq!([0x1111, 0x2222, 0x3333], output);
}

#[test]
fn test_buffered_decode_input_decodes_across_refills() {
    let input =
        ChunkedInput::new(vec![vec![0x0001], vec![0x0002, 0x0003, 0x0004]]);
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 2];

    // SAFETY: The full output range is valid.
    let read = unsafe {
        decode_with(&mut input, &mut decoder, &mut output, 0, 2)
            .expect("decode input should produce values")
    };

    assert_eq!(2, read);
    assert_eq!([0x0001_0002, 0x0003_0004], output);
}

#[test]
fn test_buffered_decode_input_returns_partial_at_clean_eof_before_finish() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 2];

    // SAFETY: The full output range is valid.
    let read = unsafe {
        decode_with(&mut input, &mut decoder, &mut output, 0, 2)
            .expect("complete value should be returned before final EOF")
    };

    assert_eq!(1, read);
    assert_eq!(0x0001_0002, output[0]);
}

#[test]
fn test_buffered_decode_input_stops_when_output_is_full() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003, 0x0004]]);
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 4);
    let mut output = [0_u32; 1];

    // SAFETY: The full output range is valid.
    let read = unsafe {
        decode_with(&mut input, &mut decoder, &mut output, 0, 1)
            .expect("full output should stop decoding")
    };

    assert_eq!(1, read);
    assert_eq!([0x0001_0002], output);
}

#[test]
fn test_buffered_decode_input_reports_initial_refill_errors() {
    let input = FailingInput;
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];

    // SAFETY: The full output range is valid.
    let error =
        unsafe { decode_with(&mut input, &mut decoder, &mut output, 0, 1) }
            .expect_err("input refill error should be returned");

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}

#[test]
fn test_buffered_decode_input_reports_transcoder_errors() {
    let input = ChunkedInput::new(vec![vec![0x0001]]);
    let mut decoder = FailingTranscodeDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];

    // SAFETY: The full output range is valid.
    let error =
        unsafe { decode_with(&mut input, &mut decoder, &mut output, 0, 1) }
            .expect_err("decoder error should be mapped to I/O error");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("BadInputIndex"));
}

#[test]
fn test_buffered_decode_input_reports_refill_errors_after_need_input() {
    let input = ErrorAfterFirstReadInput::default();
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];

    // SAFETY: The full output range is valid.
    let error =
        unsafe { decode_with(&mut input, &mut decoder, &mut output, 0, 1) }
            .expect_err("NeedInput refill error should be returned");

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}

#[test]
fn test_buffered_decode_input_returns_partial_values_before_incomplete_eof() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003]]);
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 2];

    // SAFETY: The full output range is valid.
    let read = unsafe {
        decode_with(&mut input, &mut decoder, &mut output, 0, 2)
            .expect("partial value should be returned before EOF error")
    };
    assert_eq!(1, read);
    assert_eq!(0x0001_0002, output[0]);

    // SAFETY: The full output range is valid.
    let read = unsafe {
        decode_with(&mut input, &mut decoder, &mut output, 0, 2)
            .expect("incomplete EOF tail should stay buffered")
    };
    assert_eq!(0, read);
    assert_eq!(1, input.available());
}

#[test]
fn test_buffered_decode_input_consumes_incomplete_tail() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003]]);
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 2];

    // SAFETY: The full output range is valid.
    let read = unsafe {
        decode_with(&mut input, &mut decoder, &mut output, 0, 2)
            .expect("partial value should be returned before EOF")
    };
    assert_eq!(1, read);

    // SAFETY: The full output range is valid.
    let read = unsafe {
        decode_with(&mut input, &mut decoder, &mut output, 0, 2)
            .expect("incomplete EOF tail should stay buffered")
    };
    assert_eq!(0, read);
    assert_eq!(1, input.available());

    input.consume(1);
    assert_eq!(0, input.available());
    let available = input.available();
    input.consume(available);
    assert_eq!(0, available);
}

#[test]
fn test_buffered_decode_input_consume_available_discards_tail() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003]]);
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 2];

    // SAFETY: The full output range is valid.
    let _ = unsafe {
        decode_with(&mut input, &mut decoder, &mut output, 0, 2)
            .expect("partial value should be returned before EOF")
    };
    // SAFETY: The full output range is valid.
    let _ = unsafe {
        decode_with(&mut input, &mut decoder, &mut output, 0, 2)
            .expect("incomplete EOF tail should stay buffered")
    };

    let available = input.available();
    input.consume(available);
    assert_eq!(1, available);
    assert_eq!(0, input.available());
}

#[test]
fn test_buffered_decode_input_reports_insufficient_finish_output() {
    let input = ChunkedInput::new(Vec::new());
    let mut decoder = TwoUnitFinishDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];

    let error =
        unsafe { finish_with(&mut input, &mut decoder, &mut output, 0, 1) }
            .expect_err("one-shot finish should require the full finish bound");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("insufficient finish output"));
}

#[test]
fn test_buffered_decode_input_maps_finish_capacity_bound_error() {
    let input = ChunkedInput::new(Vec::new());
    let mut decoder = CapacityBoundDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];

    let error =
        unsafe { finish_with(&mut input, &mut decoder, &mut output, 0, 1) }
            .expect_err("finish bound overflow should be mapped to I/O error");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("output length overflow"));
}

#[test]
fn test_buffered_decode_input_maps_finish_failure_variants() {
    for failure in [FinishFailure::Capacity, FinishFailure::InvalidIndex] {
        let input = ChunkedInput::new(Vec::new());
        let mut decoder = FailingFinishDecoder { failure };
        let mut input = TranscodeDecodeInput::with_capacity(input, 3);
        let mut output = [0_u32; 1];

        let error =
            unsafe { finish_with(&mut input, &mut decoder, &mut output, 0, 1) }
                .expect_err("finish failure should be mapped to I/O error");

        assert_eq!(ErrorKind::InvalidData, error.kind());
    }
}

#[test]
fn test_buffered_decode_input_finishes_decoder_at_clean_eof() {
    let input = ChunkedInput::new(Vec::new());
    let mut decoder = FinishDecoder::default();
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];

    // SAFETY: The full output range is valid.
    let read = unsafe {
        decode_with(&mut input, &mut decoder, &mut output, 0, 1)
            .expect("clean EOF should report no decoded values")
    };
    assert_eq!(0, read);

    let read =
        unsafe { finish_with(&mut input, &mut decoder, &mut output, 0, 1) }
            .expect("caller-owned decoder should finish explicitly");
    assert_eq!(1, read);
    assert_eq!([0xfeed_beef], output);

    let read =
        unsafe { finish_with(&mut input, &mut decoder, &mut output, 0, 1) }
            .expect("finished decoder should report EOF");
    assert_eq!(0, read);
}

#[test]
fn test_buffered_decode_input_delegates_zero_width_finish_at_clean_eof() {
    let input = ChunkedInput::new(Vec::new());
    let mut decoder = ZeroWidthFailingFinishDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];

    let error =
        unsafe { finish_with(&mut input, &mut decoder, &mut output, 0, 1) }
            .expect_err("zero-width finish errors should not be skipped");
    assert_eq!(ErrorKind::InvalidData, error.kind());
}

#[test]
fn test_buffered_decode_input_takes_decoder_per_call() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003, 0x0004]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 4);
    let mut first_decoder = PairDecoder;
    let mut second_decoder = PairDecoder;
    let mut mapper: fn(PairDecodeError) -> Error = map_error;
    let mut output = [0_u32; 2];

    // SAFETY: The requested output range is valid.
    let first = unsafe {
        input
            .transcode_into(&mut first_decoder, &mut mapper, &mut output, 0, 1)
            .expect("first decoder should read one value")
    };
    // SAFETY: The requested output range is valid.
    let second = unsafe {
        input
            .transcode_into(&mut second_decoder, &mut mapper, &mut output, 1, 1)
            .expect("second decoder should continue from the same buffer")
    };

    assert_eq!(1, first);
    assert_eq!(1, second);
    assert_eq!([0x0001_0002, 0x0003_0004], output);
}
