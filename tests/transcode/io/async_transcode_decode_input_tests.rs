// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

use qubit_codec as codec;
use qubit_codec::AsyncTranscodeDecodeInput;
use qubit_codec::AsyncTranscodeDecodeStep;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeProgress;
use qubit_codec::Transcoder;
use qubit_io::AsyncInput;
use qubit_utils as utils_crate;

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

/// Input that fails every attempted read.
#[derive(Debug, Default)]
struct FailingAsyncInput;

impl AsyncInput for FailingAsyncInput {
    type Item = u8;

    unsafe fn poll_read_unchecked(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _output: &mut [u8],
        _index: usize,
        _count: usize,
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(Err(io::Error::other("input failure")))
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
    ) -> Result<usize, codec::CapacityError> {
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
                utils_crate::nonzero(2),
                0,
                0,
            ));
        }
        if available_output == 0 {
            return Ok(TranscodeProgress::need_output(
                utils_crate::nonzero(1),
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

#[derive(Debug, Default)]
struct EofTailDecoder;

impl Transcoder for EofTailDecoder {
    type Input = u8;
    type Output = u16;
    type Error = TranscodeDecodeError<TestDecodeError>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, codec::CapacityError> {
        Ok(input_len)
    }

    fn reset(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn transcode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Ok(TranscodeProgress::need_input(utils_crate::nonzero(2), 0, 0))
    }

    fn transcode_eof(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        output[output_index] = u16::from(input[input_index]);
        Ok(TranscodeProgress::complete(1, 1))
    }

    fn finish(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

/// Decoder with observable lifecycle output.
#[derive(Debug, Default)]
struct LifecycleDecoder;

impl Transcoder for LifecycleDecoder {
    type Input = u8;
    type Output = u16;
    type Error = TranscodeDecodeError<TestDecodeError>;

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, codec::CapacityError> {
        Ok(0)
    }

    fn max_reset_output_len(&self) -> Result<usize, codec::CapacityError> {
        Ok(1)
    }

    fn reset(
        &mut self,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        output[output_index] = 0xaaaa;
        Ok(1)
    }

    fn transcode(
        &mut self,
        _input: &[u8],
        input_index: usize,
        _output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Ok(TranscodeProgress::complete(input_index, output_index))
    }

    fn max_finish_output_len(&self) -> Result<usize, codec::CapacityError> {
        Ok(1)
    }

    fn finish(
        &mut self,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        output[output_index] = 0xbbbb;
        Ok(1)
    }
}

/// Decoder whose lifecycle capacity query fails.
#[derive(Debug, Default)]
struct CapacityFailingDecoder;

impl Transcoder for CapacityFailingDecoder {
    type Input = u8;
    type Output = u16;
    type Error = TranscodeDecodeError<TestDecodeError>;

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, codec::CapacityError> {
        Ok(0)
    }

    fn max_reset_output_len(&self) -> Result<usize, codec::CapacityError> {
        Err(codec::CapacityError::OutputLengthOverflow)
    }

    fn reset(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        unreachable!("capacity failure prevents reset")
    }

    fn transcode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn max_finish_output_len(&self) -> Result<usize, codec::CapacityError> {
        Err(codec::CapacityError::OutputLengthOverflow)
    }

    fn finish(
        &mut self,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        unreachable!("capacity failure prevents finish")
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
    ) -> Result<usize, codec::CapacityError> {
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
            if matches!(progress.status(), codec::TranscodeStatus::NeedInput { .. })
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
            utils_crate::nonzero(2),
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

/// Verifies buffered-input operations preserve the unread window.
#[test]
fn test_async_transcode_decode_input_exposes_buffer_operations()
-> io::Result<()> {
    let mut input = AsyncTranscodeDecodeInput::new(ChunkedAsyncInput::new(
        vec![0x12, 0x34],
        2,
    ));

    assert!(AsyncInput::is_buffered(&input));
    assert_eq!(0, input.unread_len());
    assert!(complete(input.fill_more_async())?);
    assert_eq!([0x12, 0x34], input.unread());

    let mut copied = [0_u8; 3];
    // SAFETY: The destination range and unread count both fit the buffers.
    unsafe {
        input.copy_unread_to(&mut copied, 1, 2);
    }
    assert_eq!([0, 0x12, 0x34], copied);
    input.consume(1);
    assert_eq!([0x34], input.unread());

    input.inner_mut().max_chunk = 1;
    assert_eq!(1, input.inner().max_chunk);
    let (inner, unread) = input.into_parts();
    assert_eq!(2, inner.position);
    assert_eq!([0x34], unread.readable());
    Ok(())
}

/// Verifies constructor capacity requests and EOF refill behavior.
#[test]
fn test_async_transcode_decode_input_capacity_and_eof() -> io::Result<()> {
    let input = AsyncTranscodeDecodeInput::try_with_capacity(
        ChunkedAsyncInput::new(Vec::new(), 1),
        3,
    )?;
    assert!(input.capacity() >= 3);
    assert!(format!("{input:?}").contains("AsyncTranscodeDecodeInput"));

    assert!(
        AsyncTranscodeDecodeInput::try_with_capacity(
            ChunkedAsyncInput::new(Vec::new(), 1),
            usize::MAX,
        )
        .is_err()
    );

    let mut input = AsyncTranscodeDecodeInput::with_capacity(
        ChunkedAsyncInput::new(vec![0x12], 1),
        1,
    );
    assert!(!complete(input.fill_until_async(2))?);
    assert_eq!([0x12], input.unread());
    assert!(!complete(input.fill_more_async())?);
    Ok(())
}

/// Verifies zero-length decode operations and invalid destination ranges.
#[test]
fn test_async_transcode_decode_input_validates_output_range() -> io::Result<()>
{
    let mut input =
        AsyncTranscodeDecodeInput::new(ChunkedAsyncInput::new(Vec::new(), 1));
    let mut decoder = PairDecoder;
    let mut map_error =
        |error| io::Error::new(io::ErrorKind::InvalidData, error);
    let mut output = [0_u16; 1];

    assert_eq!(
        AsyncTranscodeDecodeStep::Progress(TranscodeProgress::complete(0, 0)),
        complete(input.transcode_async(
            &mut decoder,
            &mut map_error,
            &mut output,
            0,
            0,
        ))?,
    );
    let error = complete(input.transcode_async(
        &mut decoder,
        &mut map_error,
        &mut output,
        1,
        1,
    ))
    .expect_err("invalid output range must fail");
    assert_eq!(io::ErrorKind::InvalidInput, error.kind());
    Ok(())
}

/// Verifies lifecycle output uses the caller's indexed destination range.
#[test]
fn test_async_transcode_decode_input_runs_decoder_lifecycle() -> io::Result<()>
{
    let input =
        AsyncTranscodeDecodeInput::new(ChunkedAsyncInput::new(Vec::new(), 1));
    let mut decoder = LifecycleDecoder;
    let mut map_error =
        |error| io::Error::new(io::ErrorKind::InvalidData, error);
    let mut output = [0_u16; 3];

    assert_eq!(
        1,
        input.reset(&mut decoder, &mut map_error, &mut output, 1, 1)?
    );
    assert_eq!(
        1,
        input.finish(&mut decoder, &mut map_error, &mut output, 2, 1)?
    );
    assert_eq!([0, 0xaaaa, 0xbbbb], output);
    let error = input
        .reset(&mut decoder, &mut map_error, &mut output, 0, 0)
        .expect_err("short reset output must fail");
    assert_eq!(io::ErrorKind::InvalidData, error.kind());
    let error = input
        .reset(&mut decoder, &mut map_error, &mut output, 3, 1)
        .expect_err("invalid reset output range must fail");
    assert_eq!(io::ErrorKind::InvalidInput, error.kind());
    let error = input
        .finish(&mut decoder, &mut map_error, &mut output, 0, 0)
        .expect_err("short finish output must fail");
    assert_eq!(io::ErrorKind::InvalidData, error.kind());
    let error = input
        .finish(&mut decoder, &mut map_error, &mut output, 3, 1)
        .expect_err("invalid finish output range must fail");
    assert_eq!(io::ErrorKind::InvalidInput, error.kind());
    Ok(())
}

/// Verifies async input reads and EOF steps delegate through the wrapper.
#[test]
fn test_async_transcode_decode_input_reads_and_reports_eof() -> io::Result<()> {
    let mut input =
        AsyncTranscodeDecodeInput::new(ChunkedAsyncInput::new(vec![0x12], 1));
    let mut units = [0_u8; 1];
    assert_eq!(1, complete(input.read_async(&mut units))?);
    assert_eq!([0x12], units);

    let mut decoder = PairDecoder;
    let mut map_error =
        |error| io::Error::new(io::ErrorKind::InvalidData, error);
    let mut output = [0_u16; 1];
    assert_eq!(
        AsyncTranscodeDecodeStep::EndOfInput,
        complete(input.transcode_async(
            &mut decoder,
            &mut map_error,
            &mut output,
            0,
            1,
        ))?,
    );
    Ok(())
}

#[test]
fn test_async_transcode_decode_input_transcode_eof_step_consumes_buffered_tail()
-> io::Result<()> {
    let mut input = AsyncTranscodeDecodeInput::with_capacity(
        ChunkedAsyncInput::new(vec![0x5a], 1),
        1,
    );
    assert!(complete(input.fill_more_async())?);
    let mut decoder = EofTailDecoder;
    let mut map_error =
        |error| io::Error::new(io::ErrorKind::InvalidData, error);
    let mut output = [0_u16; 1];

    let progress = input.transcode_eof_step(
        &mut decoder,
        &mut map_error,
        &mut output,
        0,
        1,
    )?;

    assert_eq!(TranscodeProgress::complete(1, 1), progress);
    assert_eq!([0x5a], output);
    assert_eq!(0, input.unread_len());

    assert_eq!(
        TranscodeProgress::complete(0, 0),
        input.transcode_eof_step(
            &mut decoder,
            &mut map_error,
            &mut output,
            0,
            0,
        )?,
    );
    assert_eq!(
        TranscodeProgress::complete(0, 0),
        input.transcode_eof_step(
            &mut decoder,
            &mut map_error,
            &mut output,
            0,
            1,
        )?,
    );
    let error = input
        .transcode_eof_step(&mut decoder, &mut map_error, &mut output, 1, 1)
        .expect_err("invalid EOF output range must fail");
    assert_eq!(io::ErrorKind::InvalidInput, error.kind());
    Ok(())
}

/// Verifies decoder capacity failures become invalid-data I/O errors.
#[test]
fn test_async_transcode_decode_input_maps_lifecycle_capacity_errors()
-> io::Result<()> {
    let input =
        AsyncTranscodeDecodeInput::new(ChunkedAsyncInput::new(Vec::new(), 1));
    let mut decoder = CapacityFailingDecoder;
    let mut map_error =
        |error| io::Error::new(io::ErrorKind::InvalidData, error);
    let mut output = [0_u16; 1];

    let reset = input
        .reset(&mut decoder, &mut map_error, &mut output, 0, 1)
        .expect_err("capacity failure must be mapped");
    assert_eq!(io::ErrorKind::InvalidData, reset.kind());
    let finish = input
        .finish(&mut decoder, &mut map_error, &mut output, 0, 1)
        .expect_err("capacity failure must be mapped");
    assert_eq!(io::ErrorKind::InvalidData, finish.kind());
    Ok(())
}

/// Verifies refill failures and impossible capacity requests retain I/O errors.
#[test]
fn test_async_transcode_decode_input_maps_refill_errors() -> io::Result<()> {
    let mut input = AsyncTranscodeDecodeInput::new(FailingAsyncInput);
    let error = complete(input.fill_more_async())
        .expect_err("input failure must be preserved");
    assert_eq!(io::ErrorKind::Other, error.kind());

    let mut input = AsyncTranscodeDecodeInput::new(FailingAsyncInput);
    let mut decoder = PairDecoder;
    let mut output = [0_u16; 1];
    let error = complete(input.transcode_async(
        &mut decoder,
        &mut |error| io::Error::new(io::ErrorKind::InvalidData, error),
        &mut output,
        0,
        1,
    ))
    .expect_err("decode refill failure must be preserved");
    assert_eq!(io::ErrorKind::Other, error.kind());

    let mut input =
        AsyncTranscodeDecodeInput::new(ChunkedAsyncInput::new(Vec::new(), 1));
    let error = complete(input.fill_until_async(usize::MAX))
        .expect_err("impossible capacity must fail");
    assert_eq!(io::ErrorKind::OutOfMemory, error.kind());
    Ok(())
}
