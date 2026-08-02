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
    TranscodeDecodeError,
    TranscodeDecodeInput,
    TranscodeProgress,
    Transcoder,
};
use qubit_io::Input;

#[test]
fn try_with_capacity_allocates_decode_buffer() {
    let input =
        TranscodeDecodeInput::try_with_capacity(Cursor::new(vec![1_u8]), 1)
            .expect("decode buffer should allocate");

    assert!(input.capacity() >= 1);
}

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

fn domain(error: PairDecodeError) -> TranscodeDecodeError<PairDecodeError> {
    TranscodeDecodeError::domain_main(error, 0)
}

#[derive(Debug, Default)]
struct FixedPairCodec;

impl Codec for FixedPairCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

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
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = (value >> 16) as u16;
        output[output_index + 1] = *value as u16;
        Ok(2)
    }
}

#[derive(Debug, Default)]
struct ContextualIncompleteReadCodec;

impl Codec for ContextualIncompleteReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        _input: &[u16],
        _input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        Err(DecodeFailure::incomplete_with_source(
            PairDecodeError::BadInputIndex,
            crate::nz(2),
        ))
    }

    unsafe fn encode(
        &mut self,
        _value: &u32,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default)]
enum DecodeLifecycleMode {
    #[default]
    Normal,
    ResetError,
    FinishError,
    ResetOverreport,
    FinishOverreport,
}

#[derive(Debug, Default)]
struct DecodeLifecycleCodec {
    state: u8,
    mode: DecodeLifecycleMode,
}

impl Codec for DecodeLifecycleCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_RESET_VALUES: usize = 1;
    const MAX_DECODE_FINISH_VALUES: usize = 1;

    unsafe fn decode_reset(
        &mut self,
        output: &mut [u32],
        output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        if matches!(self.mode, DecodeLifecycleMode::ResetError) {
            return Err(PairDecodeError::BadOutputIndex);
        }
        assert_eq!(0, self.state, "decode reset must start each lifecycle");
        output[output_index] = 0xaaaa;
        self.state = 1;
        if matches!(self.mode, DecodeLifecycleMode::ResetOverreport) {
            return Ok(2);
        }
        Ok(1)
    }

    unsafe fn decode(
        &mut self,
        input: &[u16],
        input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        assert_eq!(1, self.state, "decode must run after reset");
        self.state = 2;
        Ok((u32::from(input[input_index]), crate::nz(1)))
    }

    unsafe fn decode_finish(
        &mut self,
        output: &mut [u32],
        output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        if matches!(self.mode, DecodeLifecycleMode::FinishError) {
            return Err(PairDecodeError::BadInputIndex);
        }
        assert_eq!(2, self.state, "decode finish must run after decode");
        output[output_index] = 0xbbbb;
        self.state = 0;
        if matches!(self.mode, DecodeLifecycleMode::FinishOverreport) {
            return Ok(2);
        }
        Ok(1)
    }

    unsafe fn encode(
        &mut self,
        value: &u32,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = *value as u16;
        Ok(1)
    }
}

#[derive(Debug, Eq, PartialEq)]
struct NonDefaultValue(u16);

#[derive(Debug, Default)]
struct NonDefaultValueCodec;

impl Codec for NonDefaultValueCodec {
    type Value = NonDefaultValue;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u16],
        input_index: usize,
    ) -> Result<
        (Self::Value, core::num::NonZeroUsize),
        DecodeFailure<Self::DecodeError>,
    > {
        Ok((NonDefaultValue(input[input_index]), crate::nz(1)))
    }

    unsafe fn encode(
        &mut self,
        value: &Self::Value,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = value.0;
        Ok(1)
    }
}

macro_rules! noop_reset {
    ($output:ty) => {
        fn reset(
            &mut self,
            output: &mut [$output],
            output_index: usize,
        ) -> Result<usize, TranscodeDecodeError<PairDecodeError>> {
            qubit_codec::TranscodeFailure::ensure_output_index(
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
        ) -> Result<usize, TranscodeDecodeError<PairDecodeError>> {
            qubit_codec::TranscodeFailure::ensure_output_index(
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

#[test]
#[should_panic(expected = "cannot consume beyond buffered input")]
fn test_transcode_decode_input_consume_panics_beyond_unread_window() {
    let mut input = TranscodeDecodeInput::with_capacity(
        ChunkedInput::new(vec![vec![1_u16]]),
        1,
    );

    assert!(input.fill_until(1).expect("fill should succeed"));
    input.consume(2);
}

impl Transcoder for PairDecoder {
    type Input = u16;
    type Output = u32;
    type Error = TranscodeDecodeError<PairDecodeError>;

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
    ) -> Result<TranscodeProgress, TranscodeDecodeError<PairDecodeError>> {
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
                    crate::nz(1),
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
            Ok(TranscodeProgress::need_input(crate::nz(2), read, written))
        }
    }

    fn finish(
        &mut self,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<usize, TranscodeDecodeError<PairDecodeError>> {
        Ok(0)
    }
}

#[derive(Debug, Default)]
struct NoProgressCompleteDecoder;

impl Transcoder for NoProgressCompleteDecoder {
    type Input = u16;
    type Output = u32;
    type Error = TranscodeDecodeError<PairDecodeError>;

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    noop_reset!(u32);

    fn transcode(
        &mut self,
        _input: &[u16],
        _input_index: usize,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeDecodeError<PairDecodeError>> {
        Ok(TranscodeProgress::complete(0, 0))
    }

    noop_finish!(u32);
}

#[test]
fn test_transcode_decode_input_rejects_complete_without_progress() {
    let mut input = TranscodeDecodeInput::with_capacity(
        ChunkedInput::new(vec![vec![1_u16]]),
        2,
    );
    let mut decoder = NoProgressCompleteDecoder;
    let mut output = [0_u32; 1];

    let error = input
        .transcode(&mut decoder, &mut transcode_error_to_io, &mut output, 0, 1)
        .expect_err("non-progressing Complete must be rejected");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert_eq!(
        "transcoder reported Complete after consuming 0 of 1 available input units",
        error.to_string(),
    );
}

#[derive(Debug, Default)]
struct FinishDecoder {
    finished: bool,
}

#[derive(Debug, Default)]
struct ResetDecoder {
    reset_calls: usize,
}

impl Transcoder for ResetDecoder {
    type Input = u16;
    type Output = u32;
    type Error = TranscodeDecodeError<PairDecodeError>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }

    fn reset(
        &mut self,
        output: &mut [u32],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        self.reset_calls += 1;
        output[output_index] = 0xaaaa;
        Ok(1)
    }

    fn transcode(
        &mut self,
        _input: &[u16],
        _input_index: usize,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Ok(TranscodeProgress::complete(0, 0))
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
struct OverreportingFinishDecoder;

impl Transcoder for OverreportingFinishDecoder {
    type Input = u16;
    type Output = u32;
    type Error = TranscodeDecodeError<PairDecodeError>;

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }

    noop_reset!(u32);

    fn transcode(
        &mut self,
        _input: &[u16],
        _input_index: usize,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeDecodeError<PairDecodeError>> {
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(
        &mut self,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<usize, TranscodeDecodeError<PairDecodeError>> {
        Ok(2)
    }
}

impl Transcoder for FinishDecoder {
    type Input = u16;
    type Output = u32;
    type Error = TranscodeDecodeError<PairDecodeError>;

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }

    noop_reset!(u32);

    fn transcode(
        &mut self,
        input: &[u16],
        input_index: usize,
        _output: &mut [u32],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeDecodeError<PairDecodeError>> {
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
    ) -> Result<usize, TranscodeDecodeError<PairDecodeError>> {
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

impl Transcoder for ZeroWidthFailingFinishDecoder {
    type Input = u16;
    type Output = u32;
    type Error = TranscodeDecodeError<PairDecodeError>;

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
    ) -> Result<TranscodeProgress, TranscodeDecodeError<PairDecodeError>> {
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
    ) -> Result<usize, TranscodeDecodeError<PairDecodeError>> {
        Err(domain(PairDecodeError::BadInputIndex))
    }
}

#[derive(Debug)]
struct ChunkedInput {
    chunks: VecDeque<Vec<u16>>,
    reads: usize,
    fail_after_reads: Option<usize>,
}

impl ChunkedInput {
    fn new(chunks: Vec<Vec<u16>>) -> Self {
        Self {
            chunks: VecDeque::from(chunks),
            reads: 0,
            fail_after_reads: None,
        }
    }

    /// Creates chunked input that fails before the configured read number.
    fn failing_after(chunks: Vec<Vec<u16>>, reads: usize) -> Self {
        Self {
            chunks: VecDeque::from(chunks),
            reads: 0,
            fail_after_reads: Some(reads),
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
        if self.fail_after_reads == Some(self.reads) {
            return Err(Error::new(ErrorKind::BrokenPipe, "input failure"));
        }
        self.reads += 1;
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

impl Transcoder for TwoUnitFinishDecoder {
    type Input = u16;
    type Output = u32;
    type Error = TranscodeDecodeError<PairDecodeError>;

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
    ) -> Result<TranscodeProgress, TranscodeDecodeError<PairDecodeError>> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(
        &mut self,
        output: &mut [u32],
        output_index: usize,
    ) -> Result<usize, TranscodeDecodeError<PairDecodeError>> {
        qubit_codec::TranscodeFailure::ensure_output_capacity(
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

impl Transcoder for CapacityBoundDecoder {
    type Input = u16;
    type Output = u32;
    type Error = TranscodeDecodeError<PairDecodeError>;

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
    ) -> Result<TranscodeProgress, TranscodeDecodeError<PairDecodeError>> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    noop_finish!(u32);
}

#[derive(Debug, Default)]
struct FailingTranscodeDecoder;

impl Transcoder for FailingTranscodeDecoder {
    type Input = u16;
    type Output = u32;
    type Error = TranscodeDecodeError<PairDecodeError>;

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
    ) -> Result<TranscodeProgress, TranscodeDecodeError<PairDecodeError>> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        Err(domain(PairDecodeError::BadInputIndex))
    }

    noop_finish!(u32);
}

#[derive(Debug, Default)]
struct OverreadingProgressDecoder;

impl Transcoder for OverreadingProgressDecoder {
    type Input = u16;
    type Output = u32;
    type Error = TranscodeDecodeError<PairDecodeError>;

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
    ) -> Result<TranscodeProgress, TranscodeDecodeError<PairDecodeError>> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::complete(input.len() + 1, 0))
    }

    noop_finish!(u32);
}

#[derive(Debug, Default)]
struct OverwritingProgressDecoder;

impl Transcoder for OverwritingProgressDecoder {
    type Input = u16;
    type Output = u32;
    type Error = TranscodeDecodeError<PairDecodeError>;

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
    ) -> Result<TranscodeProgress, TranscodeDecodeError<PairDecodeError>> {
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
impl Transcoder for OverflowingNeedInputDecoder {
    type Input = u16;
    type Output = u32;
    type Error = TranscodeDecodeError<PairDecodeError>;

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
    ) -> Result<TranscodeProgress, TranscodeDecodeError<PairDecodeError>> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::need_input(crate::nz(1), 0, 0))
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

impl Transcoder for FailingFinishDecoder {
    type Input = u16;
    type Output = u32;
    type Error = TranscodeDecodeError<PairDecodeError>;

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
    ) -> Result<TranscodeProgress, TranscodeDecodeError<PairDecodeError>> {
        if input_index > input.len() {
            return Err(domain(PairDecodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(
        &mut self,
        _output: &mut [u32],
        _output_index: usize,
    ) -> Result<usize, TranscodeDecodeError<PairDecodeError>> {
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
struct FailingSeekInput;

impl Input for FailingSeekInput {
    type Item = u8;

    unsafe fn read_unchecked(
        &mut self,
        _output: &mut [u8],
        _index: usize,
        _count: usize,
    ) -> std::io::Result<usize> {
        Ok(0)
    }
}

impl qubit_io::Seekable for FailingSeekInput {
    type Unit = u8;

    fn seek_to(&mut self, _position: SeekFrom) -> std::io::Result<u64> {
        Err(Error::new(ErrorKind::BrokenPipe, "seek failure"))
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

fn map_error(error: TranscodeDecodeError<PairDecodeError>) -> Error {
    Error::new(ErrorKind::InvalidData, format!("{error:?}"))
}

fn transcode_error_to_io(
    error: TranscodeDecodeError<PairDecodeError>,
) -> Error {
    map_error(error)
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
    D: Transcoder<
            Input = u16,
            Output = u32,
            Error = TranscodeDecodeError<PairDecodeError>,
        >,
{
    let mut mapper: fn(TranscodeDecodeError<PairDecodeError>) -> Error =
        map_error;
    input.transcode(decoder, &mut mapper, output, output_index, count)
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
    D: Transcoder<
            Input = u16,
            Output = u32,
            Error = TranscodeDecodeError<PairDecodeError>,
        >,
{
    let mut mapper: fn(TranscodeDecodeError<PairDecodeError>) -> Error =
        map_error;
    input.finish(decoder, &mut mapper, output, output_index, count)
}

#[test]
fn test_buffered_decode_input_reset_writes_prefix_without_consuming_input() {
    let input = ChunkedInput::new(vec![vec![0x1234]]);
    let input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut decoder = ResetDecoder::default();
    let mut mapper: fn(TranscodeDecodeError<PairDecodeError>) -> Error =
        map_error;
    let mut output = [0_u32; 1];

    let written = input
        .reset(&mut decoder, &mut mapper, &mut output, 0, 1)
        .expect("reset should write its prefix");

    assert_eq!(1, written);
    assert_eq!(1, decoder.reset_calls);
    assert_eq!([0xaaaa], output);
    assert_eq!(0, input.inner().reads);
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
fn test_buffered_decode_input_strict_decode_accepts_non_default_values() {
    let input = ChunkedInput::new(vec![vec![0x1234]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = NonDefaultValueCodec;

    let value = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect("strict decoding should not require default values");

    assert_eq!(NonDefaultValue(0x1234), value);
}

#[test]
fn test_buffered_decode_input_owned_decode_runs_complete_lifecycle() {
    let input = ChunkedInput::new(vec![vec![0x1234, 0x5678]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 2);
    let mut codec = DecodeLifecycleCodec::default();

    let first = input
        .read_decoded_lifecycle_with(&mut codec, map_codec_error)
        .expect("first value should complete its decode lifecycle");
    let second = input
        .read_decoded_lifecycle_with(&mut codec, map_codec_error)
        .expect("second value should start a fresh decode lifecycle");

    assert_eq!(&0x1234, first.value());
    assert_eq!(&0x5678, second.value());
}

#[test]
fn test_buffered_decode_input_rejects_lifecycle_output_before_reading() {
    let input = ChunkedInput::new(vec![vec![0x1234]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = DecodeLifecycleCodec::default();

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err("strict decoding must reject lifecycle output");

    assert_eq!(ErrorKind::Unsupported, error.kind());
    assert_eq!(0, input.inner().reads);
    assert_eq!(0, codec.state);

    let output = input
        .read_decoded_lifecycle_with(&mut codec, map_codec_error)
        .expect("lifecycle decoding should remain retryable");
    assert_eq!(&0x1234, output.value());
}

#[test]
fn test_buffered_decode_input_preserves_owned_decode_lifecycle_output() {
    let input = ChunkedInput::new(vec![vec![0x1234]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = DecodeLifecycleCodec::default();

    let output = input
        .read_decoded_lifecycle_with(&mut codec, map_codec_error)
        .expect("complete lifecycle output should be preserved");

    assert_eq!(&[0xaaaa], output.reset());
    assert_eq!(&0x1234, output.value());
    assert_eq!(&[0xbbbb], output.finish());
    assert_eq!((vec![0xaaaa], 0x1234, vec![0xbbbb]), output.into_parts(),);
}

#[test]
fn test_buffered_decode_input_owned_lifecycle_uses_input_scratch() {
    let input = ChunkedInput::new(vec![vec![0x1234]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 0);
    let mut codec = DecodeLifecycleCodec::default();

    let output = input
        .read_decoded_lifecycle_with(&mut codec, map_codec_error)
        .expect("scratch decode should complete its lifecycle");

    assert_eq!(&0x1234, output.value());
}

#[test]
fn test_buffered_decode_input_writes_decode_lifecycle_output_to_separate_scratch()
 {
    let input = ChunkedInput::new(vec![vec![0x1234, 0x5678]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 2);
    let mut codec = DecodeLifecycleCodec::default();
    let mut reset_output = [0_u32; 1];
    let mut finish_output = [0_u32; 1];

    let first = input
        .read_decoded_lifecycle_with_scratch(
            &mut codec,
            &mut reset_output,
            &mut finish_output,
            map_codec_error,
        )
        .expect("first value should use caller-provided lifecycle storage");
    let second = input
        .read_decoded_lifecycle_with_scratch(
            &mut codec,
            &mut reset_output,
            &mut finish_output,
            map_codec_error,
        )
        .expect("second value should reuse caller-provided lifecycle storage");

    assert_eq!((0x1234, 1, 1), first.into_parts());
    assert_eq!((0x5678, 1, 1), second.into_parts());
    assert_eq!([0xaaaa], reset_output);
    assert_eq!([0xbbbb], finish_output);
}

#[test]
fn test_buffered_decode_input_lifecycle_rejects_short_reset_storage() {
    let input = ChunkedInput::new(vec![vec![0x1234]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = DecodeLifecycleCodec::default();
    let mut reset_output = [];
    let mut finish_output = [0_u32; 1];

    let error = input
        .read_decoded_lifecycle_with_scratch(
            &mut codec,
            &mut reset_output,
            &mut finish_output,
            map_codec_error,
        )
        .expect_err("short reset output must be rejected");

    assert_eq!(ErrorKind::InvalidInput, error.kind());
    assert_eq!(
        "decode reset output is shorter than the codec reset bound",
        error.to_string(),
    );
    assert_eq!(0, input.inner().reads);
    assert_eq!(0, codec.state);
}

#[test]
fn test_buffered_decode_input_lifecycle_rejects_short_finish_storage() {
    let input = ChunkedInput::new(vec![vec![0x1234]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = DecodeLifecycleCodec::default();
    let mut reset_output = [0_u32; 1];
    let mut finish_output = [];

    let error = input
        .read_decoded_lifecycle_with_scratch(
            &mut codec,
            &mut reset_output,
            &mut finish_output,
            map_codec_error,
        )
        .expect_err("short finish output must be rejected");

    assert_eq!(ErrorKind::InvalidInput, error.kind());
    assert_eq!(
        "decode finish output is shorter than the codec finish bound",
        error.to_string(),
    );
    assert_eq!(0, input.inner().reads);
    assert_eq!(0, codec.state);
}

#[test]
fn test_buffered_decode_input_lifecycle_scratch_accepts_non_default_values() {
    let input = ChunkedInput::new(vec![vec![0x1234]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = NonDefaultValueCodec;
    let mut reset_output = [];
    let mut finish_output = [];

    let progress = input
        .read_decoded_lifecycle_with_scratch(
            &mut codec,
            &mut reset_output,
            &mut finish_output,
            map_codec_error,
        )
        .expect("stateless codecs should not require default values");

    assert_eq!(&NonDefaultValue(0x1234), progress.value());
}

#[test]
fn test_buffered_decode_input_lifecycle_maps_lifecycle_errors() {
    for (mode, expected) in [
        (DecodeLifecycleMode::ResetError, "bad output index"),
        (DecodeLifecycleMode::FinishError, "bad input index"),
    ] {
        let input = ChunkedInput::new(vec![vec![0x1234]]);
        let mut input = TranscodeDecodeInput::with_capacity(input, 1);
        let mut codec = DecodeLifecycleCodec { state: 0, mode };

        let error = input
            .read_decoded_lifecycle_with(&mut codec, map_codec_error)
            .expect_err("configured lifecycle stage should fail");

        assert_eq!(ErrorKind::InvalidData, error.kind());
        assert_eq!(expected, error.to_string());
    }
}

#[test]
#[should_panic(expected = "Codec::decode_reset wrote beyond its reset bound")]
fn test_buffered_decode_input_read_decoded_panics_on_reset_overreport() {
    let input = ChunkedInput::new(vec![vec![0x1234]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = DecodeLifecycleCodec {
        state: 0,
        mode: DecodeLifecycleMode::ResetOverreport,
    };

    let _ = input.read_decoded_lifecycle_with(&mut codec, map_codec_error);
}

#[test]
#[should_panic(expected = "Codec::decode_finish wrote beyond its finish bound")]
fn test_buffered_decode_input_read_decoded_panics_on_finish_overreport() {
    let input = ChunkedInput::new(vec![vec![0x1234]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = DecodeLifecycleCodec {
        state: 0,
        mode: DecodeLifecycleMode::FinishOverreport,
    };

    let _ = input.read_decoded_lifecycle_with(&mut codec, map_codec_error);
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
fn test_buffered_decode_input_transcode_respects_output_range() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut mapper: fn(TranscodeDecodeError<PairDecodeError>) -> Error =
        map_error;
    let mut output = [0_u32; 1];

    let read = input
        .transcode(&mut decoder, &mut mapper, &mut output, 0, 1)
        .expect("checked decode should accept a valid output range");

    assert_eq!(1, read);
    assert_eq!([0x0001_0002], output);
}

#[test]
fn test_buffered_decode_input_transcode_rejects_invalid_output_range() {
    let input = ChunkedInput::new(Vec::new());
    let mut decoder = PairDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut mapper: fn(TranscodeDecodeError<PairDecodeError>) -> Error =
        map_error;
    let mut output = [0_u32; 1];

    let error = input
        .transcode(&mut decoder, &mut mapper, &mut output, 1, 1)
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
    let input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut mapper: fn(TranscodeDecodeError<PairDecodeError>) -> Error =
        map_error;
    let mut output = [0_u32; 1];

    let error = input
        .finish(&mut decoder, &mut mapper, &mut output, 1, 1)
        .expect_err("invalid finish output range should be rejected");

    assert_eq!(ErrorKind::InvalidInput, error.kind());
    assert_eq!(
        "finish output range exceeds destination buffer",
        error.to_string(),
    );
}

#[test]
#[should_panic(expected = "finish wrote beyond its bound")]
fn test_buffered_decode_input_finish_panics_when_decoder_overreports_bound() {
    let input = ChunkedInput::new(Vec::new());
    let mut decoder = OverreportingFinishDecoder;
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut output = [0_u32; 1];

    let _ = finish_with(&mut input, &mut decoder, &mut output, 0, 1);
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
    assert_eq!(Ok(1), decoder.max_finish_output_len());
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
    assert_eq!(Ok(1), decoder.max_finish_output_len());
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
    let mut mapper: fn(TranscodeDecodeError<PairDecodeError>) -> Error =
        map_error;
    let mut output = [0_u32; 2];
    let first = input
        .transcode(&mut first_decoder, &mut mapper, &mut output, 0, 1)
        .expect("first decoder should read one value");
    let second = input
        .transcode(&mut second_decoder, &mut mapper, &mut output, 1, 1)
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

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

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
    ) -> Result<usize, Self::EncodeError> {
        Ok(2)
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

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 4;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 4;

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
    ) -> Result<usize, Self::EncodeError> {
        Ok(2)
    }
}

#[derive(Debug, Default)]
struct OverconsumeReadCodec;

impl Codec for OverconsumeReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

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
    ) -> Result<usize, Self::EncodeError> {
        Ok(2)
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
fn test_buffered_decode_input_maps_incomplete_source_at_eof() {
    let input = ChunkedInput::new(vec![vec![0x0001]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = ContextualIncompleteReadCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err("EOF should preserve the codec incomplete source");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert_eq!("bad input index", error.to_string());
    assert!(input.unread().is_empty());
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
fn test_buffered_decode_input_read_decoded_grows_when_value_exceeds_capacity() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003, 0x0004]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = FixedPairCodec;

    let value = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect("decode should grow its persistent buffer across refills");

    assert_eq!(0x0001_0002, value);
    assert!(input.capacity() >= 2);
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

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 4;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 4;

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
    ) -> Result<usize, Self::EncodeError> {
        Ok(2)
    }
}

#[derive(Debug, Default)]
struct OverlongIncompleteReadCodec;

impl Codec for OverlongIncompleteReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        _input: &[u16],
        _input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        Err(DecodeFailure::incomplete(crate::nz(3)))
    }

    unsafe fn encode(
        &mut self,
        _value: &u32,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Ok(2)
    }
}

#[derive(Debug, Default)]
struct OverconsumeInvalidReadCodec;

impl Codec for OverconsumeInvalidReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

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
    ) -> Result<usize, Self::EncodeError> {
        Ok(2)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum ScratchReadMode {
    #[default]
    GrowThenSucceed,
    Succeed,
    Overconsume,
    StuckIncomplete,
    Invalid,
    InvalidOverconsume,
    InvalidUnknown,
}

#[derive(Debug, Default)]
struct ScratchGrowingReadCodec {
    mode: ScratchReadMode,
}

impl ScratchGrowingReadCodec {
    /// Creates the fixture in the selected decode-outcome mode.
    fn with_mode(mode: ScratchReadMode) -> Self {
        Self { mode }
    }
}

impl Codec for ScratchGrowingReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 4;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 4;

    unsafe fn decode(
        &mut self,
        input: &[u16],
        input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        let available = input.len().saturating_sub(input_index);
        match self.mode {
            ScratchReadMode::GrowThenSucceed if available < 3 => {
                return Err(DecodeFailure::incomplete(crate::nz(3)));
            }
            ScratchReadMode::StuckIncomplete => {
                return Err(DecodeFailure::incomplete(crate::nz(2)));
            }
            ScratchReadMode::Overconsume => {
                return Ok((0, crate::nz(5)));
            }
            ScratchReadMode::Invalid => {
                return Err(DecodeFailure::invalid(
                    PairDecodeError::BadInputIndex,
                    crate::nz(1),
                ));
            }
            ScratchReadMode::InvalidOverconsume => {
                return Err(DecodeFailure::invalid(
                    PairDecodeError::BadInputIndex,
                    crate::nz(5),
                ));
            }
            ScratchReadMode::InvalidUnknown => {
                return Err(DecodeFailure::invalid_unknown(
                    PairDecodeError::BadInputIndex,
                ));
            }
            ScratchReadMode::GrowThenSucceed | ScratchReadMode::Succeed => {}
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
    ) -> Result<usize, Self::EncodeError> {
        Ok(2)
    }
}

/// Runs one configured single-value decode and returns its final input state.
///
/// The returned result preserves I/O or mapped codec errors, while the adapter
/// lets each scenario verify exactly which source units remain unread.
fn read_with_scratch_mode(
    input: ChunkedInput,
    capacity: usize,
    mode: ScratchReadMode,
) -> (std::io::Result<u32>, TranscodeDecodeInput<ChunkedInput>) {
    let mut input = TranscodeDecodeInput::with_capacity(input, capacity);
    let mut codec = ScratchGrowingReadCodec::with_mode(mode);
    let result = input.read_decoded_with(&mut codec, map_codec_error);
    (result, input)
}

#[derive(Debug, Default)]
struct ScratchByteCodec;

impl Codec for ScratchByteCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 3;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 3;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        let available = input.len().saturating_sub(input_index);
        if available < 3 {
            return Err(DecodeFailure::incomplete(crate::nz(3)));
        }
        Ok((input[input_index], crate::nz(2)))
    }

    unsafe fn encode(
        &mut self,
        _value: &u8,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Ok(1)
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
fn test_buffered_decode_input_switches_to_scratch_when_incomplete_exceeds_capacity()
 {
    let input =
        ChunkedInput::new(vec![vec![0x0001, 0x0002], vec![0x0003, 0x0004]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 2);
    let mut codec = PartialWindowIncompleteCodec;

    let value = input.read_decoded_with(&mut codec, map_codec_error).expect(
        "decode should switch to scratch when the hint exceeds capacity",
    );

    assert_eq!(0x0001_0002, value);
    assert_eq!(&[0x0003, 0x0004], input.unread());
}

#[test]
#[should_panic(
    expected = "Codec::decode incomplete required_total exceeded Codec::MAX_DECODE_UNITS_PER_VALUE"
)]
fn test_buffered_decode_input_panics_when_scratch_hint_exceeds_codec_maximum() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002, 0x0003]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = OverlongIncompleteReadCodec;

    let _ = input.read_decoded_with(&mut codec, map_codec_error);
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
    let mut codec = ScratchGrowingReadCodec::default();

    let value = input.read_decoded_with(&mut codec, map_codec_error).expect(
        "scratch decode should grow the required window across refills",
    );

    assert_eq!(0x0001_0002, value);
    assert_eq!(&[0x0003], input.unread());
}

#[test]
fn test_buffered_decode_input_configurable_codec_validates_buffered_decode_contract()
 {
    let (result, input) = read_with_scratch_mode(
        ChunkedInput::new(vec![vec![0x0001, 0x0002]]),
        2,
        ScratchReadMode::Succeed,
    );
    assert_eq!(0x0001_0002, result.expect("a complete pair should decode"));
    assert!(input.unread().is_empty());

    let (result, input) = read_with_scratch_mode(
        ChunkedInput::new(Vec::new()),
        2,
        ScratchReadMode::Succeed,
    );
    assert_eq!(
        ErrorKind::UnexpectedEof,
        result.expect_err("empty input should report EOF").kind(),
    );
    assert!(input.unread().is_empty());

    let (result, input) = read_with_scratch_mode(
        ChunkedInput::failing_after(Vec::new(), 0),
        2,
        ScratchReadMode::Succeed,
    );
    assert_eq!(
        ErrorKind::BrokenPipe,
        result
            .expect_err("initial read errors should propagate")
            .kind(),
    );
    assert!(input.unread().is_empty());

    let (result, input) = read_with_scratch_mode(
        ChunkedInput::new(vec![vec![0x0001, 0x0002]]),
        2,
        ScratchReadMode::StuckIncomplete,
    );
    let error =
        result.expect_err("a satisfied incomplete hint should be rejected");
    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("available window"));
    assert_eq!(&[0x0001, 0x0002], input.unread());

    let (result, input) = read_with_scratch_mode(
        ChunkedInput::new(vec![vec![0x0001, 0x0002]]),
        2,
        ScratchReadMode::Overconsume,
    );
    let error =
        result.expect_err("successful decode cannot over-consume input");
    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("unread window"));
    assert_eq!(&[0x0001, 0x0002], input.unread());

    let (result, input) = read_with_scratch_mode(
        ChunkedInput::new(vec![vec![0x0001, 0x0002]]),
        2,
        ScratchReadMode::Invalid,
    );
    assert_eq!(
        "bad input index",
        result
            .expect_err("codec errors should be mapped")
            .to_string(),
    );
    assert_eq!(&[0x0002], input.unread());

    let (result, input) = read_with_scratch_mode(
        ChunkedInput::new(vec![vec![0x0001, 0x0002]]),
        2,
        ScratchReadMode::InvalidOverconsume,
    );
    let error =
        result.expect_err("invalid-input hints cannot over-consume input");
    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("unread window"));
    assert_eq!(&[0x0001, 0x0002], input.unread());

    let (result, input) = read_with_scratch_mode(
        ChunkedInput::new(vec![vec![0x0001, 0x0002]]),
        2,
        ScratchReadMode::InvalidUnknown,
    );
    assert_eq!(
        "bad input index",
        result
            .expect_err("unknown consumption should be mapped")
            .to_string(),
    );
    assert_eq!(&[0x0001, 0x0002], input.unread());
}

#[test]
fn test_buffered_decode_input_configurable_codec_validates_scratch_decode_contract()
 {
    let (result, input) = read_with_scratch_mode(
        ChunkedInput::new(Vec::new()),
        1,
        ScratchReadMode::Succeed,
    );
    assert_eq!(
        ErrorKind::UnexpectedEof,
        result
            .expect_err("empty scratch input should report EOF")
            .kind(),
    );
    assert!(input.unread().is_empty());

    let (result, input) = read_with_scratch_mode(
        ChunkedInput::failing_after(Vec::new(), 0),
        1,
        ScratchReadMode::Succeed,
    );
    assert_eq!(
        ErrorKind::BrokenPipe,
        result
            .expect_err("scratch read errors should propagate")
            .kind(),
    );
    assert!(input.unread().is_empty());

    let (result, input) = read_with_scratch_mode(
        ChunkedInput::new(vec![vec![0x0001, 0x0002]]),
        1,
        ScratchReadMode::StuckIncomplete,
    );
    let error =
        result.expect_err("a satisfied scratch hint should be rejected");
    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("available window"));
    assert_eq!(&[0x0001, 0x0002], input.unread());

    let (result, input) = read_with_scratch_mode(
        ChunkedInput::new(vec![vec![0x0001, 0x0002]]),
        1,
        ScratchReadMode::Overconsume,
    );
    let error = result
        .expect_err("successful scratch decode cannot over-consume input");
    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("unread window"));
    assert_eq!(&[0x0001, 0x0002], input.unread());

    let (result, input) = read_with_scratch_mode(
        ChunkedInput::new(vec![vec![0x0001, 0x0002]]),
        1,
        ScratchReadMode::Invalid,
    );
    assert_eq!(
        "bad input index",
        result
            .expect_err("scratch codec errors should be mapped")
            .to_string(),
    );
    assert_eq!(&[0x0002], input.unread());

    let (result, input) = read_with_scratch_mode(
        ChunkedInput::new(vec![vec![0x0001, 0x0002]]),
        1,
        ScratchReadMode::InvalidOverconsume,
    );
    let error =
        result.expect_err("scratch invalid hints cannot over-consume input");
    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("unread window"));
    assert_eq!(&[0x0001, 0x0002], input.unread());

    let (result, input) = read_with_scratch_mode(
        ChunkedInput::new(vec![vec![0x0001, 0x0002]]),
        1,
        ScratchReadMode::InvalidUnknown,
    );
    assert_eq!(
        "bad input index",
        result
            .expect_err("unknown scratch consumption should be mapped")
            .to_string(),
    );
    assert_eq!(&[0x0001, 0x0002], input.unread());
}

#[test]
fn test_buffered_decode_input_scratch_unread_supports_buffer_apis() {
    let input = ChunkedInput::new(vec![
        vec![0x0001],
        vec![0x0002, 0x0003],
        vec![0x0004],
    ]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = ScratchGrowingReadCodec::default();

    let value = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect("scratch decode should leave a scratch tail");
    assert_eq!(0x0001_0002, value);
    assert_eq!(1, input.unread_len());
    assert_eq!(&[0x0003], input.unread());

    assert!(
        input
            .fill_until(2)
            .expect("scratch refill should append units"),
        "scratch refill should reach the requested length",
    );
    assert_eq!(&[0x0003, 0x0004], input.unread());

    let mut copied = [0_u16; 2];
    // SAFETY: The destination range is valid and separate from the input
    // buffer.
    unsafe {
        input.copy_unread_to(&mut copied, 0, 2);
    }
    assert_eq!([0x0003, 0x0004], copied);

    input.consume(1);
    assert_eq!(&[0x0004], input.unread());

    let mut empty = [0_u16; 0];
    // SAFETY: The empty destination range is valid.
    let read = unsafe { input.read_unchecked(&mut empty, 0, 0) }
        .expect("zero-count scratch read should succeed");
    assert_eq!(0, read);

    let mut one = [0_u16; 1];
    // SAFETY: The destination range is valid.
    let read = unsafe { input.read_unchecked(&mut one, 0, 1) }
        .expect("scratch-only read should succeed");
    assert_eq!(1, read);
    assert_eq!([0x0004], one);
}

#[test]
fn test_buffered_decode_input_scratch_fill_until_reports_eof() {
    let input = ChunkedInput::new(vec![vec![0x0001], vec![0x0002, 0x0003]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = ScratchGrowingReadCodec::default();

    input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect("scratch decode should leave one unread unit");

    assert!(
        !input
            .fill_until(2)
            .expect("scratch refill should report clean EOF"),
        "scratch refill should return false when the wrapped input ends",
    );
    assert_eq!(&[0x0003], input.unread());
}

#[test]
fn test_buffered_decode_input_scratch_fill_until_propagates_read_errors() {
    let input = ChunkedInput::failing_after(
        vec![vec![0x0001, 0x0002], vec![0x0003]],
        2,
    );
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = ScratchGrowingReadCodec::default();

    input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect("scratch decode should leave one unread unit");
    let error = input
        .fill_until(2)
        .expect_err("scratch refill errors should propagate");

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
    assert_eq!(&[0x0003], input.unread());
}

#[test]
fn test_buffered_decode_input_scratch_read_unchecked_continues_into_input() {
    let input = ChunkedInput::new(vec![
        vec![0x0001],
        vec![0x0002, 0x0003],
        vec![0x0004, 0x0005],
    ]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = ScratchGrowingReadCodec::default();

    input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect("scratch decode should leave one unread unit");
    assert!(input.fill_until(2).expect("scratch refill should succeed"));

    let mut output = [0_u16; 3];
    // SAFETY: The destination range is valid.
    let read = unsafe { input.read_unchecked(&mut output, 0, 3) }
        .expect("scratch read should continue into the wrapped input");

    assert_eq!(3, read);
    assert_eq!([0x0003, 0x0004, 0x0005], output);
}

#[test]
fn test_buffered_decode_input_into_parts_preserves_scratch_unread() {
    let input = ChunkedInput::new(vec![
        vec![0x0001],
        vec![0x0002, 0x0003],
        vec![0x0004],
    ]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = ScratchGrowingReadCodec::default();

    input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect("scratch decode should leave one unread unit");
    assert!(input.fill_until(2).expect("scratch refill should succeed"));

    let (_inner, unread) = input.into_parts();

    assert_eq!(&[0x0003, 0x0004], unread.readable());
}

#[test]
fn test_buffered_decode_input_seek_adjusts_for_scratch_unread() {
    let mut input =
        TranscodeDecodeInput::with_capacity(Cursor::new(vec![1, 2, 3, 4]), 1);
    let mut codec = ScratchByteCodec;

    let value = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect("scratch byte decode should leave one unread byte");
    assert_eq!(1, value);
    assert_eq!(&[3], input.unread());

    let position = Seek::seek(&mut input, SeekFrom::Current(0))
        .expect("relative seek should account for scratch unread bytes");
    assert_eq!(2, position);

    let mut next = [0_u8; 1];
    let read = Read::read(&mut input, &mut next)
        .expect("seek should clear scratch and reposition the inner input");
    assert_eq!(1, read);
    assert_eq!([3], next);
}

#[test]
fn test_buffered_decode_input_seek_rejects_underflowing_scratch_adjustment() {
    let mut input =
        TranscodeDecodeInput::with_capacity(Cursor::new(vec![1, 2, 3]), 1);
    let mut codec = ScratchByteCodec;

    input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect("scratch byte decode should leave one unread byte");
    let error = Seek::seek(&mut input, SeekFrom::Current(i64::MIN))
        .expect_err("relative seek adjustment should reject underflow");

    assert_eq!(ErrorKind::InvalidInput, error.kind());
}

#[test]
fn test_buffered_decode_input_seek_propagates_wrapped_seek_errors() {
    let mut input = TranscodeDecodeInput::with_capacity(FailingSeekInput, 1);

    let error = Seek::seek(&mut input, SeekFrom::Start(0))
        .expect_err("wrapped seek errors should be propagated");

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
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
    assert_eq!(&[0x0002], input.unread());
}

#[test]
fn test_buffered_decode_input_read_decoded_scratch_maps_invalid_unknown() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = InvalidWithoutConsumedReadCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err("scratch decode should map invalid unknown input");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert_eq!("bad input index", error.to_string());
    assert_eq!(&[0x0001, 0x0002], input.unread());
}

#[derive(Debug, Default)]
struct AlwaysIncompleteReadCodec;

impl Codec for AlwaysIncompleteReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 4;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 4;

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
    ) -> Result<usize, Self::EncodeError> {
        Ok(2)
    }
}

#[derive(Debug, Default)]
struct StuckIncompleteReadCodec;

impl Codec for StuckIncompleteReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

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
    ) -> Result<usize, Self::EncodeError> {
        Ok(2)
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
    let mut codec = ScratchGrowingReadCodec::default();

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
    assert!(error.to_string().contains("available window"));
}

#[test]
fn test_buffered_decode_input_read_decoded_scratch_rejects_overconsuming_codec()
{
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = OverconsumeReadCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err("scratch decode should reject over-consumption");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(
        error
            .to_string()
            .contains("codec consumed units exceed unread window")
    );
}

#[test]
fn test_buffered_decode_input_read_decoded_scratch_rejects_invalid_consumed_hint()
 {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 1);
    let mut codec = OverconsumeInvalidReadCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err("scratch decode should reject invalid consumed hints");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(
        error
            .to_string()
            .contains("decode error consumed units exceed unread window")
    );
}

#[derive(Debug, Default)]
struct ImpossibleIncompleteMainLoopCodec;

impl Codec for ImpossibleIncompleteMainLoopCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

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
    ) -> Result<usize, Self::EncodeError> {
        Ok(2)
    }
}

#[derive(Debug, Default)]
struct InvalidWithConsumedReadCodec;

impl Codec for InvalidWithConsumedReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

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
    ) -> Result<usize, Self::EncodeError> {
        Ok(2)
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

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 4;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 4;

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
    ) -> Result<usize, Self::EncodeError> {
        Ok(2)
    }
}

#[derive(Debug, Default)]
struct InvalidWithoutConsumedReadCodec;

impl Codec for InvalidWithoutConsumedReadCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairDecodeError;
    type EncodeError = PairDecodeError;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        _input: &[u16],
        _input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        Err(DecodeFailure::invalid_unknown(
            PairDecodeError::BadInputIndex,
        ))
    }

    unsafe fn encode(
        &mut self,
        _value: &u32,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Ok(2)
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
    let input = ChunkedInput::failing_after(vec![vec![0x0001, 0x0002]], 1);
    let mut input = TranscodeDecodeInput::with_capacity(input, 3);
    let mut codec = IncompleteBeyondBufferReadCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err(
            "refill errors after an incomplete decode should propagate",
        );

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}

#[test]
fn test_buffered_decode_input_read_decoded_maps_invalid_unknown() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 2);
    let mut codec = InvalidWithoutConsumedReadCodec;

    let error = input
        .read_decoded_with(&mut codec, map_codec_error)
        .expect_err("invalid unknown decode should be mapped");

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
fn test_buffered_decode_input_transcode_accepts_zero_count() {
    let input = ChunkedInput::new(vec![vec![0x0001, 0x0002]]);
    let mut input = TranscodeDecodeInput::with_capacity(input, 2);
    let mut decoder = PairDecoder;
    let mut mapper = map_error;

    assert_eq!(
        0,
        input
            .transcode(&mut decoder, &mut mapper, &mut [0_u32; 1], 0, 0)
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
