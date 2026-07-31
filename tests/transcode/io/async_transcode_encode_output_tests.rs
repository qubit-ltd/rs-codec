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
    AsyncTranscodeEncodeOutput,
    CapacityError,
    TranscodeEncodeError,
    TranscodeEncoder,
    TranscodeProgress,
    Transcoder,
};
use qubit_io::AsyncOutput;

#[derive(Debug)]
struct ChunkedAsyncOutput {
    bytes: Vec<u8>,
    max_chunk: usize,
    pending: bool,
    flushed: bool,
    failed: bool,
}

impl ChunkedAsyncOutput {
    /// Creates an output that accepts at most `max_chunk` bytes per poll.
    fn new(max_chunk: usize) -> Self {
        Self {
            bytes: Vec::new(),
            max_chunk,
            pending: true,
            flushed: false,
            failed: false,
        }
    }
}

impl AsyncOutput for ChunkedAsyncOutput {
    type Item = u8;

    unsafe fn poll_write_unchecked(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        input: &[u8],
        index: usize,
        count: usize,
    ) -> Poll<io::Result<usize>> {
        if self.failed {
            return Poll::Ready(Err(io::Error::other("output failure")));
        }
        if self.pending {
            self.pending = false;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        self.pending = true;
        let written = count.min(self.max_chunk);
        self.bytes.extend_from_slice(&input[index..index + written]);
        Poll::Ready(Ok(written))
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        self.flushed = true;
        Poll::Ready(Ok(()))
    }
}

/// Polls a future to completion without selecting an async runtime.
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

/// Polls a future once without selecting an asynchronous runtime.
fn poll_once<F>(future: Pin<&mut F>) -> Poll<F::Output>
where
    F: Future,
{
    let mut context = Context::from_waker(Waker::noop());
    future.poll(&mut context)
}

#[derive(Debug, Default)]
struct CopyEncoder;

impl Transcoder for CopyEncoder {
    type Input = char;
    type Output = u8;
    type Error = TranscodeEncodeError<(), char>;

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
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        output[output_index] = b'^';
        Ok(1)
    }

    fn transcode(
        &mut self,
        input: &[char],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let input = &input[input_index..];
        let output = &mut output[output_index..];
        let written = input.len().min(output.len());
        for (unit, value) in output.iter_mut().zip(input).take(written) {
            *unit = *value as u8;
        }
        if written == input.len() {
            Ok(TranscodeProgress::complete(written, written))
        } else {
            Ok(TranscodeProgress::need_output(
                output_index + written,
                qubit_codec::nz(1),
                output.len() - written,
                written,
                written,
            ))
        }
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }

    fn finish(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        output[output_index] = b'!';
        Ok(1)
    }
}

impl TranscodeEncoder for CopyEncoder {
    type EncodeError = ();
}

/// Encoder whose capacity query fails before writing output.
#[derive(Debug, Default)]
struct CapacityFailingEncoder {
    maximum: bool,
}

impl Transcoder for CapacityFailingEncoder {
    type Input = char;
    type Output = u8;
    type Error = TranscodeEncodeError<(), char>;

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        if self.maximum {
            Ok(usize::MAX)
        } else {
            Err(CapacityError::OutputLengthOverflow)
        }
    }

    fn reset(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn transcode(
        &mut self,
        _input: &[char],
        _input_index: usize,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        unreachable!("capacity failure prevents transcoding")
    }

    fn finish(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

impl TranscodeEncoder for CapacityFailingEncoder {
    type EncodeError = ();
}

#[test]
fn test_async_transcode_encode_output_preserves_lifecycle_output_across_pending()
-> io::Result<()> {
    let mut output = AsyncTranscodeEncodeOutput::with_capacity(
        ChunkedAsyncOutput::new(1),
        1,
    );
    let mut encoder = CopyEncoder;
    let mut map_error = |_| io::Error::other("copy encoder cannot fail");

    complete(output.reset_async(&mut encoder, &mut map_error))?;
    assert_eq!(
        TranscodeProgress::need_output(2, qubit_codec::nz(1), 0, 1, 1),
        complete(output.transcode_async(
            &mut encoder,
            &mut map_error,
            &['a', 'b'],
            0,
            2,
        ))?,
    );
    assert_eq!(
        TranscodeProgress::complete(1, 1),
        complete(output.transcode_async(
            &mut encoder,
            &mut map_error,
            &['a', 'b'],
            1,
            1,
        ))?,
    );
    complete(output.finish_async(&mut encoder, &mut map_error))?;
    complete(output.flush_async())?;

    let (inner, pending) = output.into_parts();
    assert!(pending.is_empty());
    assert_eq!(b"^ab!", inner.bytes.as_slice());
    assert!(inner.flushed);
    Ok(())
}

/// Verifies encoder progress is returned before later delivery can pend.
#[test]
fn test_async_transcode_encode_output_commits_progress_before_later_pending()
-> io::Result<()> {
    let mut output = AsyncTranscodeEncodeOutput::with_capacity(
        ChunkedAsyncOutput::new(1),
        1,
    );
    let mut encoder = CopyEncoder;
    let mut map_error = |_| io::Error::other("copy encoder cannot fail");
    let mut future = Box::pin(output.transcode_async(
        &mut encoder,
        &mut map_error,
        &['a', 'b'],
        0,
        2,
    ));

    match poll_once(future.as_mut()) {
        Poll::Ready(Ok(progress)) => {
            assert_eq!(
                TranscodeProgress::need_output(1, qubit_codec::nz(1), 0, 1, 1),
                progress,
            );
        }
        other => panic!("expected committed encode progress, got {other:?}"),
    }
    drop(future);

    assert_eq!(1, output.pending_len());
    Ok(())
}

/// Verifies constructors and draining expose buffered output state correctly.
#[test]
fn test_async_transcode_encode_output_exposes_buffer_operations()
-> io::Result<()> {
    let output = AsyncTranscodeEncodeOutput::try_with_capacity(
        ChunkedAsyncOutput::new(1),
        3,
    )?;
    assert!(output.capacity() >= 3);
    assert_eq!(0, output.pending_len());
    assert!(format!("{output:?}").contains("AsyncTranscodeEncodeOutput"));
    assert!(
        AsyncTranscodeEncodeOutput::try_with_capacity(
            ChunkedAsyncOutput::new(1),
            usize::MAX,
        )
        .is_err()
    );

    let mut output = AsyncTranscodeEncodeOutput::with_capacity(
        ChunkedAsyncOutput::new(1),
        1,
    );
    let mut encoder = CopyEncoder;
    let mut map_error = |_| io::Error::other("copy encoder cannot fail");
    complete(output.transcode_async(
        &mut encoder,
        &mut map_error,
        &['a'],
        0,
        1,
    ))?;
    assert_eq!(1, output.pending_len());
    complete(output.drain_async())?;
    assert_eq!(0, output.pending_len());
    assert_eq!(b"a", output.inner().bytes.as_slice());
    output.inner_mut().flushed = true;
    assert!(output.inner().flushed);
    Ok(())
}

/// Verifies zero-length encode operations and invalid source ranges.
#[test]
fn test_async_transcode_encode_output_validates_input_range() -> io::Result<()>
{
    let mut output =
        AsyncTranscodeEncodeOutput::new(ChunkedAsyncOutput::new(1));
    let mut encoder = CopyEncoder;
    let mut map_error = |_| io::Error::other("copy encoder cannot fail");

    assert_eq!(
        TranscodeProgress::complete(0, 0),
        complete(output.transcode_async(
            &mut encoder,
            &mut map_error,
            &['a'],
            0,
            0,
        ))?,
    );
    let error = complete(output.transcode_async(
        &mut encoder,
        &mut map_error,
        &['a'],
        1,
        1,
    ))
    .expect_err("invalid input range must fail");
    assert_eq!(io::ErrorKind::InvalidInput, error.kind());
    Ok(())
}

/// Verifies capacity planning failures cross the asynchronous I/O boundary.
#[test]
fn test_async_transcode_encode_output_maps_capacity_errors() -> io::Result<()> {
    let mut output =
        AsyncTranscodeEncodeOutput::new(ChunkedAsyncOutput::new(1));
    let mut encoder = CapacityFailingEncoder::default();
    let mut map_error =
        |_| io::Error::other("encoder cannot reach domain failure");

    let error = complete(output.transcode_async(
        &mut encoder,
        &mut map_error,
        &['a'],
        0,
        1,
    ))
    .expect_err("capacity failure must cross the I/O boundary");
    assert_eq!(io::ErrorKind::InvalidData, error.kind());
    encoder.maximum = true;
    let error = complete(output.transcode_async(
        &mut encoder,
        &mut map_error,
        &['a'],
        0,
        1,
    ))
    .expect_err("impossible spare capacity must fail");
    assert_eq!(io::ErrorKind::OutOfMemory, error.kind());
    Ok(())
}

/// Verifies asynchronous output delivery failures remain visible to callers.
#[test]
fn test_async_transcode_encode_output_propagates_delivery_errors()
-> io::Result<()> {
    let mut output = AsyncTranscodeEncodeOutput::with_capacity(
        ChunkedAsyncOutput::new(1),
        1,
    );
    let mut encoder = CopyEncoder;
    let mut map_error = |_| io::Error::other("copy encoder cannot fail");

    complete(output.transcode_async(
        &mut encoder,
        &mut map_error,
        &['a'],
        0,
        1,
    ))?;
    output.inner_mut().failed = true;

    let error = complete(output.drain_async())
        .expect_err("delivery error must be preserved");
    assert_eq!(io::ErrorKind::Other, error.kind());
    Ok(())
}
