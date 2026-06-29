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
    DecodeFailure,
    TranscodeDecodeInput,
    TranscodeError,
    TranscodeProgress,
    Transcoder,
};
use qubit_io::Input;

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

fn domain(error: PairDecodeError) -> TranscodeError<PairDecodeError> {
    TranscodeError::domain(error, qubit_codec::CodecPhase::Main, None)
}

#[derive(Debug, Default)]
struct FixedPairCodec;

impl Codec for FixedPairCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("fixed pair width is non-zero");
    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("fixed pair width is non-zero");

    unsafe fn decode(
        &mut self,
        input: &[u16],
        input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        let available = input.len().saturating_sub(input_index);
        if available < 2 {
            return Err(DecodeFailure::incomplete(crate::nz(2)));
        }
        let high = input[input_index] as u32;
        let low = input[input_index + 1] as u32;
        Ok(((high << 16) | low, crate::nz(2)))
    }

    unsafe fn encode(
        &mut self,
        value: &u32,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[output_index] = (value >> 16) as u16;
        output[output_index + 1] = *value as u16;
        Ok(crate::nz(2))
    }
}

macro_rules! noop_reset {
    ($output:ty) => {
        fn reset(
            &mut self,
            output: &mut [$output],
            output_index: usize,
        ) -> Result<usize, Self::Error> {
            TranscodeError::<Self::DomainError>::ensure_output_index(
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
        ) -> Result<usize, Self::Error> {
            TranscodeError::<Self::DomainError>::ensure_output_index(
                output.len(),
                output_index,
            )?;
            Ok(0)
        }
    };
}

#[derive(Debug, Default)]
struct PairDecoder;

#[test]
fn test_transcode_decode_input_exposes_unread_window() {
    let mut input = TranscodeDecodeInput::with_capacity(
        ChunkedInput::new(vec![vec![1_u16, 2, 3]]),
        3,
    );

    assert!(input.fill_until(2).expect("fill should succeed"));
    assert_eq!(&[1, 2, 3], input.unread());

    input.consume(2);
    assert_eq!(&[3], input.unread());
}

impl Transcoder<u16, u32> for PairDecoder {
    type Error = TranscodeError<PairDecodeError>;
    type DomainError = PairDecodeError;

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
        Ok(input_len / 2)
    }

    noop_reset!(u32);

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        output: &mut [u32],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        if output_index > output.len() {
            return Err(domain(PairDecodeError::BadOutputIndex));
        }
        let mut read = 0;
        let mut written = 0;
        while input_index + read + 1 < input.len() {
            if output_index + written == output.len() {
                return Ok(TranscodeProgress::need_output(
                    output_index + written,
                    crate::nz(1),
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
                crate::nz(2),
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
    type Error = TranscodeError<PairDecodeError>;
    type DomainError = PairDecodeError;

    fn map_error(
        &self,
        error: TranscodeError<Self::DomainError>,
    ) -> Self::Error {
        error
    }

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(usize::from(!self.finished))
    }

    noop_reset!(u32);

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        if output_index > 0 {
            return Err(domain(PairDecodeError::BadOutputIndex));
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
            return Err(domain(PairDecodeError::InsufficientOutput {
                output_index,
                required: 1,
                available: 0,
            }));
        }
        output[output_index] = 0xfeed_beef;
        self.finished = true;
        Ok(1)
    }
}

#[derive(Debug, Default)]
struct ZeroWidthFailingFinishDecoder;

impl Transcoder<u16, u32> for ZeroWidthFailingFinishDecoder {
    type Error = TranscodeError<PairDecodeError>;
    type DomainError = PairDecodeError;

    fn map_error(
        &self,
        error: TranscodeError<Self::DomainError>,
    ) -> Self::Error {
        error
    }

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(0)
    }

    noop_reset!(u32);

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        if output_index > 0 {
            return Err(domain(PairDecodeError::BadOutputIndex));
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(
        &mut self,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        Err(domain(PairDecodeError::BadInputIndex))
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
    type Error = TranscodeError<PairDecodeError>;
    type DomainError = PairDecodeError;

    fn map_error(
        &self,
        error: TranscodeError<Self::DomainError>,
    ) -> Self::Error {
        error
    }

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(2)
    }

    noop_reset!(u32);

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(
        &mut self,
        output: &mut [u32],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        TranscodeError::<Self::DomainError>::ensure_output_capacity(
            output.len(),
            output_index,
            2,
        )?;
        output[output_index] = 0xaaaa;
        output[output_index + 1] = 0xbbbb;
        Ok(2)
    }
}

#[derive(Debug, Default)]
struct CapacityBoundDecoder;

impl Transcoder<u16, u32> for CapacityBoundDecoder {
    type Error = TranscodeError<PairDecodeError>;
    type DomainError = PairDecodeError;

    fn map_error(
        &self,
        error: TranscodeError<Self::DomainError>,
    ) -> Self::Error {
        error
    }

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Err(CapacityError::OutputLengthOverflow)
    }

    noop_reset!(u32);

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    noop_finish!(u32);
}

#[derive(Debug, Default)]
struct FailingTranscodeDecoder;

impl Transcoder<u16, u32> for FailingTranscodeDecoder {
    type Error = TranscodeError<PairDecodeError>;
    type DomainError = PairDecodeError;

    fn map_error(
        &self,
        error: TranscodeError<Self::DomainError>,
    ) -> Self::Error {
        error
    }

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    noop_reset!(u32);

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        Err(domain(PairDecodeError::BadInputIndex))
    }

    noop_finish!(u32);
}

#[derive(Debug, Default)]
struct OverreadingProgressDecoder;

impl Transcoder<u16, u32> for OverreadingProgressDecoder {
    type Error = TranscodeError<PairDecodeError>;
    type DomainError = PairDecodeError;

    fn map_error(
        &self,
        error: TranscodeError<Self::DomainError>,
    ) -> Self::Error {
        error
    }

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    noop_reset!(u32);

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::complete(input.len() + 1, 0))
    }

    noop_finish!(u32);
}

#[derive(Debug, Default)]
struct OverwritingProgressDecoder;

impl Transcoder<u16, u32> for OverwritingProgressDecoder {
    type Error = TranscodeError<PairDecodeError>;
    type DomainError = PairDecodeError;

    fn map_error(
        &self,
        error: TranscodeError<Self::DomainError>,
    ) -> Self::Error {
        error
    }

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(2)
    }

    noop_reset!(u32);

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::complete(0, output_index + 2))
    }

    noop_finish!(u32);
}

#[cfg(debug_assertions)]
#[derive(Debug, Default)]
struct OverflowingNeedInputDecoder;

#[cfg(debug_assertions)]
impl Transcoder<u16, u32> for OverflowingNeedInputDecoder {
    type Error = TranscodeError<PairDecodeError>;
    type DomainError = PairDecodeError;

    fn map_error(
        &self,
        error: TranscodeError<Self::DomainError>,
    ) -> Self::Error {
        error
    }

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    noop_reset!(u32);

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::need_input(
            input_index,
            crate::nz(1),
            input.len() - input_index,
            0,
            0,
        ))
    }

    noop_finish!(u32);
}

#[cfg(debug_assertions)]
#[derive(Debug, Default)]
struct MisindexedNeedInputDecoder;

#[cfg(debug_assertions)]
impl Transcoder<u16, u32> for MisindexedNeedInputDecoder {
    type Error = TranscodeError<PairDecodeError>;
    type DomainError = PairDecodeError;

    fn map_error(
        &self,
        error: TranscodeError<Self::DomainError>,
    ) -> Self::Error {
        error
    }

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    noop_reset!(u32);

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::need_input(
            input_index + 1,
            crate::nz(1),
            1,
            0,
            0,
        ))
    }

    noop_finish!(u32);
}

#[cfg(debug_assertions)]
#[derive(Debug, Default)]
struct MisindexedNeedOutputDecoder;

#[cfg(debug_assertions)]
impl Transcoder<u16, u32> for MisindexedNeedOutputDecoder {
    type Error = TranscodeError<PairDecodeError>;
    type DomainError = PairDecodeError;

    fn map_error(
        &self,
        error: TranscodeError<Self::DomainError>,
    ) -> Self::Error {
        error
    }

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    noop_reset!(u32);

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::need_output(
            output_index + 1,
            crate::nz(1),
            0,
            0,
            0,
        ))
    }

    noop_finish!(u32);
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
    type Error = TranscodeError<PairDecodeError>;
    type DomainError = PairDecodeError;

    fn map_error(
        &self,
        error: TranscodeError<Self::DomainError>,
    ) -> Self::Error {
        error
    }

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(0)
    }

    noop_reset!(u32);

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(
        &mut self,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        match self.failure {
            FinishFailure::Capacity => {
                Err(domain(PairDecodeError::CapacityOverflow))
            }
            FinishFailure::InvalidIndex => {
                Err(domain(PairDecodeError::InvalidOutputIndex {
                    index: 4,
                    len: 1,
                }))
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

fn map_error(error: TranscodeError<PairDecodeError>) -> Error {
    Error::new(ErrorKind::InvalidData, format!("{error:?}"))
}

fn map_codec_error(error: PairDecodeError) -> Error {
    Error::new(ErrorKind::InvalidData, error)
}

fn decode_with<I, D>(
    input: &mut TranscodeDecodeInput<I>,
    decoder: &mut D,
    output: &mut [u32],
    output_index: usize,
    count: usize,
) -> std::io::Result<usize>
where
    I: Input<Item = u16>,
    D: Transcoder<u16, u32, Error = TranscodeError<PairDecodeError>>,
{
    let mut mapper: fn(TranscodeError<PairDecodeError>) -> Error = map_error;
    input.transcode_into(decoder, &mut mapper, output, output_index, count)
}

fn finish_with<I, D>(
    input: &mut TranscodeDecodeInput<I>,
    decoder: &mut D,
    output: &mut [u32],
    output_index: usize,
    count: usize,
) -> std::io::Result<usize>
where
    I: Input<Item = u16>,
    D: Transcoder<u16, u32, Error = TranscodeError<PairDecodeError>>,
{
    let mut mapper: fn(TranscodeError<PairDecodeError>) -> Error = map_error;
    input.finish_transcode_into(
        decoder,
        &mut mapper,
        output,
        output_index,
        count,
    )
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
fn test_buffered_decode_input_reads_one_codec_value() {
    let input = ChunkedInput::new(vec![vec![0x1234], vec![0x5678, 0x9abc]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 2);
    let mut codec = FixedPairCodec;

    let value = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect("one codec value should decode across refills");

    assert_eq!(0x1234_5678, value);
    assert!(input.unread().is_empty());
    assert!(
        input.fill_until(1).expect("tail refill should succeed"),
        "tail unit should remain readable after one value"
    );
    assert_eq!(&[0x9abc], input.unread());
}

#[test]
fn test_buffered_decode_input_returns_zero_for_zero_count() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];
    let read = decode_with(&mut input, &mut decoder, &mut output, 0, 0)
        .expect("zero-count read should be a no-op");

    assert_eq!(0, read);
    assert_eq!([0], output);
}

#[test]
fn test_buffered_decode_input_transcode_into_respects_output_range() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut mapper: fn(TranscodeError<PairDecodeError>) -> Error = map_error;
    let mut output = [0_u32; 1];

    let read = input
        .transcode_into(&mut decoder, &mut mapper, &mut output, 0, 1)
        .expect("checked decode should accept a valid output range");

    assert_eq!(1, read);
    assert_eq!([0x0001_0002], output);
}

#[test]
fn test_buffered_decode_input_transcode_into_rejects_invalid_output_range() {
    let input = ChunkedInput::new(Vec::new());
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut mapper: fn(TranscodeError<PairDecodeError>) -> Error = map_error;
    let mut output = [0_u32; 1];

    let error = input
        .transcode_into(&mut decoder, &mut mapper, &mut output, 1, 1)
        .expect_err("invalid output range should be rejected before decoding");

    assert_eq!(ErrorKind::InvalidInput, error.kind());
    assert_eq!(
        "decoded output range exceeds destination buffer",
        error.to_string(),
    );
}

#[test]
fn test_buffered_decode_input_decodes_across_refills() {
    let input =
        ChunkedInput::new(vec![vec![0x0001], vec![0x0002, 0x0003, 0x0004]]);
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 2];
    let read = decode_with(&mut input, &mut decoder, &mut output, 0, 2)
        .expect("decode input should produce values");

    assert_eq!(2, read);
    assert_eq!([0x0001_0002, 0x0003_0004], output);
}

#[test]
fn test_buffered_decode_input_returns_partial_at_clean_eof_before_finish() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 2];
    let read = decode_with(&mut input, &mut decoder, &mut output, 0, 2)
        .expect("complete value should be returned before final EOF");

    assert_eq!(1, read);
    assert_eq!(0x0001_0002, output[0]);
}

#[test]
fn test_buffered_decode_input_stops_when_output_is_full() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003, 0x0004]]);
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 4);
    let mut output = [0_u32; 1];
    let read = decode_with(&mut input, &mut decoder, &mut output, 0, 1)
        .expect("full output should stop decoding");

    assert_eq!(1, read);
    assert_eq!([0x0001_0002], output);
}

#[test]
fn test_buffered_decode_input_reports_initial_refill_errors() {
    let input = FailingInput;
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];
    let error = decode_with(&mut input, &mut decoder, &mut output, 0, 1)
        .expect_err("input refill error should be returned");

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}

#[test]
fn test_buffered_decode_input_reports_transcoder_errors() {
    let input = ChunkedInput::new(vec![vec![0x0001]]);
    let mut decoder = FailingTranscodeDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];
    let error = decode_with(&mut input, &mut decoder, &mut output, 0, 1)
        .expect_err("decoder error should be mapped to I/O error");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("BadInputIndex"));
}

#[test]
fn test_buffered_decode_input_rejects_overreported_read_progress() {
    let input = ChunkedInput::new(vec![vec![0x0001]]);
    let mut decoder = OverreadingProgressDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];
    let error = decode_with(&mut input, &mut decoder, &mut output, 0, 1)
        .expect_err("overreported input progress should be rejected");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("consumed"));
    assert!(error.to_string().contains("only"));
}

#[test]
fn test_buffered_decode_input_rejects_overreported_write_progress() {
    let input = ChunkedInput::new(vec![vec![0x0001]]);
    let mut decoder = OverwritingProgressDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];
    let error = decode_with(&mut input, &mut decoder, &mut output, 0, 1)
        .expect_err("overreported output progress should be rejected");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("wrote"));
    assert!(error.to_string().contains("output slots"));
}

#[cfg(debug_assertions)]
#[test]
fn test_buffered_decode_input_rejects_overflowing_need_input() {
    let input = ChunkedInput::new(vec![vec![0x0001]]);
    let mut decoder = OverflowingNeedInputDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];
    let error = decode_with(&mut input, &mut decoder, &mut output, 0, 1)
        .expect_err("satisfied NeedInput requirement should be rejected");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("reported required"));
}

#[cfg(debug_assertions)]
#[test]
fn test_buffered_decode_input_rejects_misindexed_need_input() {
    let input = ChunkedInput::new(vec![vec![0x0001]]);
    let mut decoder = MisindexedNeedInputDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];
    let error = decode_with(&mut input, &mut decoder, &mut output, 0, 1)
        .expect_err("misindexed NeedInput status should be rejected");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("reported status index"));
}

#[cfg(debug_assertions)]
#[test]
fn test_buffered_decode_input_rejects_misindexed_need_output() {
    let input = ChunkedInput::new(vec![vec![0x0001]]);
    let mut decoder = MisindexedNeedOutputDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];
    let error = decode_with(&mut input, &mut decoder, &mut output, 0, 1)
        .expect_err("misindexed NeedOutput status should be rejected");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("reported status index"));
}

#[test]
fn test_buffered_decode_input_reports_refill_errors_after_need_input() {
    let input = ErrorAfterFirstReadInput::default();
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];
    let error = decode_with(&mut input, &mut decoder, &mut output, 0, 1)
        .expect_err("NeedInput refill error should be returned");

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}

#[test]
fn test_buffered_decode_input_returns_partial_values_before_incomplete_eof() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003]]);
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 2];
    let read = decode_with(&mut input, &mut decoder, &mut output, 0, 2)
        .expect("partial value should be returned before EOF error");
    assert_eq!(1, read);
    assert_eq!(0x0001_0002, output[0]);
    let read = decode_with(&mut input, &mut decoder, &mut output, 0, 2)
        .expect("incomplete EOF tail should stay buffered");
    assert_eq!(0, read);
    assert_eq!(1, input.unread_len());
}

#[test]
fn test_buffered_decode_input_consumes_incomplete_tail() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003]]);
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 2];
    let read = decode_with(&mut input, &mut decoder, &mut output, 0, 2)
        .expect("partial value should be returned before EOF");
    assert_eq!(1, read);
    let read = decode_with(&mut input, &mut decoder, &mut output, 0, 2)
        .expect("incomplete EOF tail should stay buffered");
    assert_eq!(0, read);
    assert_eq!(1, input.unread_len());

    input.consume(1);
    assert_eq!(0, input.unread_len());
    let available = input.unread_len();
    input.consume(available);
    assert_eq!(0, available);
}

#[test]
fn test_buffered_decode_input_consume_available_discards_tail() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003]]);
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 2];
    let _ = decode_with(&mut input, &mut decoder, &mut output, 0, 2)
        .expect("partial value should be returned before EOF");
    let _ = decode_with(&mut input, &mut decoder, &mut output, 0, 2)
        .expect("incomplete EOF tail should stay buffered");

    let available = input.unread_len();
    input.consume(available);
    assert_eq!(1, available);
    assert_eq!(0, input.unread_len());
}

#[test]
fn test_buffered_decode_input_reports_insufficient_finish_output() {
    let input = ChunkedInput::new(Vec::new());
    let mut decoder = TwoUnitFinishDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];

    let error = finish_with(&mut input, &mut decoder, &mut output, 0, 1)
        .expect_err("one-shot finish should require the full finish bound");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("insufficient output"));
}

#[test]
fn test_buffered_decode_input_rejects_finish_count_below_finish_bound() {
    let input = ChunkedInput::new(Vec::new());
    let mut decoder = TwoUnitFinishDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 2];

    let error = finish_with(&mut input, &mut decoder, &mut output, 0, 1)
        .expect_err("count must cap the finish output range");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("insufficient output"));
    assert_eq!([0, 0], output);
}

#[test]
fn test_buffered_decode_input_finish_rejects_invalid_output_range() {
    let input = ChunkedInput::new(Vec::new());
    let mut decoder = FinishDecoder::default();
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut mapper: fn(TranscodeError<PairDecodeError>) -> Error = map_error;
    let mut output = [0_u32; 1];

    let error = input
        .finish_transcode_into(&mut decoder, &mut mapper, &mut output, 1, 1)
        .expect_err("invalid finish output range should be rejected");

    assert_eq!(ErrorKind::InvalidInput, error.kind());
    assert_eq!(
        "finish output range exceeds destination buffer",
        error.to_string(),
    );
}

#[test]
fn test_buffered_decode_input_maps_finish_capacity_bound_error() {
    let input = ChunkedInput::new(Vec::new());
    let mut decoder = CapacityBoundDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];

    let error = finish_with(&mut input, &mut decoder, &mut output, 0, 1)
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

        let error = finish_with(&mut input, &mut decoder, &mut output, 0, 1)
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
    let read = decode_with(&mut input, &mut decoder, &mut output, 0, 1)
        .expect("clean EOF should report no decoded values");
    assert_eq!(0, read);

    let read = finish_with(&mut input, &mut decoder, &mut output, 0, 1)
        .expect("caller-owned decoder should finish explicitly");
    assert_eq!(1, read);
    assert_eq!([0xfeed_beef], output);

    let read = finish_with(&mut input, &mut decoder, &mut output, 0, 1)
        .expect("finished decoder should report EOF");
    assert_eq!(0, read);
}

#[test]
fn test_buffered_decode_input_delegates_zero_width_finish_at_clean_eof() {
    let input = ChunkedInput::new(Vec::new());
    let mut decoder = ZeroWidthFailingFinishDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut output = [0_u32; 1];

    let error = finish_with(&mut input, &mut decoder, &mut output, 0, 1)
        .expect_err("zero-width finish errors should not be skipped");
    assert_eq!(ErrorKind::InvalidData, error.kind());
}

#[test]
fn test_buffered_decode_input_takes_decoder_per_call() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003, 0x0004]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 4);
    let mut first_decoder = PairDecoder;
    let mut second_decoder = PairDecoder;
    let mut mapper: fn(TranscodeError<PairDecodeError>) -> Error = map_error;
    let mut output = [0_u32; 2];
    let first = input
        .transcode_into(&mut first_decoder, &mut mapper, &mut output, 0, 1)
        .expect("first decoder should read one value");
    let second = input
        .transcode_into(&mut second_decoder, &mut mapper, &mut output, 1, 1)
        .expect("second decoder should continue from the same buffer");

    assert_eq!(1, first);
    assert_eq!(1, second);
    assert_eq!([0x0001_0002, 0x0003_0004], output);
}

#[test]
fn test_buffered_decode_input_exposes_buffer_capacity_and_fill_until() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 4);

    assert!(input.capacity() >= 4);
    assert_eq!(0, input.unread_len());

    let filled = input
        .fill_until(2)
        .expect("fill should read buffered units");
    assert!(filled);
    assert_eq!(2, input.unread_len());
}

#[test]
fn test_buffered_decode_input_copy_unread_and_read_unchecked() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 4);
    assert!(input.fill_until(3).expect("fill should succeed"));

    let mut copied = [0_u16; 3];
    // SAFETY: The destination range is valid and does not overlap the buffer.
    unsafe {
        input.copy_unread_to(&mut copied, 0, 2);
    }
    assert_eq!([0x0001, 0x0002, 0], copied);
    assert_eq!(3, input.unread_len());

    let mut read = [0_u16; 2];
    // SAFETY: The destination range is valid.
    let read_count = unsafe { input.read_unchecked(&mut read, 0, 2) }
        .expect("read should copy unread units");
    assert_eq!(2, read_count);
    assert_eq!([0x0001, 0x0002], read);
    assert_eq!(1, input.unread_len());
}

#[derive(Debug, Default)]
struct InvalidPairReadCodec;

impl Codec for InvalidPairReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");
    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");

    unsafe fn decode(
        &mut self,
        input: &[u16],
        input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        let _ = input[input_index];
        Err(DecodeFailure::invalid(
            PairDecodeError::BadInputIndex,
            crate::nz(1),
        ))
    }

    unsafe fn encode(
        &mut self,
        _value: &u32,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Ok(crate::nz(2))
    }
}

#[derive(Debug, Default)]
struct GrowingPairReadCodec {
    pass: bool,
}

impl Codec for GrowingPairReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");
    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(4).expect("variable width");

    unsafe fn decode(
        &mut self,
        input: &[u16],
        input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        let available = input.len().saturating_sub(input_index);
        if !self.pass && available < 4 {
            return Err(DecodeFailure::incomplete(crate::nz(4)));
        }
        self.pass = true;
        let high = input[input_index] as u32;
        let low = input[input_index + 1] as u32;
        Ok(((high << 16) | low, crate::nz(2)))
    }

    unsafe fn encode(
        &mut self,
        _value: &u32,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Ok(crate::nz(2))
    }
}

#[derive(Debug, Default)]
struct OverconsumeReadCodec;

impl Codec for OverconsumeReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");
    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");

    unsafe fn decode(
        &mut self,
        _input: &[u16],
        _input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        Ok((0, core::num::NonZeroUsize::new(3).expect("three units")))
    }

    unsafe fn encode(
        &mut self,
        _value: &u32,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Ok(crate::nz(2))
    }
}

#[test]
fn test_buffered_decode_input_read_decoded_reports_unexpected_eof() {
    let input = ChunkedInput::new(Vec::<Vec<u16>>::new());
    let mut input = TranscodeDecodeInput::with_capacity(input, 2);
    let mut codec = FixedPairCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err("empty input should fail before a complete value");

    assert_eq!(ErrorKind::UnexpectedEof, error.kind());
}

#[test]
fn test_buffered_decode_input_read_decoded_maps_invalid_input() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 2);
    let mut codec = InvalidPairReadCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err("invalid codec input should be mapped");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert_eq!("bad input index", error.to_string());
}

#[test]
fn test_buffered_decode_input_read_decoded_uses_scratch_when_value_exceeds_capacity()
 {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003, 0x0004]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = FixedPairCodec;

    let value = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect("scratch decode should succeed across refills");

    assert_eq!(0x0001_0002, value);
}

#[test]
fn test_buffered_decode_input_read_decoded_refills_after_incomplete() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003, 0x0004]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 4);
    let mut codec = GrowingPairReadCodec::default();

    let value = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect("incomplete input should refill before decoding");

    assert_eq!(0x0001_0002, value);
}

#[test]
fn test_buffered_decode_input_read_decoded_rejects_overconsuming_codec() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 2);
    let mut codec = OverconsumeReadCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err("codec over-consumption should be rejected");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(
        error
            .to_string()
            .contains("codec consumed units exceed unread window")
    );
}

#[derive(Debug, Default)]
struct PartialWindowIncompleteCodec;

impl Codec for PartialWindowIncompleteCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");
    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(4).expect("variable width");

    unsafe fn decode(
        &mut self,
        input: &[u16],
        input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        let available = input.len().saturating_sub(input_index);
        if available < 4 {
            return Err(DecodeFailure::incomplete(crate::nz(4)));
        }
        let high = input[input_index] as u32;
        let low = input[input_index + 1] as u32;
        Ok(((high << 16) | low, crate::nz(2)))
    }

    unsafe fn encode(
        &mut self,
        _value: &u32,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Ok(crate::nz(2))
    }
}

#[derive(Debug, Default)]
struct OverconsumeInvalidReadCodec;

impl Codec for OverconsumeInvalidReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");
    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");

    unsafe fn decode(
        &mut self,
        _input: &[u16],
        _input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        Err(DecodeFailure::invalid(
            PairDecodeError::BadInputIndex,
            core::num::NonZeroUsize::new(3).expect("three units"),
        ))
    }

    unsafe fn encode(
        &mut self,
        _value: &u32,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Ok(crate::nz(2))
    }
}

#[derive(Debug, Default)]
struct ScratchGrowingReadCodec;

impl Codec for ScratchGrowingReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");
    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(4).expect("variable width");

    unsafe fn decode(
        &mut self,
        input: &[u16],
        input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        let available = input.len().saturating_sub(input_index);
        if available < 3 {
            return Err(DecodeFailure::incomplete(crate::nz(3)));
        }
        let high = input[input_index] as u32;
        let low = input[input_index + 1] as u32;
        Ok(((high << 16) | low, crate::nz(2)))
    }

    unsafe fn encode(
        &mut self,
        _value: &u32,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Ok(crate::nz(2))
    }
}

#[test]
fn test_buffered_decode_input_read_decoded_refills_to_maximum_window() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003, 0x0004]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 4);
    let mut codec = GrowingPairReadCodec::default();

    assert!(input.fill_until(3).expect("prefill should succeed"));
    let value = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect("decoder should refill to the codec maximum window");

    assert_eq!(0x0001_0002, value);
}

#[test]
fn test_buffered_decode_input_read_decoded_handles_incomplete_in_main_loop() {
    let input =
        ChunkedInput::new(vec![vec![0x0001, 0x0002], vec![0x0003, 0x0004]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 4);
    let mut codec = PartialWindowIncompleteCodec;

    let value = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect("incomplete input should refill inside the main decode loop");

    assert_eq!(0x0001_0002, value);
}

#[test]
fn test_buffered_decode_input_read_decoded_rejects_invalid_consumed_hint() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 2);
    let mut codec = OverconsumeInvalidReadCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err(
            "invalid consumed hints beyond the unread window should fail",
        );

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(
        error
            .to_string()
            .contains("decode error consumed units exceed unread window")
    );
}

#[test]
fn test_buffered_decode_input_read_decoded_scratch_grows_required_window() {
    let input = ChunkedInput::new(vec![vec![0x0001], vec![0x0002, 0x0003]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = ScratchGrowingReadCodec;

    let value = input.read_decoded_with(&mut codec, map_codec_error).expect(
        "scratch decode should grow the required window across refills",
    );

    assert_eq!(0x0001_0002, value);
}

#[test]
fn test_buffered_decode_input_read_decoded_scratch_maps_invalid_input() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = InvalidPairReadCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err("scratch decode should map invalid codec input");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert_eq!("bad input index", error.to_string());
}

#[derive(Debug, Default)]
struct AlwaysIncompleteReadCodec;

impl Codec for AlwaysIncompleteReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");
    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(4).expect("variable width");

    unsafe fn decode(
        &mut self,
        _input: &[u16],
        _input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        Err(DecodeFailure::incomplete(crate::nz(4)))
    }

    unsafe fn encode(
        &mut self,
        _value: &u32,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Ok(crate::nz(2))
    }
}

#[derive(Debug, Default)]
struct StuckIncompleteReadCodec;

impl Codec for StuckIncompleteReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");
    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");

    unsafe fn decode(
        &mut self,
        _input: &[u16],
        _input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        Err(DecodeFailure::incomplete(crate::nz(2)))
    }

    unsafe fn encode(
        &mut self,
        _value: &u32,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Ok(crate::nz(2))
    }
}

#[test]
fn test_buffered_decode_input_read_decoded_reports_eof_before_minimum_width() {
    let input = ChunkedInput::new(vec![vec![0x0001]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 2);
    let mut codec = FixedPairCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err("single unit input should fail before a pair is available");

    assert_eq!(ErrorKind::UnexpectedEof, error.kind());
}

#[test]
fn test_buffered_decode_input_read_decoded_reports_eof_after_incomplete() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 4);
    let mut codec = AlwaysIncompleteReadCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err("incomplete decode should fail at EOF");

    assert_eq!(ErrorKind::UnexpectedEof, error.kind());
}

#[test]
fn test_buffered_decode_input_read_decoded_refills_after_required_window_growth()
 {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002], vec![0x0003]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut codec = ScratchGrowingReadCodec;

    let value = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect("required window growth should refill and retry decoding");

    assert_eq!(0x0001_0002, value);
    assert_eq!(&[0x0003], input.unread());
}

#[test]
fn test_buffered_decode_input_read_decoded_scratch_reports_eof() {
    let input = ChunkedInput::new(Vec::<Vec<u16>>::new());
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = FixedPairCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err("scratch decode should fail at EOF");

    assert_eq!(ErrorKind::UnexpectedEof, error.kind());
}

#[test]
fn test_buffered_decode_input_read_decoded_scratch_rejects_impossible_incomplete()
 {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = StuckIncompleteReadCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err(
            "scratch decode should reject impossible incomplete windows",
        );

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("loaded scratch window"));
}

#[derive(Debug, Default)]
struct ImpossibleIncompleteMainLoopCodec;

impl Codec for ImpossibleIncompleteMainLoopCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");
    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");

    unsafe fn decode(
        &mut self,
        _input: &[u16],
        _input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        Err(DecodeFailure::incomplete(crate::nz(2)))
    }

    unsafe fn encode(
        &mut self,
        _value: &u32,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Ok(crate::nz(2))
    }
}

#[derive(Debug, Default)]
struct InvalidWithConsumedReadCodec;

impl Codec for InvalidWithConsumedReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");
    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");

    unsafe fn decode(
        &mut self,
        _input: &[u16],
        _input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        Err(DecodeFailure::invalid(
            PairDecodeError::BadInputIndex,
            core::num::NonZeroUsize::MIN,
        ))
    }

    unsafe fn encode(
        &mut self,
        _value: &u32,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Ok(crate::nz(2))
    }
}

#[derive(Debug, Default)]
struct FailingReadInput;

impl Input for FailingReadInput {
    type Item = u16;

    unsafe fn read_unchecked(
        &mut self,
        _output: &mut [u16],
        _index: usize,
        _count: usize,
    ) -> std::io::Result<usize> {
        Err(Error::new(ErrorKind::BrokenPipe, "input read failure"))
    }
}

#[derive(Debug)]
struct ErrorAfterTwoUnitInput {
    first_read: bool,
}

impl Default for ErrorAfterTwoUnitInput {
    fn default() -> Self {
        Self { first_read: true }
    }
}

impl Input for ErrorAfterTwoUnitInput {
    type Item = u16;

    unsafe fn read_unchecked(
        &mut self,
        output: &mut [u16],
        index: usize,
        count: usize,
    ) -> std::io::Result<usize> {
        if self.first_read {
            self.first_read = false;
            let read = count.min(2);
            output[index..index + read]
                .copy_from_slice(&[0x0001, 0x0002][..read]);
            Ok(read)
        } else {
            Err(Error::new(ErrorKind::BrokenPipe, "refill failure"))
        }
    }
}

#[derive(Debug, Default)]
struct IncompleteBeyondBufferReadCodec;

impl Codec for IncompleteBeyondBufferReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");
    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(4).expect("variable width");

    unsafe fn decode(
        &mut self,
        _input: &[u16],
        _input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        Err(DecodeFailure::incomplete(crate::nz(4)))
    }

    unsafe fn encode(
        &mut self,
        _value: &u32,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Ok(crate::nz(2))
    }
}

#[derive(Debug, Default)]
struct InvalidWithoutConsumedReadCodec;

impl Codec for InvalidWithoutConsumedReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");
    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::new(2).expect("pair width");

    unsafe fn decode(
        &mut self,
        _input: &[u16],
        _input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        Err(DecodeFailure::invalid_without_consumed(
            PairDecodeError::BadInputIndex,
        ))
    }

    unsafe fn encode(
        &mut self,
        _value: &u32,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Ok(crate::nz(2))
    }
}

#[test]
fn test_buffered_decode_input_read_decoded_rejects_impossible_incomplete_in_window()
 {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 4);
    let mut codec = ImpossibleIncompleteMainLoopCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err("incomplete inside the decode window should be rejected");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(
        error.to_string().contains(
            "codec reported incomplete input within available window"
        )
    );
}

#[test]
fn test_buffered_decode_input_read_decoded_consumes_invalid_consumed_hint() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 2);
    let mut codec = InvalidWithConsumedReadCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err(
            "invalid consumed hints should be mapped after consumption",
        );

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert_eq!("bad input index", error.to_string());
    assert_eq!(1, input.unread_len());
}

#[test]
fn test_buffered_decode_input_read_decoded_propagates_initial_refill_error() {
    let mut input = TranscodeDecodeInput::with_capacity(FailingReadInput, 2);
    let mut codec = FixedPairCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err("initial non-scratch refill errors should propagate");

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}

#[test]
fn test_buffered_decode_input_read_decoded_propagates_max_window_refill_error()
{
    let mut input = TranscodeDecodeInput::with_capacity(
        ErrorAfterTwoUnitInput::default(),
        4,
    );
    let mut codec = GrowingPairReadCodec::default();

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err(
            "refill errors while reserving the maximum window should propagate",
        );

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}

#[test]
fn test_buffered_decode_input_read_decoded_propagates_incomplete_refill_error()
{
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut codec = IncompleteBeyondBufferReadCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err(
            "refill errors after an incomplete decode should propagate",
        );

    assert_eq!(ErrorKind::InvalidInput, error.kind());
}

#[test]
fn test_buffered_decode_input_read_decoded_maps_invalid_without_consumed_hint()
{
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 2);
    let mut codec = InvalidWithoutConsumedReadCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err("invalid decode without a consumed hint should be mapped");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert_eq!("bad input index", error.to_string());
    assert_eq!(2, input.unread_len());
}

#[test]
fn test_buffered_decode_input_read_decoded_scratch_propagates_read_errors() {
    let mut input = TranscodeDecodeInput::with_capacity(FailingReadInput, 1);
    let mut codec = FixedPairCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err("scratch decode should propagate read failures");

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}

#[test]
fn test_buffered_decode_input_transcode_into_accepts_zero_count() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 2);
    let mut decoder = PairDecoder;
    let mut mapper = map_error;

    assert_eq!(
        0,
        input
            .transcode_into(&mut decoder, &mut mapper, &mut [0_u32; 1], 0, 0)
            .expect("zero count should succeed without reading"),
    );
}

#[test]
fn test_buffered_decode_input_debug_shows_wrapped_input() {
    let input =
        TranscodeDecodeInput::with_capacity(ChunkedInput::new(vec![]), 2);
    let debug = format!("{input:?}");

    assert!(debug.contains("TranscodeDecodeInput"));
    assert!(debug.contains("input"));
}
