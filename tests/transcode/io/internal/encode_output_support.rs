// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::Error;
use std::io::ErrorKind;
use std::io::Result as IoResult;

use qubit_codec::TranscodeEncodeError;
use qubit_codec::TranscodeEncodeOutput;
use qubit_codec::TranscodeEncoder;
use qubit_codec::Transcoder;
use qubit_io::Output;

use super::PairEncodeError;

/// Provides a configurable in-memory output for synchronous encoding tests.
#[derive(Debug, Default)]
pub(in crate::transcode::io) struct UnitOutput {
    pub(in crate::transcode::io) units: Vec<u16>,
    pub(in crate::transcode::io) flushed: bool,
    pub(in crate::transcode::io) fail_write: bool,
    pub(in crate::transcode::io) fail_flush: bool,
}

impl Output for UnitOutput {
    type Item = u16;

    unsafe fn write_unchecked(&mut self, input: &[u16], index: usize, count: usize) -> IoResult<usize> {
        if self.fail_write {
            return Err(Error::new(ErrorKind::BrokenPipe, "output write failure"));
        }
        self.units.extend_from_slice(&input[index..index + count]);
        Ok(count)
    }

    fn flush(&mut self) -> IoResult<()> {
        if self.fail_flush {
            return Err(Error::new(ErrorKind::BrokenPipe, "output flush failure"));
        }
        self.flushed = true;
        Ok(())
    }
}

/// Maps framework and codec errors to invalid-data I/O errors for the tests.
pub(in crate::transcode::io) fn map_error(error: TranscodeEncodeError<PairEncodeError, u32>) -> Error {
    Error::new(ErrorKind::InvalidData, format!("{error:?}"))
}

/// Maps codec-domain errors to invalid-data I/O errors for the tests.
pub(in crate::transcode::io) fn map_codec_error(error: PairEncodeError) -> Error {
    Error::new(ErrorKind::InvalidData, error)
}

/// Panics if framework failures incorrectly reach the codec-domain mapper.
pub(in crate::transcode::io) fn panic_codec_error(_error: PairEncodeError) -> Error {
    panic!("framework failures must not call the domain mapper")
}

/// Encodes a selected input range using the shared unit-output test policy.
pub(in crate::transcode::io) fn encode_with<E>(
    output: &mut TranscodeEncodeOutput<UnitOutput>,
    encoder: &mut E,
    input: &[u32],
    input_index: usize,
    count: usize,
) -> IoResult<usize>
where
    E: TranscodeEncoder<
            Input = u32,
            Output = u16,
            EncodeError = PairEncodeError,
            Error = TranscodeEncodeError<PairEncodeError, u32>,
        >,
{
    let mut mapper: fn(TranscodeEncodeError<PairEncodeError, u32>) -> Error = map_error;
    output.transcode(encoder, &mut mapper, input, input_index, count)
}

/// Finishes an encoder using the shared unit-output test policy.
pub(in crate::transcode::io) fn finish_with<E>(
    output: &mut TranscodeEncodeOutput<UnitOutput>,
    encoder: &mut E,
) -> IoResult<()>
where
    E: Transcoder<Input = u32, Output = u16, Error = TranscodeEncodeError<PairEncodeError, u32>>,
{
    let mut mapper: fn(TranscodeEncodeError<PairEncodeError, u32>) -> Error = map_error;
    output.finish(encoder, &mut mapper)
}
