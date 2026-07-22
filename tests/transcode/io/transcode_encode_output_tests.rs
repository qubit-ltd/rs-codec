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
    Codec,
    DecodeFailure,
    TranscodeEncodeError,
    TranscodeEncodeOutput,
    TranscodeProgress,
    Transcoder,
};
use qubit_io::Output;

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

fn domain<Value>(
    error: PairEncodeError,
) -> TranscodeEncodeError<PairEncodeError, Value> {
    TranscodeEncodeError::domain_main(error, 0)
}

#[derive(Debug, Default)]
struct CompleteEncodeLifecycleCodec {
    state: u8,
}

impl Codec for CompleteEncodeLifecycleCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairEncodeError;
    type EncodeError = PairEncodeError;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_RESET_UNITS: usize = 1;
    const MAX_ENCODE_FINISH_UNITS: usize = 1;

    unsafe fn encode_reset(
        &mut self,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        assert_eq!(0, self.state, "encode reset must start each lifecycle");
        output[output_index] = 0xaaaa;
        self.state = 1;
        Ok(1)
    }

    unsafe fn decode(
        &mut self,
        input: &[u16],
        input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        Ok((u32::from(input[input_index]), crate::nz(1)))
    }

    unsafe fn encode(
        &mut self,
        value: &u32,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        assert_eq!(1, self.state, "encode must run after reset");
        output[output_index] = *value as u16;
        self.state = 2;
        Ok(1)
    }

    unsafe fn encode_finish(
        &mut self,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        assert_eq!(2, self.state, "encode finish must run after encode");
        output[output_index] = 0xbbbb;
        self.state = 0;
        Ok(1)
    }
}

#[derive(Debug, Default)]
struct ResetWidthCodec {
    pre_reset_queries: core::cell::Cell<usize>,
    reset: bool,
}

impl Codec for ResetWidthCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairEncodeError;
    type EncodeError = PairEncodeError;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_UNITS_PER_VALUE: usize = 2;

    fn encode_len(&self, _value: &u32) -> usize {
        if self.reset {
            2
        } else {
            self.pre_reset_queries.set(self.pre_reset_queries.get() + 1);
            1
        }
    }

    unsafe fn decode(
        &mut self,
        input: &[u16],
        input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        Ok((u32::from(input[input_index]), crate::nz(1)))
    }

    unsafe fn encode(
        &mut self,
        value: &u32,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        assert!(self.reset, "encode must observe reset state");
        output[output_index] = *value as u16;
        output[output_index + 1] = (*value >> 16) as u16;
        Ok(2)
    }

    unsafe fn encode_reset(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        self.reset = true;
        Ok(0)
    }

    unsafe fn encode_finish(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        self.reset = false;
        Ok(0)
    }
}

macro_rules! noop_reset {
    ($output:ty) => {
        fn reset(
            &mut self,
            output: &mut [$output],
            output_index: usize,
        ) -> Result<usize, TranscodeEncodeError<PairEncodeError, ()>> {
            TranscodeEncodeError::<PairEncodeError, ()>::ensure_output_index(
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
        ) -> Result<usize, TranscodeEncodeError<PairEncodeError, ()>> {
            TranscodeEncodeError::<PairEncodeError, ()>::ensure_output_index(
                output.len(),
                output_index,
            )?;
            Ok(0)
        }
    };
}

#[derive(Debug, Default)]
struct PairEncoder;

impl Transcoder for PairEncoder {
    type Input = u32;
    type Output = u16;
    type Error = TranscodeEncodeError<PairEncodeError, ()>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
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
    ) -> Result<TranscodeProgress, TranscodeEncodeError<PairEncodeError, ()>>
    {
        if input_index > input.len() {
            return Err(domain(PairEncodeError::BadInputIndex));
        }
        if output_index > output.len() {
            return Err(domain(PairEncodeError::BadOutputIndex));
        }
        let mut read = 0;
        let mut written = 0;
        while input_index + read < input.len() {
            if input[input_index + read] == u32::MAX {
                return Err(domain(PairEncodeError::BadInputIndex));
            }
            if output_index + written + 2 > output.len() {
                let available = output.len() - (output_index + written);
                return Ok(TranscodeProgress::need_output(
                    output_index + written,
                    crate::nz(2),
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
    ) -> Result<usize, TranscodeEncodeError<PairEncodeError, ()>> {
        Ok(0)
    }
}

#[derive(Debug, Default)]
struct FinishEncoder {
    finished: bool,
    fail_finish: bool,
}

impl Transcoder for FinishEncoder {
    type Input = u32;
    type Output = u16;
    type Error = TranscodeEncodeError<PairEncodeError, ()>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeEncodeError<PairEncodeError, ()>>
    {
        if input_index > input.len() {
            return Err(domain(PairEncodeError::BadInputIndex));
        }
        if output_index > output.len() {
            return Err(domain(PairEncodeError::BadOutputIndex));
        }
        if input_index == input.len() {
            return Ok(TranscodeProgress::complete(0, 0));
        }
        if output_index == output.len() {
            return Ok(TranscodeProgress::need_output(
                output_index,
                crate::nz(1),
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
    ) -> Result<usize, TranscodeEncodeError<PairEncodeError, ()>> {
        if self.fail_finish {
            return Err(domain(PairEncodeError::CapacityOverflow));
        }
        if self.finished {
            return Ok(0);
        }
        if output_index >= output.len() {
            return Err(domain(PairEncodeError::InsufficientOutput {
                output_index,
                required: 1,
                available: 0,
            }));
        }
        output[output_index] = 0xeeee;
        self.finished = true;
        Ok(1)
    }
}

#[derive(Debug, Default)]
struct TwoUnitFinishEncoder;

impl Transcoder for TwoUnitFinishEncoder {
    type Input = u32;
    type Output = u16;
    type Error = TranscodeEncodeError<PairEncodeError, ()>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(2)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeEncodeError<PairEncodeError, ()>>
    {
        if input_index > input.len() {
            return Err(domain(PairEncodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(
        &mut self,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<usize, TranscodeEncodeError<PairEncodeError, ()>> {
        output[output_index] = 0xaaaa;
        output[output_index + 1] = 0xbbbb;
        Ok(2)
    }
}

#[derive(Debug, Default)]
struct OverreportedFinishEncoder;

impl Transcoder for OverreportedFinishEncoder {
    type Input = u32;
    type Output = u16;
    type Error = TranscodeEncodeError<PairEncodeError, ()>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeEncodeError<PairEncodeError, ()>>
    {
        Ok(TranscodeProgress::complete(input.len() - input_index, 0))
    }

    fn finish(
        &mut self,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<usize, TranscodeEncodeError<PairEncodeError, ()>> {
        output[output_index] = 0xaaaa;
        Ok(2)
    }
}

#[derive(Debug, Default)]
struct ZeroWidthFailingFinishEncoder;

impl Transcoder for ZeroWidthFailingFinishEncoder {
    type Input = u32;
    type Output = u16;
    type Error = TranscodeEncodeError<PairEncodeError, ()>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
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
    ) -> Result<TranscodeProgress, TranscodeEncodeError<PairEncodeError, ()>>
    {
        if input_index > input.len() {
            return Err(domain(PairEncodeError::BadInputIndex));
        }
        if output_index > output.len() {
            return Err(domain(PairEncodeError::BadOutputIndex));
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, TranscodeEncodeError<PairEncodeError, ()>> {
        Err(domain(PairEncodeError::BadInputIndex))
    }
}

#[derive(Debug, Default)]
struct UnitOutput {
    units: Vec<u16>,
    flushed: bool,
    fail_write: bool,
    fail_flush: bool,
}

impl Output for UnitOutput {
    type Item = u16;

    unsafe fn write_unchecked(
        &mut self,
        input: &[u16],
        index: usize,
        count: usize,
    ) -> std::io::Result<usize> {
        if self.fail_write {
            return Err(Error::new(
                ErrorKind::BrokenPipe,
                "output write failure",
            ));
        }
        self.units.extend_from_slice(&input[index..index + count]);
        Ok(count)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.fail_flush {
            return Err(Error::new(
                ErrorKind::BrokenPipe,
                "output flush failure",
            ));
        }
        self.flushed = true;
        Ok(())
    }
}

#[derive(Debug, Default)]
struct CapacityBoundEncoder;

impl Transcoder for CapacityBoundEncoder {
    type Input = u32;
    type Output = u16;
    type Error = TranscodeEncodeError<PairEncodeError, ()>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
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
    ) -> Result<TranscodeProgress, TranscodeEncodeError<PairEncodeError, ()>>
    {
        if input_index > input.len() {
            return Err(domain(PairEncodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    noop_finish!(u16);
}

#[derive(Clone, Copy, Debug)]
enum FinishFailure {
    InvalidIndex,
    InsufficientOutput,
}

#[derive(Debug)]
struct FailingFinishEncoder {
    failure: FinishFailure,
}

impl Transcoder for FailingFinishEncoder {
    type Input = u32;
    type Output = u16;
    type Error = TranscodeEncodeError<PairEncodeError, ()>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
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
    ) -> Result<TranscodeProgress, TranscodeEncodeError<PairEncodeError, ()>>
    {
        if input_index > input.len() {
            return Err(domain(PairEncodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, TranscodeEncodeError<PairEncodeError, ()>> {
        match self.failure {
            FinishFailure::InvalidIndex => {
                Err(domain(PairEncodeError::InvalidOutputIndex {
                    index: 4,
                    len: 1,
                }))
            }
            FinishFailure::InsufficientOutput => {
                Err(domain(PairEncodeError::InsufficientOutput {
                    output_index: 0,
                    required: 2,
                    available: 1,
                }))
            }
        }
    }
}

#[derive(Debug, Default)]
struct NeedInputEncoder;

impl Transcoder for NeedInputEncoder {
    type Input = u32;
    type Output = u16;
    type Error = TranscodeEncodeError<PairEncodeError, ()>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeEncodeError<PairEncodeError, ()>>
    {
        if input_index > input.len() {
            return Err(domain(PairEncodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::need_input(
            input_index,
            crate::nz(2),
            input.len() - input_index,
            0,
            0,
        ))
    }

    noop_finish!(u16);
}

#[cfg(debug_assertions)]
#[derive(Debug, Default)]
struct NeedOutputAfterReadEncoder;

#[cfg(debug_assertions)]
impl Transcoder for NeedOutputAfterReadEncoder {
    type Input = u32;
    type Output = u16;
    type Error = TranscodeEncodeError<PairEncodeError, ()>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeEncodeError<PairEncodeError, ()>>
    {
        if input_index > input.len() {
            return Err(domain(PairEncodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::need_output(
            output_index,
            crate::nz(1),
            output.len() - output_index,
            1,
            0,
        ))
    }

    noop_finish!(u16);
}

#[derive(Debug, Default)]
struct NeedOutputAfterWriteEncoder;

impl Transcoder for NeedOutputAfterWriteEncoder {
    type Input = u32;
    type Output = u16;
    type Error = TranscodeEncodeError<PairEncodeError, ()>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeEncodeError<PairEncodeError, ()>>
    {
        if input_index >= input.len() {
            return Err(domain(PairEncodeError::BadInputIndex));
        }
        output[output_index] = input[input_index] as u16;
        Ok(TranscodeProgress::need_output(
            output_index + 1,
            crate::nz(1),
            output.len() - (output_index + 1),
            1,
            1,
        ))
    }

    noop_finish!(u16);
}

#[derive(Debug, Default)]
struct NeedOutputAfterReadPastCapacityEncoder;

impl Transcoder for NeedOutputAfterReadPastCapacityEncoder {
    type Input = u32;
    type Output = u16;
    type Error = TranscodeEncodeError<PairEncodeError, ()>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeEncodeError<PairEncodeError, ()>>
    {
        if input_index >= input.len() {
            return Err(domain(PairEncodeError::BadInputIndex));
        }
        output[output_index] = input[input_index] as u16;
        Ok(TranscodeProgress::need_output(
            output_index + 1,
            crate::nz(2),
            output.len() - (output_index + 1),
            1,
            1,
        ))
    }

    noop_finish!(u16);
}

#[derive(Debug, Default)]
struct PrefixBeforeReadEncoder {
    emitted_prefix: bool,
}

impl Transcoder for PrefixBeforeReadEncoder {
    type Input = u32;
    type Output = u16;
    type Error = TranscodeEncodeError<PairEncodeError, ()>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        input_len
            .checked_add(1)
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeEncodeError<PairEncodeError, ()>>
    {
        if input_index > input.len() {
            return Err(domain(PairEncodeError::BadInputIndex));
        }
        if output_index > output.len() {
            return Err(domain(PairEncodeError::BadOutputIndex));
        }
        if !self.emitted_prefix {
            if output_index == output.len() {
                return Ok(TranscodeProgress::need_output(
                    output_index,
                    crate::nz(1),
                    0,
                    0,
                    0,
                ));
            }
            output[output_index] = 0xaaaa;
            self.emitted_prefix = true;
            return Ok(TranscodeProgress::need_output(
                output_index + 1,
                crate::nz(1),
                output.len() - (output_index + 1),
                0,
                1,
            ));
        }
        if input_index == input.len() {
            return Ok(TranscodeProgress::complete(0, 0));
        }
        if output_index == output.len() {
            return Ok(TranscodeProgress::need_output(
                output_index,
                crate::nz(1),
                0,
                0,
                0,
            ));
        }
        output[output_index] = input[input_index] as u16;
        Ok(TranscodeProgress::complete(1, 1))
    }

    noop_finish!(u16);
}

#[derive(Debug, Default)]
struct OverreadingProgressEncoder;

impl Transcoder for OverreadingProgressEncoder {
    type Input = u32;
    type Output = u16;
    type Error = TranscodeEncodeError<PairEncodeError, ()>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeEncodeError<PairEncodeError, ()>>
    {
        if input_index > input.len() {
            return Err(domain(PairEncodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::complete(input.len() + 1, 0))
    }

    noop_finish!(u16);
}

#[derive(Debug, Default)]
struct OverwritingProgressEncoder;

impl Transcoder for OverwritingProgressEncoder {
    type Input = u32;
    type Output = u16;
    type Error = TranscodeEncodeError<PairEncodeError, ()>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len + 1)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeEncodeError<PairEncodeError, ()>>
    {
        if input_index > input.len() {
            return Err(domain(PairEncodeError::BadInputIndex));
        }
        let available = output.len() - output_index;
        Ok(TranscodeProgress::complete(0, available + 1))
    }

    noop_finish!(u16);
}

#[cfg(debug_assertions)]
#[derive(Debug, Default)]
struct OverflowingNeedOutputEncoder;

#[cfg(debug_assertions)]
impl Transcoder for OverflowingNeedOutputEncoder {
    type Input = u32;
    type Output = u16;
    type Error = TranscodeEncodeError<PairEncodeError, ()>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeEncodeError<PairEncodeError, ()>>
    {
        if input_index > input.len() {
            return Err(domain(PairEncodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::need_output(
            output_index,
            crate::nz(1),
            output.len() - output_index,
            0,
            0,
        ))
    }

    noop_finish!(u16);
}

#[cfg(debug_assertions)]
#[derive(Debug, Default)]
struct MisindexedNeedOutputEncoder;

#[cfg(debug_assertions)]
impl Transcoder for MisindexedNeedOutputEncoder {
    type Input = u32;
    type Output = u16;
    type Error = TranscodeEncodeError<PairEncodeError, ()>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    noop_reset!(u16);

    fn transcode(
        &mut self,
        input: &[u32],
        input_index: usize,
        _output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeEncodeError<PairEncodeError, ()>>
    {
        if input_index > input.len() {
            return Err(domain(PairEncodeError::BadInputIndex));
        }
        Ok(TranscodeProgress::need_output(
            output_index + 1,
            crate::nz(1),
            0,
            0,
            0,
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

fn map_error(error: TranscodeEncodeError<PairEncodeError, ()>) -> Error {
    Error::new(ErrorKind::InvalidData, format!("{error:?}"))
}

fn map_codec_error(error: PairEncodeError) -> Error {
    Error::new(ErrorKind::InvalidData, error)
}

fn panic_codec_error(_error: PairEncodeError) -> Error {
    panic!("framework failures must not call the domain mapper")
}

fn encode_with<E>(
    output: &mut TranscodeEncodeOutput<UnitOutput>,
    encoder: &mut E,
    input: &[u32],
    input_index: usize,
    count: usize,
) -> std::io::Result<usize>
where
    E: Transcoder<
            Input = u32,
            Output = u16,
            Error = TranscodeEncodeError<PairEncodeError, ()>,
        >,
{
    let mut mapper: fn(TranscodeEncodeError<PairEncodeError, ()>) -> Error =
        map_error;
    output.transcode_from(encoder, &mut mapper, input, input_index, count)
}

fn finish_with<E>(
    output: &mut TranscodeEncodeOutput<UnitOutput>,
    encoder: &mut E,
) -> std::io::Result<()>
where
    E: Transcoder<
            Input = u32,
            Output = u16,
            Error = TranscodeEncodeError<PairEncodeError, ()>,
        >,
{
    let mut mapper: fn(TranscodeEncodeError<PairEncodeError, ()>) -> Error =
        map_error;
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
fn test_buffered_encode_output_writes_one_codec_value() {
    let mut output =
        TranscodeEncodeOutput::with_capacity(UnitOutput::default(), 2);
    let mut codec = ScriptedEncodeCodec::new(ScriptedEncodeMode::Success);

    output
        .write_encoded_with(&mut codec, &0x1234_5678, map_codec_error)
        .expect("one codec value should encode into the output buffer");
    output.flush().expect("encoded units should flush");

    assert_eq!(&[0x5678, 0x1234], output.inner().units.as_slice());
}

#[test]
fn test_buffered_encode_output_returns_zero_for_zero_count() {
    let output = UnitOutput::default();
    let mut encoder = PairEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);
    let written = encode_with(&mut output, &mut encoder, &[0x0001_0002], 0, 0)
        .expect("zero-count write should be a no-op");

    assert_eq!(0, written);
    assert!(output.inner().units.is_empty());
}

#[test]
fn test_buffered_encode_output_transcode_from_respects_input_range() {
    let output = UnitOutput::default();
    let mut encoder = PairEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 4);
    let mut mapper: fn(TranscodeEncodeError<PairEncodeError, ()>) -> Error =
        map_error;

    let written = output
        .transcode_from(&mut encoder, &mut mapper, &[0x0001_0002], 0, 1)
        .expect("checked encode should accept a valid input range");

    output.flush().expect("flush should drain encoded units");

    assert_eq!(1, written);
    assert_eq!(&[1, 2], output.inner().units.as_slice());
}

#[test]
fn test_buffered_encode_output_transcode_from_rejects_invalid_input_range() {
    let output = UnitOutput::default();
    let mut encoder = PairEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 4);
    let mut mapper: fn(TranscodeEncodeError<PairEncodeError, ()>) -> Error =
        map_error;

    let error = output
        .transcode_from(&mut encoder, &mut mapper, &[0x0001_0002], 1, 1)
        .expect_err("invalid input range should be rejected before encoding");

    assert_eq!(ErrorKind::InvalidInput, error.kind());
    assert_eq!(
        "encode input range exceeds source buffer",
        error.to_string(),
    );
}

#[test]
fn test_buffered_encode_output_encodes_and_flushes_units() {
    let output = UnitOutput::default();
    let mut encoder = PairEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);
    let written = encode_with(
        &mut output,
        &mut encoder,
        &[0x0001_0002, 0x0003_0004],
        0,
        2,
    )
    .expect("encoding should accept both values");
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
    let written = encode_with(&mut output, &mut encoder, &[0x0001_0002], 0, 1)
        .expect("first value should fill the unit buffer");
    assert_eq!(1, written);
    assert!(output.inner().units.is_empty());
    let written = encode_with(&mut output, &mut encoder, &[0x0003_0004], 0, 1)
        .expect("second value should flush the full buffer first");
    assert_eq!(1, written);

    output.flush().expect("flush should drain buffered units");
    assert_eq!(&[1, 2, 3, 4], output.inner().units.as_slice());
}

#[test]
fn test_buffered_encode_output_defers_entry_flush_error_until_flush() {
    let output = FixedCapacityOutput::new(0);
    let mut encoder = FinishEncoder::default();
    let mut output = TranscodeEncodeOutput::with_capacity(output, 1);
    let mut mapper: fn(TranscodeEncodeError<PairEncodeError, ()>) -> Error =
        map_error;

    let written = output
        .transcode_from(&mut encoder, &mut mapper, &[0x1234], 0, 1)
        .expect("first value should remain pending in the internal buffer");
    assert_eq!(1, written);
    assert_eq!(0, output.spare_capacity());

    let written = output
        .transcode_from(&mut encoder, &mut mapper, &[0x5678], 0, 1)
        .expect("persistent buffer should grow without an entry flush");
    assert_eq!(1, written);

    let error = output.flush().expect_err("explicit flush should fail");
    assert_eq!(ErrorKind::InvalidInput, error.kind());
    assert!(error.to_string().contains("fixed output capacity exceeded"));
}

#[test]
fn test_buffered_encode_output_grows_for_no_progress_need_output() {
    let output = UnitOutput::default();
    let mut encoder = PairEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 1);
    let written = encode_with(&mut output, &mut encoder, &[0x0001_0002], 0, 1)
        .expect("persistent buffer should grow for the required output");

    assert_eq!(1, written);
}

#[cfg(debug_assertions)]
#[test]
fn test_buffered_encode_output_returns_after_need_output_consumes_input() {
    let output = UnitOutput::default();
    let mut encoder = NeedOutputAfterReadEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 1);
    let error = encode_with(&mut output, &mut encoder, &[0x1234], 0, 1)
        .expect_err("NeedOutput with available space violates progress");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("reported required"));
}

#[test]
fn test_buffered_encode_output_reports_transcoder_errors_as_io_errors() {
    let output = UnitOutput::default();
    let mut encoder = PairEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);
    let input = [u32::MAX];
    let error = encode_with(&mut output, &mut encoder, &input, 0, 1)
        .expect_err("encoder error should be mapped to I/O error");

    assert_eq!(ErrorKind::InvalidData, error.kind());
}

#[test]
fn test_buffered_encode_output_rejects_need_input_status() {
    let output = UnitOutput::default();
    let mut encoder = NeedInputEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);
    let error = encode_with(&mut output, &mut encoder, &[0x1234], 0, 1)
        .expect_err("encoder NeedInput status should be rejected");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(
        error
            .to_string()
            .contains("unexpectedly requested more input")
    );
}

#[test]
fn test_buffered_encode_output_rejects_overreported_read_progress() {
    let output = UnitOutput::default();
    let mut encoder = OverreadingProgressEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);
    let error = encode_with(&mut output, &mut encoder, &[0x1234], 0, 1)
        .expect_err("overreported input progress should be rejected");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("consumed"));
    assert!(error.to_string().contains("only"));
}

#[test]
fn test_buffered_encode_output_rejects_overreported_write_progress() {
    let output = UnitOutput::default();
    let mut encoder = OverwritingProgressEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 1);
    let error = encode_with(&mut output, &mut encoder, &[0x1234], 0, 1)
        .expect_err("overreported output progress should be rejected");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("wrote"));
    assert!(error.to_string().contains("output slots"));
}

#[cfg(debug_assertions)]
#[test]
fn test_buffered_encode_output_rejects_overflowing_need_output() {
    let output = UnitOutput::default();
    let mut encoder = OverflowingNeedOutputEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);
    let error = encode_with(&mut output, &mut encoder, &[0x1234], 0, 1)
        .expect_err("satisfied NeedOutput requirement should be rejected");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("reported required"));
}

#[cfg(debug_assertions)]
#[test]
fn test_buffered_encode_output_rejects_misindexed_need_output() {
    let output = UnitOutput::default();
    let mut encoder = MisindexedNeedOutputEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);
    let error = encode_with(&mut output, &mut encoder, &[0x1234], 0, 1)
        .expect_err("misindexed NeedOutput status should be rejected");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("reported status index"));
}

#[test]
fn test_buffered_encode_output_flush_does_not_finish_encoder() {
    let output = UnitOutput::default();
    let mut encoder = FinishEncoder::default();
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);
    let written = encode_with(&mut output, &mut encoder, &[0x1234], 0, 1)
        .expect("encoding should accept the value");
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

    assert_eq!(Ok(1), encoder.max_finish_output_len());
    finish_with(&mut output, &mut encoder)
        .expect("finish should write trailer and flush");

    assert_eq!(&[0xeeee], output.inner().units.as_slice());
    assert!(output.inner().flushed);
    assert_eq!(Ok(1), encoder.max_finish_output_len());
    output.inner_mut().flushed = false;
    output
        .flush()
        .expect("explicit flush should be harmless after finish");
    assert_eq!(&[0xeeee], output.inner().units.as_slice());
}

#[test]
#[should_panic(expected = "finish wrote beyond its bound")]
fn test_buffered_encode_output_finish_panics_when_encoder_overreports_bound() {
    let output = UnitOutput::default();
    let mut encoder = OverreportedFinishEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);
    let mut mapper: fn(TranscodeEncodeError<PairEncodeError, ()>) -> Error =
        map_error;

    let _ = output.finish(&mut encoder, &mut mapper);
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
    let output = UnitOutput::default();
    let mut encoder = FinishEncoder {
        fail_finish: true,
        ..FinishEncoder::default()
    };
    let mut output = TranscodeEncodeOutput::with_capacity(output, 3);
    let error = finish_with(&mut output, &mut encoder)
        .expect_err("finish capacity failure should be mapped to I/O");
    assert_eq!(ErrorKind::InvalidData, error.kind());

    for failure in [
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
    let mut mapper: fn(TranscodeEncodeError<PairEncodeError, ()>) -> Error =
        map_error;
    let first = output
        .transcode_from(&mut first_encoder, &mut mapper, &[0x0001_0002], 0, 1)
        .expect("first encoder should write one value");
    let second = output
        .transcode_from(&mut second_encoder, &mut mapper, &[0x0003_0004], 0, 1)
        .expect("second encoder should reuse the same buffer");

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
    let first = encode_with(&mut output, &mut encoder, &[0x0001_0002], 0, 1)
        .expect("first value should fill the spare buffer");
    assert_eq!(1, first);
    assert_eq!(0, output.spare_capacity());
    let second = encode_with(&mut output, &mut encoder, &[0x0003_0004], 0, 1)
        .expect("second value should flush before encoding");
    assert_eq!(1, second);
    output.flush().expect("flush should drain buffered units");
    assert_eq!(&[1, 2, 3, 4], output.inner().units.as_slice());
}

#[test]
fn test_buffered_encode_output_flushes_after_partial_need_output_progress() {
    let output = UnitOutput::default();
    let mut encoder = NeedOutputAfterWriteEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 1);
    let written = encode_with(&mut output, &mut encoder, &[0x1234], 0, 1)
        .expect("partial need-output progress should flush buffered units");

    assert_eq!(1, written);
    output.flush().expect("flush should drain buffered units");
    assert_eq!(&[0x1234], output.inner().units.as_slice());
}

#[test]
fn test_buffered_encode_output_grows_for_post_read_need_output() {
    let output = FixedCapacityOutput::new(1);
    let mut encoder = NeedOutputAfterReadPastCapacityEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 1);
    let mut mapper: fn(TranscodeEncodeError<PairEncodeError, ()>) -> Error =
        map_error;

    let written = output
        .transcode_from(&mut encoder, &mut mapper, &[0x1234], 0, 1)
        .expect("post-read NeedOutput should grow the persistent buffer");

    assert_eq!(1, written);
}

#[test]
fn test_buffered_encode_output_retries_after_need_output_without_reading() {
    let output = UnitOutput::default();
    let mut encoder = PrefixBeforeReadEncoder::default();
    let mut output = TranscodeEncodeOutput::with_capacity(output, 1);
    let written = encode_with(&mut output, &mut encoder, &[0x1234], 0, 1)
        .expect("encoder should grow and retain the prefix with the value");

    assert_eq!(1, written);
    assert!(output.inner().units.is_empty());

    output
        .flush()
        .expect("flush should drain final buffered unit");
    assert_eq!(&[0xaaaa, 0x1234], output.inner().units.as_slice());
}

#[test]
fn test_buffered_encode_output_finish_reports_spare_capacity_error() {
    let output = FixedCapacityOutput::new(0);
    let mut encoder = FinishEncoder::default();
    let mut output = TranscodeEncodeOutput::with_capacity(output, 1);
    let mut mapper: fn(TranscodeEncodeError<PairEncodeError, ()>) -> Error =
        map_error;

    let error = output
        .finish(&mut encoder, &mut mapper)
        .expect_err("finish should report spare-capacity errors");

    assert_eq!(ErrorKind::InvalidInput, error.kind());
}

#[test]
fn test_buffered_encode_output_finish_grows_for_required_spare_capacity() {
    let output = UnitOutput::default();
    let mut encoder = TwoUnitFinishEncoder;
    let mut output = TranscodeEncodeOutput::with_capacity(output, 1);
    let mut mapper: fn(TranscodeEncodeError<PairEncodeError, ()>) -> Error =
        map_error;

    output
        .finish(&mut encoder, &mut mapper)
        .expect("finish should grow and reserve its full bound before writing");

    assert_eq!(2, output.inner().units.len());
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScriptedEncodeMode {
    Success,
    Reject,
    EncodeError,
    FinishError,
    ExactLenExceedsBound,
}

#[derive(Debug)]
struct ScriptedEncodeCodec {
    mode: ScriptedEncodeMode,
}

impl ScriptedEncodeCodec {
    fn new(mode: ScriptedEncodeMode) -> Self {
        Self { mode }
    }
}

impl Codec for ScriptedEncodeCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairEncodeError;
    type EncodeError = PairEncodeError;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_RESET_UNITS: usize = 1;
    const MAX_ENCODE_FINISH_UNITS: usize = 1;

    fn can_encode_value(&self, _value: &u32) -> bool {
        self.mode != ScriptedEncodeMode::Reject
    }

    fn encode_len(&self, _value: &u32) -> usize {
        match self.mode {
            ScriptedEncodeMode::ExactLenExceedsBound => 4,
            _ => 2,
        }
    }

    unsafe fn decode(
        &mut self,
        input: &[u16],
        input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        Ok((u32::from(input[input_index]), crate::nz(1)))
    }

    unsafe fn encode(
        &mut self,
        value: &u32,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        if self.mode == ScriptedEncodeMode::EncodeError {
            return Err(PairEncodeError::BadOutputIndex);
        }
        output[output_index] = *value as u16;
        output[output_index + 1] = (*value >> 16) as u16;
        Ok(2)
    }

    unsafe fn encode_reset(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Ok(0)
    }

    unsafe fn encode_finish(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        if self.mode == ScriptedEncodeMode::FinishError {
            return Err(PairEncodeError::BadInputIndex);
        }
        Ok(0)
    }
}

#[derive(Debug, Default)]
struct ZeroWriteOutput;

impl Output for ZeroWriteOutput {
    type Item = u8;

    unsafe fn write_unchecked(
        &mut self,
        _input: &[u8],
        _index: usize,
        _count: usize,
    ) -> std::io::Result<usize> {
        Ok(0)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_buffered_encode_output_write_encoded_grows_persistent_buffer() {
    let mut output =
        TranscodeEncodeOutput::with_capacity(UnitOutput::default(), 0);
    let mut codec = ScriptedEncodeCodec::new(ScriptedEncodeMode::Success);

    output
        .write_encoded_with(&mut codec, &0x0001_0002, map_codec_error)
        .expect("large value should grow the persistent buffer");
    output.flush().expect("encoded units should flush");

    assert_eq!(&[0x0002, 0x0001], output.inner().units.as_slice());
    assert!(output.spare_capacity() >= 2);
}

#[test]
fn test_buffered_encode_output_write_encoded_runs_complete_lifecycle() {
    let mut output =
        TranscodeEncodeOutput::with_capacity(UnitOutput::default(), 3);
    let mut codec = CompleteEncodeLifecycleCodec::default();

    output
        .write_encoded_with(&mut codec, &0x1234, map_codec_error)
        .expect("first value should complete its encode lifecycle");
    output
        .write_encoded_with(&mut codec, &0x5678, map_codec_error)
        .expect("second value should start a fresh encode lifecycle");
    output.flush().expect("encoded units should flush");

    assert_eq!(
        &[0xaaaa, 0x1234, 0xbbbb, 0xaaaa, 0x5678, 0xbbbb],
        output.inner().units.as_slice(),
    );
}

/// Verifies that buffered complete encoding sizes values after codec reset.
#[test]
fn test_write_encoded_with_queries_width_after_reset() {
    let mut output =
        TranscodeEncodeOutput::with_capacity(UnitOutput::default(), 2);
    let mut codec = ResetWidthCodec::default();

    output
        .write_encoded_with(&mut codec, &0x1234_5678, map_codec_error)
        .expect("reset-state width should fit the reserved codec bound");
    output.flush().expect("encoded units should flush");

    assert_eq!(&[0x5678, 0x1234], output.inner().units.as_slice());
    assert_eq!(0, codec.pre_reset_queries.get());
}

#[test]
fn test_buffered_encode_output_write_encoded_maps_domain_error() {
    let mut output =
        TranscodeEncodeOutput::with_capacity(UnitOutput::default(), 4);
    let mut codec = ScriptedEncodeCodec::new(ScriptedEncodeMode::EncodeError);

    let error = output
        .write_encoded_with(&mut codec, &0x0001_0002, map_codec_error)
        .expect_err("domain encode failure should be mapped");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert_eq!("bad output index", error.to_string());
}

#[test]
fn test_buffered_encode_output_write_encoded_reports_unencodable() {
    let mut output =
        TranscodeEncodeOutput::with_capacity(UnitOutput::default(), 4);
    let mut codec = ScriptedEncodeCodec::new(ScriptedEncodeMode::Reject);

    let error = output
        .write_encoded_with(&mut codec, &0x0001_0002, map_codec_error)
        .expect_err("unencodable values should be rejected");

    assert_eq!(ErrorKind::InvalidInput, error.kind());
    assert_eq!("codec cannot encode value", error.to_string());
}

#[test]
fn test_buffered_encode_output_write_all_reports_write_zero() {
    let mut output = TranscodeEncodeOutput::with_capacity(ZeroWriteOutput, 0);

    let error = Write::write_all(&mut output, &[1, 2, 3])
        .expect_err("zero-length writes should surface WriteZero");

    assert_eq!(ErrorKind::WriteZero, error.kind());
}

#[derive(Debug, Default)]
struct OverflowEncodeBoundCodec;

impl Codec for OverflowEncodeBoundCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = PairEncodeError;
    type EncodeError = PairEncodeError;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_RESET_UNITS: usize = usize::MAX;
    const MAX_ENCODE_FINISH_UNITS: usize = usize::MAX;

    unsafe fn decode(
        &mut self,
        input: &[u16],
        input_index: usize,
    ) -> Result<(u32, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        Ok((u32::from(input[input_index]), crate::nz(1)))
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

#[derive(Debug)]
struct BrokenPipeByteOutput;

impl Output for BrokenPipeByteOutput {
    type Item = u8;

    unsafe fn write_unchecked(
        &mut self,
        _input: &[u8],
        _index: usize,
        _count: usize,
    ) -> std::io::Result<usize> {
        Err(Error::new(ErrorKind::BrokenPipe, "byte output failure"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Err(Error::new(
            ErrorKind::BrokenPipe,
            "byte output flush failure",
        ))
    }
}

#[test]
fn test_buffered_encode_output_write_encoded_reports_output_bound_overflow() {
    let mut output =
        TranscodeEncodeOutput::with_capacity(UnitOutput::default(), 4);
    let mut codec = OverflowEncodeBoundCodec;

    let error = output
        .write_encoded_with(&mut codec, &0x0001_0002, map_codec_error)
        .expect_err("encode bound overflow should be mapped to invalid input");

    assert_eq!(ErrorKind::InvalidInput, error.kind());
    assert_eq!("codec output bound overflow", error.to_string());
}

#[test]
#[should_panic(
    expected = "Codec::encode_len exceeded Codec::MAX_UNITS_PER_VALUE"
)]
fn test_buffered_encode_output_write_encoded_rejects_width_beyond_bound() {
    let mut output =
        TranscodeEncodeOutput::with_capacity(UnitOutput::default(), 4);
    let mut codec =
        ScriptedEncodeCodec::new(ScriptedEncodeMode::ExactLenExceedsBound);

    let _ =
        output.write_encoded_with(&mut codec, &0x0001_0002, panic_codec_error);
}

#[test]
#[should_panic(
    expected = "Codec::encode_len exceeded Codec::MAX_UNITS_PER_VALUE"
)]
fn test_buffered_encode_output_write_encoded_rejects_width_beyond_bound_via_scratch()
 {
    let mut output =
        TranscodeEncodeOutput::with_capacity(UnitOutput::default(), 0);
    let mut codec =
        ScriptedEncodeCodec::new(ScriptedEncodeMode::ExactLenExceedsBound);

    let _ =
        output.write_encoded_with(&mut codec, &0x0001_0002, panic_codec_error);
}

#[test]
fn test_buffered_encode_output_write_encoded_propagates_non_capacity_spare_errors()
 {
    let inner = UnitOutput {
        fail_write: true,
        fail_flush: true,
        ..UnitOutput::default()
    };
    let mut output = TranscodeEncodeOutput::with_capacity(inner, 0);
    let mut codec = ScriptedEncodeCodec::new(ScriptedEncodeMode::Success);

    let error = output
        .write_encoded_with(&mut codec, &0x0001_0002, map_codec_error)
        .and_then(|()| output.flush())
        .expect_err("buffer flush errors should propagate");

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}

#[test]
fn test_buffered_encode_output_write_encoded_scratch_propagates_flush_errors() {
    let inner = UnitOutput {
        fail_flush: true,
        ..UnitOutput::default()
    };
    let mut output = TranscodeEncodeOutput::with_capacity(inner, 0);
    let mut codec = ScriptedEncodeCodec::new(ScriptedEncodeMode::Success);

    let error = output
        .write_encoded_with(&mut codec, &0x0001_0002, map_codec_error)
        .and_then(|()| output.flush())
        .expect_err("persistent-buffer flush should propagate flush failures");

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}

#[test]
fn test_buffered_encode_output_write_encoded_scratch_propagates_write_errors() {
    let inner = UnitOutput {
        fail_write: true,
        ..UnitOutput::default()
    };
    let mut output = TranscodeEncodeOutput::with_capacity(inner, 0);
    let mut codec = ScriptedEncodeCodec::new(ScriptedEncodeMode::Success);

    let error = output
        .write_encoded_with(&mut codec, &0x0001_0002, map_codec_error)
        .and_then(|()| output.flush())
        .expect_err("persistent-buffer flush should propagate write failures");

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}

#[test]
fn test_buffered_encode_output_write_encoded_maps_domain_error_via_scratch() {
    let mut output =
        TranscodeEncodeOutput::with_capacity(UnitOutput::default(), 0);
    let mut codec = ScriptedEncodeCodec::new(ScriptedEncodeMode::EncodeError);

    let error = output
        .write_encoded_with(&mut codec, &0x0001_0002, map_codec_error)
        .expect_err("scratch encode should map domain failures");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert_eq!("bad output index", error.to_string());
}

#[test]
fn test_buffered_encode_output_write_encoded_maps_finish_error_via_scratch() {
    let mut output =
        TranscodeEncodeOutput::with_capacity(UnitOutput::default(), 0);
    let mut codec = ScriptedEncodeCodec::new(ScriptedEncodeMode::FinishError);

    let error = output
        .write_encoded_with(&mut codec, &0x0001_0002, map_codec_error)
        .expect_err("scratch encode should map finish failures");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert_eq!("bad input index", error.to_string());
}

#[test]
fn test_buffered_encode_output_write_all_propagates_write_errors() {
    let mut output =
        TranscodeEncodeOutput::with_capacity(BrokenPipeByteOutput, 1);

    let error = Write::write_all(&mut output, &[1, 2])
        .expect_err("write_all should propagate output write errors");

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}

#[test]
fn test_buffered_encode_output_write_encoded_defers_flush_errors() {
    let inner = UnitOutput {
        fail_write: true,
        fail_flush: true,
        ..UnitOutput::default()
    };
    let mut output = TranscodeEncodeOutput::with_capacity(inner, 4);
    let mut codec = ScriptedEncodeCodec::new(ScriptedEncodeMode::Success);

    output
        .write_encoded_with(&mut codec, &0x0001_0002, map_codec_error)
        .expect("first value should leave less than one complete bound");
    assert_eq!(2, output.spare_capacity());

    output
        .write_encoded_with(&mut codec, &0x0003_0004, map_codec_error)
        .expect("persistent buffer growth should not flush");

    let error = output.flush().expect_err("explicit flush should fail");
    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}

#[test]
fn test_buffered_encode_output_debug_shows_wrapped_output() {
    let output = TranscodeEncodeOutput::with_capacity(UnitOutput::default(), 2);
    let debug = format!("{output:?}");

    assert!(debug.contains("TranscodeEncodeOutput"));
    assert!(debug.contains("output"));
}
