// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::{
    Cursor,
    Error,
    ErrorKind,
    Seek,
    SeekFrom,
    Write,
};

use qubit_codec::{
    CapacityError,
    TranscodeEncodeOutput,
    TranscodeError,
    TranscodeProgress,
    Transcoder,
};
use qubit_io::Output;

use crate::nz;

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
enum PairEncodeError {
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

macro_rules! noop_reset {
    ($output:ty) => {
        fn reset(
            &mut self,
            output: &mut [$output],
            output_index: usize,
        ) -> Result<usize, TranscodeError<Self::Error>> {
            TranscodeError::<Self::Error>::ensure_output_index(
                output.len(),
                output_index,
            )?;
            Ok(0)
        }
    };
}

macro_rules! noop_finish {
    ($output:ty) => {
        fn finish(
            &mut self,
            output: &mut [$output],
            output_index: usize,
        ) -> Result<usize, TranscodeError<Self::Error>> {
            TranscodeError::<Self::Error>::ensure_output_index(
                output.len(),
                output_index,
            )?;
            Ok(0)
        }
    };
}

#[derive(Debug, Default)]
struct PairEncoder;

impl Transcoder<u32, u16> for PairEncoder {
    type Error = PairEncodeError;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        input_len
            .checked_mul(2)
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
        if input_index > input.len() {
            return Err(TranscodeError::Domain(PairEncodeError::BadInputIndex));
        }
        if output_index > output.len() {
            return Err(TranscodeError::Domain(
                PairEncodeError::BadOutputIndex,
            ));
        }
        let mut read = 0;
        let mut written = 0;
        while input_index + read < input.len() {
            if input[input_index + read] == u32::MAX {
                return Err(TranscodeError::Domain(
                    PairEncodeError::BadInputIndex,
                ));
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
    ) -> Result<usize, TranscodeError<Self::Error>> {
        Ok(0)
    }
}

#[derive(Debug, Default)]
struct FinishEncoder {
    finished: bool,
}

impl Transcoder<u32, u16> for FinishEncoder {
    type Error = PairEncodeError;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(usize::from(!self.finished))
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
        if input_index > input.len() {
            return Err(TranscodeError::Domain(PairEncodeError::BadInputIndex));
        }
        if output_index > output.len() {
            return Err(TranscodeError::Domain(
                PairEncodeError::BadOutputIndex,
            ));
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
    ) -> Result<usize, TranscodeError<Self::Error>> {
        if self.finished {
            return Ok(0);
        }
        if output_index >= output.len() {
            return Err(TranscodeError::Domain(
                PairEncodeError::InsufficientOutput {
                    output_index,
                    required: 1,
                    available: 0,
                },
            ));
        }
        output[output_index] = 0xeeee;
        self.finished = true;
        Ok(1)
    }
}

#[derive(Debug, Default)]
struct ZeroWidthFailingFinishEncoder;

impl Transcoder<u32, u16> for ZeroWidthFailingFinishEncoder {
    type Error = PairEncodeError;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(0)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
        if input_index > input.len() {
            return Err(TranscodeError::Domain(PairEncodeError::BadInputIndex));
        }
        if output_index > output.len() {
            return Err(TranscodeError::Domain(
                PairEncodeError::BadOutputIndex,
            ));
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        Err(TranscodeError::Domain(PairEncodeError::BadInputIndex))
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

impl Transcoder<u32, u16> for CapacityBoundEncoder {
    type Error = PairEncodeError;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Err(CapacityError::OutputLengthOverflow)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
        if input_index > input.len() {
            return Err(TranscodeError::Domain(PairEncodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    noop_finish!(u16);
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

impl Transcoder<u32, u16> for FailingFinishEncoder {
    type Error = PairEncodeError;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(0)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
        if input_index > input.len() {
            return Err(TranscodeError::Domain(PairEncodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        match self.failure {
            FinishFailure::Capacity => {
                Err(TranscodeError::Domain(PairEncodeError::CapacityOverflow))
            }
            FinishFailure::InvalidIndex => Err(TranscodeError::Domain(
                PairEncodeError::InvalidOutputIndex { index: 4, len: 1 },
            )),
            FinishFailure::InsufficientOutput => Err(TranscodeError::Domain(
                PairEncodeError::InsufficientOutput {
                    output_index: 0,
                    required: 2,
                    available: 1,
                },
            )),
        }
    }
}

#[derive(Debug, Default)]
struct NeedInputEncoder;

impl Transcoder<u32, u16> for NeedInputEncoder {
    type Error = PairEncodeError;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
        if input_index > input.len() {
            return Err(TranscodeError::Domain(PairEncodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::need_input(input_index, nz(1), 0, 0, 0))
    }

    noop_finish!(u16);
}

#[derive(Debug, Default)]
struct NeedOutputAfterReadEncoder;

impl Transcoder<u32, u16> for NeedOutputAfterReadEncoder {
    type Error = PairEncodeError;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        _output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
        if input_index > input.len() {
            return Err(TranscodeError::Domain(PairEncodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::need_output(output_index, nz(1), 0, 1, 0))
    }

    noop_finish!(u16);
}

#[derive(Debug, Default)]
struct NeedOutputAfterWriteEncoder;

impl Transcoder<u32, u16> for NeedOutputAfterWriteEncoder {
    type Error = PairEncodeError;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
        if input_index >= input.len() {
            return Err(TranscodeError::Domain(PairEncodeError::BadInputIndex));
        }
        output[output_index] = input[input_index] as u16;
        Ok(TranscodeProgress::need_output(
            output_index + 1,
            nz(1),
            0,
            1,
            1,
        ))
    }

    noop_finish!(u16);
}

#[derive(Debug)]
struct FixedCapacityOutput {
    units: Vec<u16>,
    flushed: bool,
    capacity: usize,
}

impl FixedCapacityOutput {
    fn new(capacity: usize) -> Self {
        Self {
            units: Vec::new(),
            flushed: false,
            capacity,
        }
    }
}

impl Output for FixedCapacityOutput {
    type Item = u16;

    unsafe fn write_unchecked(
        &mut self,
        input: &[u16],
        index: usize,
        count: usize,
    ) -> std::io::Result<usize> {
        if self.units.len() + count > self.capacity {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "fixed output capacity exceeded",
            ));
        }
        self.units.extend_from_slice(&input[index..index + count]);
        Ok(count)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flushed = true;
        Ok(())
    }
}

fn map_error(error: TranscodeError<PairEncodeError>) -> Error {
    Error::new(ErrorKind::InvalidData, format!("{error:?}"))
}

unsafe fn encode_with<E>(
    output: &mut TranscodeEncodeOutput<UnitOutput>,
    encoder: &mut E,
    input: &[u32],
    input_index: usize,
    count: usize,
) -> std::io::Result<usize>
where
    E: Transcoder<u32, u16, Error = PairEncodeError>,
{
    let mut mapper: fn(TranscodeError<PairEncodeError>) -> Error = map_error;
    // SAFETY: The caller upholds the requested input range contract.
    unsafe {
        output.transcode_from(encoder, &mut mapper, input, input_index, count)
    }
}

fn finish_with<E>(
    output: &mut TranscodeEncodeOutput<UnitOutput>,
    encoder: &mut E,
) -> std::io::Result<()>
where
    E: Transcoder<u32, u16, Error = PairEncodeError>,
{
    let mut mapper: fn(TranscodeError<PairEncodeError>) -> Error = map_error;
    output.finish(encoder, &mut mapper)
}

#[test]
fn test_buffered_encode_output_exposes_parts_and_debug() {
    let output = UnitOutput::default();
    let output = TranscodeEncodeOutput::with_capacity(output, 3);

    let debug = format!("{output:?}");
    assert!(debug.contains("TranscodeEncodeOutput"));
    assert!(output.inner().units.is_empty());

    let (inner, pending) = output.into_parts();
    assert!(inner.units.is_empty());
    assert!(pending.is_empty());
}

#[test]
fn test_buffered_encode_output_exposes_raw_byte_write_and_seek_adapters() {
    let mut output = TranscodeEncodeOutput::new(Cursor::new(Vec::new()));
    output.inner_mut().set_position(0);

    let written = Write::write(&mut output, &[1, 2])
        .expect("raw unit write should succeed");
    assert_eq!(2, written);
    let written = Write::write(&mut output, &[3, 4])
        .expect("raw unit write should succeed");
    assert_eq!(2, written);
    assert_eq!(
        1,
        Write::write(&mut output, &[5])
            .expect("std::io::Write should delegate to raw unit writes")
    );
    Write::write_all(&mut output, &[6, 7])
        .expect("std::io::Write::write_all should delegate to raw units");
    Write::flush(&mut output).expect("std::io::Write::flush should drain");
    assert_eq!(&[1, 2, 3, 4, 5, 6, 7], output.inner().get_ref().as_slice(),);

    assert_eq!(
        1,
        Seek::seek(&mut output, SeekFrom::Start(1))
            .expect("std::io::Seek should flush then delegate")
    );
    Write::write_all(&mut output, &[8])
        .expect("write after seek should update the wrapped cursor");
    output.flush().expect("flush should drain after seek");
    assert_eq!(&[1, 8, 3, 4, 5, 6, 7], output.inner().get_ref().as_slice(),);
}

#[test]
fn test_buffered_encode_output_returns_zero_for_zero_count() {
    let output = UnitOutput::default();
    let mut encoder = PairEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);

    // SAFETY: The empty input range at index zero is valid.
    let written = unsafe {
        encode_with(&mut output, &mut encoder, &[0x0001_0002], 0, 0)
            .expect("zero-count write should be a no-op")
    };

    assert_eq!(0, written);
    assert!(output.inner().units.is_empty());
}

#[test]
fn test_buffered_encode_output_transcode_from_respects_input_range() {
    let output = UnitOutput::default();
    let mut encoder = PairEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 4);
    let mut mapper: fn(TranscodeError<PairEncodeError>) -> Error = map_error;

    let written = unsafe {
        output.transcode_from(&mut encoder, &mut mapper, &[0x0001_0002], 0, 1)
    }
    .expect("checked encode should accept a valid input range");

    output.flush().expect("flush should drain encoded units");

    assert_eq!(1, written);
    assert_eq!(&[1, 2], output.inner().units.as_slice());
}

#[test]
fn test_buffered_encode_output_encodes_and_flushes_units() {
    let output = UnitOutput::default();
    let mut encoder = PairEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);

    // SAFETY: The full input range is valid.
    let written = unsafe {
        encode_with(
            &mut output,
            &mut encoder,
            &[0x0001_0002, 0x0003_0004],
            0,
            2,
        )
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
    let mut encoder = PairEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 2);

    // SAFETY: The full input range is valid.
    let written = unsafe {
        encode_with(&mut output, &mut encoder, &[0x0001_0002], 0, 1)
            .expect("first value should fill the unit buffer")
    };
    assert_eq!(1, written);
    assert!(output.inner().units.is_empty());

    // SAFETY: The full input range is valid.
    let written = unsafe {
        encode_with(&mut output, &mut encoder, &[0x0003_0004], 0, 1)
            .expect("second value should flush the full buffer first")
    };
    assert_eq!(1, written);

    output.flush().expect("flush should drain buffered units");
    assert_eq!(&[1, 2, 3, 4], output.inner().units.as_slice());
}

#[test]
fn test_buffered_encode_output_reports_no_progress_need_output_capacity() {
    let output = UnitOutput::default();
    let mut encoder = PairEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 1);

    // SAFETY: The full input range is valid.
    let error =
        unsafe { encode_with(&mut output, &mut encoder, &[0x0001_0002], 0, 1) }
            .expect_err(
                "insufficient fixed buffer capacity should be reported",
            );

    assert_eq!(ErrorKind::InvalidInput, error.kind());
    assert!(error.to_string().contains("spare capacity"));
}

#[test]
fn test_buffered_encode_output_returns_after_need_output_consumes_input() {
    let output = UnitOutput::default();
    let mut encoder = NeedOutputAfterReadEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 1);

    // SAFETY: The full input range is valid.
    let written = unsafe {
        encode_with(&mut output, &mut encoder, &[0x1234], 0, 1)
            .expect("need-output after consuming input should return progress")
    };

    assert_eq!(1, written);
}

#[test]
fn test_buffered_encode_output_reports_transcoder_errors_as_io_errors() {
    let output = UnitOutput::default();
    let mut encoder = PairEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);
    let input = [u32::MAX];

    // SAFETY: The full input range is valid.
    let error = unsafe { encode_with(&mut output, &mut encoder, &input, 0, 1) }
        .expect_err("encoder error should be mapped to I/O error");

    assert_eq!(ErrorKind::InvalidData, error.kind());
}

#[test]
fn test_buffered_encode_output_rejects_need_input_status() {
    let output = UnitOutput::default();
    let mut encoder = NeedInputEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);

    // SAFETY: The full input range is valid.
    let error =
        unsafe { encode_with(&mut output, &mut encoder, &[0x1234], 0, 1) }
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
    let mut encoder = FinishEncoder::default();
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);

    // SAFETY: The full input range is valid.
    let written = unsafe {
        encode_with(&mut output, &mut encoder, &[0x1234], 0, 1)
            .expect("encoding should accept the value")
    };
    assert_eq!(1, written);

    output
        .flush()
        .expect("flush should only drain buffered units");
    assert_eq!(&[0x1234], output.inner().units.as_slice());

    finish_with(&mut output, &mut encoder)
        .expect("finish should write encoder trailer");
    assert_eq!(&[0x1234, 0xeeee], output.inner().units.as_slice());
}

#[test]
fn test_buffered_encode_output_finish_writes_and_flushes() {
    let output = UnitOutput::default();
    let mut encoder = FinishEncoder::default();
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);

    finish_with(&mut output, &mut encoder)
        .expect("finish should write trailer and flush");

    assert_eq!(&[0xeeee], output.inner().units.as_slice());
    assert!(output.inner().flushed);
    output.inner_mut().flushed = false;
    output
        .flush()
        .expect("explicit flush should be harmless after finish");
    assert_eq!(&[0xeeee], output.inner().units.as_slice());
}

#[test]
fn test_buffered_encode_output_maps_finish_capacity_bound_error() {
    let output = UnitOutput::default();
    let mut encoder = CapacityBoundEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);

    let error = finish_with(&mut output, &mut encoder)
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
        let mut encoder = FailingFinishEncoder { failure };
        let mut output = TranscodeEncodeOutput::with_capacity(output, 3);

        let error = finish_with(&mut output, &mut encoder)
            .expect_err("finish failure should be mapped to I/O error");

        assert_eq!(ErrorKind::InvalidData, error.kind());
    }
}

#[test]
fn test_buffered_encode_output_finish_delegates_zero_width_finish() {
    let output = UnitOutput::default();
    let mut encoder = ZeroWidthFailingFinishEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);

    let error = finish_with(&mut output, &mut encoder)
        .expect_err("zero-width finish errors should not be skipped");
    assert_eq!(ErrorKind::InvalidData, error.kind());
}

#[test]
fn test_buffered_encode_output_takes_encoder_per_call() {
    let output = UnitOutput::default();
    let mut output = TranscodeEncodeOutput::with_capacity(output, 4);
    let mut first_encoder = PairEncoder;
    let mut second_encoder = PairEncoder;
    let mut mapper: fn(TranscodeError<PairEncodeError>) -> Error = map_error;

    // SAFETY: The requested input range is valid.
    let first = unsafe {
        output
            .transcode_from(
                &mut first_encoder,
                &mut mapper,
                &[0x0001_0002],
                0,
                1,
            )
            .expect("first encoder should write one value")
    };
    // SAFETY: The requested input range is valid.
    let second = unsafe {
        output
            .transcode_from(
                &mut second_encoder,
                &mut mapper,
                &[0x0003_0004],
                0,
                1,
            )
            .expect("second encoder should reuse the same buffer")
    };

    output.flush().expect("flush should drain buffered units");

    assert_eq!(1, first);
    assert_eq!(1, second);
    assert_eq!(&[1, 2, 3, 4], output.inner().units.as_slice());
}

#[test]
fn test_buffered_encode_output_exposes_spare_buffer_api() {
    let output = UnitOutput::default();
    let mut output = TranscodeEncodeOutput::with_capacity(output, 4);

    assert!(output.spare_capacity() >= 4);

    let (units, index, available) = output.spare_raw_parts_mut();
    assert!(available >= 4);
    units[index] = 0x00aa;
    units[index + 1] = 0x00bb;
    // SAFETY: Two initialized units were written inside the reserved spare
    // range.
    unsafe {
        output.advance(2);
    }
    output
        .ensure_spare_capacity(2)
        .expect("spare capacity should remain available");
    output.flush().expect("flush should drain spare units");
    assert_eq!(&[0x00aa, 0x00bb], output.inner().units.as_slice());
}

#[test]
fn test_buffered_encode_output_transcode_from_flushes_when_spare_is_empty() {
    let output = UnitOutput::default();
    let mut encoder = PairEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 2);

    // SAFETY: The full input range is valid.
    let first = unsafe {
        encode_with(&mut output, &mut encoder, &[0x0001_0002], 0, 1)
            .expect("first value should fill the spare buffer")
    };
    assert_eq!(1, first);
    assert_eq!(0, output.spare_capacity());

    // SAFETY: The full input range is valid.
    let second = unsafe {
        encode_with(&mut output, &mut encoder, &[0x0003_0004], 0, 1)
            .expect("second value should flush before encoding")
    };
    assert_eq!(1, second);
    output.flush().expect("flush should drain buffered units");
    assert_eq!(&[1, 2, 3, 4], output.inner().units.as_slice());
}

#[test]
fn test_buffered_encode_output_flushes_after_partial_need_output_progress() {
    let output = UnitOutput::default();
    let mut encoder = NeedOutputAfterWriteEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 2);

    // SAFETY: The full input range is valid.
    let written = unsafe {
        encode_with(&mut output, &mut encoder, &[0x1234], 0, 1)
            .expect("partial need-output progress should flush buffered units")
    };

    assert_eq!(1, written);
    output.flush().expect("flush should drain buffered units");
    assert_eq!(&[0x1234], output.inner().units.as_slice());
}

#[test]
fn test_buffered_encode_output_finish_reports_spare_capacity_error() {
    let output = FixedCapacityOutput::new(0);
    let mut encoder = FinishEncoder::default();
    let mut output = TranscodeEncodeOutput::with_capacity(output, 1);
    let mut mapper: fn(TranscodeError<PairEncodeError>) -> Error = map_error;

    let error = output
        .finish(&mut encoder, &mut mapper)
        .expect_err("finish should report spare-capacity errors");

    assert_eq!(ErrorKind::InvalidInput, error.kind());
}
