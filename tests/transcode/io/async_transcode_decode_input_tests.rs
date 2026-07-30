// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::{
    future::Future,
    io,
    pin::Pin,
    task::{
        Context,
        Poll,
        Waker,
    },
};

use qubit_codec::{
    AsyncTranscodeDecodeInput,
    AsyncTranscodeDecodeStep,
    TranscodeDecodeError,
    TranscodeProgress,
    Transcoder,
};
use qubit_io::AsyncInput;

/// Domain error type required by the transcode decoder contract.
#[derive(Debug, thiserror::Error)]
#[error("test decoder error")]
struct TestDecodeError;

/// Asynchronous input that yields small chunks and one pending state per read.
#[derive(Debug)]
struct ChunkedAsyncInput {
    bytes: Vec<u8>,
    position: usize,
    max_chunk: usize,
    pending: bool,
}

impl ChunkedAsyncInput {
    /// Creates an input over bytes with at most max_chunk bytes per read.
    fn new(bytes: Vec<u8>, max_chunk: usize) -> Self {
        Self {
            bytes,
            position: 0,
            max_chunk,
            pending: true,
        }
    }
}

impl AsyncInput for ChunkedAsyncInput {
    type Item = u8;

    /// Polls one bounded source read.
    unsafe fn poll_read_unchecked(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        output: &mut [u8],
        index: usize,
        count: usize,
    ) -> Poll<io::Result<usize>> {
        if self.pending {
            self.pending = false;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        self.pending = true;
        let available = self.bytes.len().saturating_sub(self.position);
        let read = available.min(count).min(self.max_chunk);
        output[index..index + read]
            .copy_from_slice(&self.bytes[self.position..self.position + read]);
        self.position += read;
        Poll::Ready(Ok(read))
    }
}

/// Polls a future to completion without requiring an asynchronous runtime.
fn complete<F>(future: F) -> F::Output
where
    F: Future,
{
    let mut context = Context::from_waker(Waker::noop());
    let mut future = std::pin::pin!(future);
    for _ in 0..128 {
        if let Poll::Ready(output) = future.as_mut().poll(&mut context) {
            return output;
        }
    }
    panic!("test future did not complete");
}

/// Polls a future once without requiring an asynchronous runtime.
fn poll_once<F>(future: Pin<&mut F>) -> Poll<F::Output>
where
    F: Future,
{
    let mut context = Context::from_waker(Waker::noop());
    future.poll(&mut context)
}

/// Decoder that emits one big-endian u16 for every pair of bytes.
#[derive(Debug, Default)]
struct PairDecoder;

impl Transcoder for PairDecoder {
    type Input = u8;
    type Output = u16;
    type Error = TranscodeDecodeError<TestDecodeError>;

    /// Reports the maximum output length for a supplied byte input.
    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, qubit_codec::CapacityError> {
        Ok(input_len / 2)
    }

    /// Resets this stateless decoder.
    fn reset(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }

    /// Decodes one pair when enough input and output are available.
    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let available_input = input.len() - input_index;
        let available_output = output.len() - output_index;
        if available_input < 2 {
            return Ok(TranscodeProgress::need_input(
                input_index,
                qubit_codec::nz(2),
                available_input,
                0,
                0,
            ));
        }
        if available_output == 0 {
            return Ok(TranscodeProgress::need_output(
                output_index,
                qubit_codec::nz(1),
                0,
                0,
                0,
            ));
        }
        output[output_index] =
            u16::from_be_bytes([input[input_index], input[input_index + 1]]);
        Ok(TranscodeProgress::complete(2, 1))
    }

    /// Finishes this stateless decoder without output.
    fn finish(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

/// Decoder that violates the complete-progress contract.
#[derive(Debug, Default)]
struct NoProgressDecoder;

impl Transcoder for NoProgressDecoder {
    type Input = u8;
    type Output = u16;
    type Error = TranscodeDecodeError<TestDecodeError>;

    /// Reports no output requirement.
    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, qubit_codec::CapacityError> {
        Ok(0)
    }

    /// Resets this stateless decoder.
    fn reset(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }

    /// Incorrectly claims completion without consuming visible input.
    fn transcode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Ok(TranscodeProgress::complete(0, 0))
    }

    /// Finishes this stateless decoder without output.
    fn finish(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

/// Verifies refilling across pending chunk boundaries before decoding.
#[test]
fn test_async_transcode_decode_input_refills_and_decodes() -> io::Result<()> {
    let mut input = AsyncTranscodeDecodeInput::with_capacity(
        ChunkedAsyncInput::new(vec![0x12, 0x34], 1),
        2,
    );
    let mut decoder = PairDecoder;
    let mut output = [0_u16; 1];
    let mut map_error =
        |error| io::Error::new(io::ErrorKind::InvalidData, error);

    let first = complete(input.transcode_async(
        &mut decoder,
        &mut map_error,
        &mut output,
        0,
        1,
    ))?;
    assert!(matches!(
        first,
        AsyncTranscodeDecodeStep::Progress(progress)
            if matches!(progress.status(), qubit_codec::TranscodeStatus::NeedInput { .. })
    ));
    assert!(complete(input.fill_until_async(2))?);
    let step = complete(input.transcode_async(
        &mut decoder,
        &mut map_error,
        &mut output,
        0,
        1,
    ))?;

    assert_eq!(
        AsyncTranscodeDecodeStep::Progress(TranscodeProgress::complete(2, 1)),
        step
    );
    assert_eq!([0x1234], output);
    assert_eq!(0, input.unread_len());
    Ok(())
}

/// Verifies EOF preserves an incomplete suffix for caller-defined policy.
#[test]
fn test_async_transcode_decode_input_preserves_incomplete_eof_suffix()
-> io::Result<()> {
    let mut input = AsyncTranscodeDecodeInput::with_capacity(
        ChunkedAsyncInput::new(vec![0x12], 1),
        2,
    );
    let mut decoder = PairDecoder;
    let mut output = [0_u16; 1];
    let mut map_error =
        |error| io::Error::new(io::ErrorKind::InvalidData, error);

    let step = complete(input.transcode_async(
        &mut decoder,
        &mut map_error,
        &mut output,
        0,
        1,
    ))?;

    assert_eq!(
        AsyncTranscodeDecodeStep::Progress(TranscodeProgress::need_input(
            0,
            qubit_codec::nz(2),
            1,
            0,
            0,
        )),
        step
    );
    assert_eq!([0x12], input.unread());
    Ok(())
}

/// Verifies a decoded value is returned before a later input poll can pend.
#[test]
fn test_async_transcode_decode_input_commits_progress_before_later_pending()
-> io::Result<()> {
    let mut source = ChunkedAsyncInput::new(vec![0x12, 0x34, 0x56], 2);
    source.pending = false;
    let mut input = AsyncTranscodeDecodeInput::with_capacity(source, 2);
    let mut decoder = PairDecoder;
    let mut output = [0_u16; 2];
    let mut map_error =
        |error| io::Error::new(io::ErrorKind::InvalidData, error);

    let mut future = Box::pin(input.transcode_async(
        &mut decoder,
        &mut map_error,
        &mut output,
        0,
        2,
    ));
    match poll_once(future.as_mut()) {
        Poll::Ready(Ok(AsyncTranscodeDecodeStep::Progress(progress))) => {
            assert_eq!(TranscodeProgress::complete(2, 1), progress);
        }
        other => panic!("expected committed decode progress, got {other:?}"),
    }
    drop(future);

    assert_eq!([0x1234, 0], output);
    assert_eq!(0, input.unread_len());
    Ok(())
}

/// Verifies invalid transcoder progress becomes an invalid-data I/O error.
#[test]
fn test_async_transcode_decode_input_rejects_invalid_progress() -> io::Result<()>
{
    let mut input = AsyncTranscodeDecodeInput::with_capacity(
        ChunkedAsyncInput::new(vec![0x12], 1),
        1,
    );
    let mut decoder = NoProgressDecoder;
    let mut output = [0_u16; 1];
    let mut map_error =
        |error| io::Error::new(io::ErrorKind::InvalidData, error);

    let error = complete(input.transcode_async(
        &mut decoder,
        &mut map_error,
        &mut output,
        0,
        1,
    ))
    .expect_err("invalid progress must be rejected");

    assert_eq!(io::ErrorKind::InvalidData, error.kind());
    Ok(())
}
