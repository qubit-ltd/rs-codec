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
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::Ordering;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

use qubit_codec::AsyncTranscodeEncodeOutput;
use qubit_codec::CapacityError;
use qubit_codec::TranscodeEncodeError;
use qubit_codec::TranscodeEncoder;
use qubit_codec::TranscodeFailure;
use qubit_codec::TranscodeProgress;
use qubit_codec::Transcoder;
use qubit_io::AsyncOutput;
use qubit_utils as utils_crate;
use thiserror::Error as ThisError;

#[derive(Debug)]
struct ChunkedAsyncOutput {
    bytes: Vec<u8>,
    max_chunk: usize,
    pending: bool,
    flushed: bool,
    failed: bool,
    flush_pending: bool,
    flush_fail_once: bool,
    flush_calls: usize,
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
            flush_pending: false,
            flush_fail_once: false,
            flush_calls: 0,
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

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.flush_calls += 1;
        if self.flush_pending {
            self.flush_pending = false;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        if self.flush_fail_once {
            self.flush_fail_once = false;
            return Poll::Ready(Err(io::Error::other("flush fixture failure")));
        }
        self.flushed = true;
        Poll::Ready(Ok(()))
    }
}

/// Polls a future to completion without selecting an async runtime.
fn complete<F>(future: F) -> F::Output
where
    F: Future,
{
    super::internal::complete_when_woken(future)
}

/// Polls a future once without selecting an asynchronous runtime.
fn poll_once<F>(future: Pin<&mut F>) -> Poll<F::Output>
where
    F: Future,
{
    let mut context = Context::from_waker(Waker::noop());
    future.poll(&mut context)
}

#[derive(Default)]
struct AtomicTailEncoder {
    fail_reset: bool,
}

impl Transcoder for AtomicTailEncoder {
    type Input = u8;
    type Output = u8;
    type Error = TranscodeEncodeError<(), u8>;

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        if self.fail_reset {
            return Err(TranscodeEncodeError::domain_reset(()));
        }
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        TranscodeFailure::ensure_transcode_indices(input.len(), input_index, output.len(), output_index)?;
        let source = &input[input_index..];
        let target = &mut output[output_index..];
        if source.is_empty() {
            return Ok(TranscodeProgress::complete(0, 0));
        }
        if target.is_empty() {
            return Ok(TranscodeProgress::need_output(utils_crate::nonzero(1), 0, 0));
        }
        target[0] = source[0];
        let remaining = source.len() - 1;
        if target.len() - 1 < remaining {
            return Ok(TranscodeProgress::need_output(utils_crate::nonzero(remaining), 1, 1));
        }
        target[1..source.len()].copy_from_slice(&source[1..]);
        Ok(TranscodeProgress::complete(source.len(), source.len()))
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }
}

impl TranscodeEncoder for AtomicTailEncoder {
    type EncodeError = ();
}

#[test]
fn test_successful_reset_clears_previous_output_requirement() -> io::Result<()> {
    for fail_reset_first in [false, true] {
        let mut output = AsyncTranscodeEncodeOutput::with_capacity(ChunkedAsyncOutput::new(8), 8);
        let mut encoder = AtomicTailEncoder::default();
        let mut mapper = |_| io::Error::other("fixture encoder error");
        complete(output.reset_async(&mut encoder, &mut mapper))?;
        let input = [7_u8; 17];
        let progress = complete(output.transcode_async(&mut encoder, &mut mapper, &input, 0, 17))?;
        assert_eq!(TranscodeProgress::need_output(utils_crate::nonzero(16), 1, 1), progress);
        assert_eq!(8, output.capacity());
        assert_eq!(1, output.pending_len());
        output.inner_mut().failed = true;
        encoder.fail_reset = fail_reset_first;
        if fail_reset_first {
            assert!(complete(output.reset_async(&mut encoder, &mut mapper)).is_err());
            assert_eq!(8, output.capacity());
            assert_eq!(1, output.pending_len());
            encoder.fail_reset = false;
        }
        complete(output.reset_async(&mut encoder, &mut mapper))?;
        let progress = complete(output.transcode_async(&mut encoder, &mut mapper, &[9], 0, 1))?;
        assert_eq!(TranscodeProgress::complete(1, 1), progress);
        assert_eq!(
            8,
            output.capacity(),
            "successful reset must discard the old capacity hint"
        );
        assert_eq!(2, output.pending_len());
        assert!(
            output.inner().bytes.is_empty(),
            "new input must not force pending delivery"
        );
        output.inner_mut().failed = false;
        complete(output.flush_async())?;
        assert_eq!([7, 9], output.inner().bytes.as_slice());
    }
    Ok(())
}

#[test]
fn test_failed_reset_keeps_previous_output_requirement() -> io::Result<()> {
    let mut output = AsyncTranscodeEncodeOutput::with_capacity(ChunkedAsyncOutput::new(8), 8);
    let mut encoder = AtomicTailEncoder::default();
    let mut mapper = |_| io::Error::other("fixture encoder error");
    complete(output.reset_async(&mut encoder, &mut mapper))?;
    let input = [7_u8; 17];
    let first = complete(output.transcode_async(&mut encoder, &mut mapper, &input, 0, 17))?;
    assert_eq!(TranscodeProgress::need_output(utils_crate::nonzero(16), 1, 1), first);
    encoder.fail_reset = true;
    assert!(complete(output.reset_async(&mut encoder, &mut mapper)).is_err());
    output.inner_mut().failed = true;
    let error = complete(output.transcode_async(&mut encoder, &mut mapper, &input, 1, 16))
        .expect_err("failed reset must retain the old retry requirement");
    assert_eq!(io::ErrorKind::Other, error.kind());
    assert_eq!(1, output.pending_len());
    output.inner_mut().failed = false;
    let resumed = complete(output.transcode_async(&mut encoder, &mut mapper, &input, 1, 16))?;
    assert_eq!(TranscodeProgress::complete(16, 16), resumed);
    complete(output.flush_async())?;
    assert_eq!(input.as_slice(), output.inner().bytes.as_slice());
    Ok(())
}

#[derive(Debug, Default)]
struct CopyEncoder;

impl Transcoder for CopyEncoder {
    type Input = char;
    type Output = u8;
    type Error = TranscodeEncodeError<(), char>;

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
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
                utils_crate::nonzero(1),
                written,
                written,
            ))
        }
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        output[output_index] = b'!';
        Ok(1)
    }
}

impl TranscodeEncoder for CopyEncoder {
    type EncodeError = ();
}

#[derive(Debug, Default)]
struct CountingCopyEncoder {
    transcode_calls: usize,
    reset_calls: usize,
    finish_calls: usize,
    lifecycle_bound: usize,
}

impl Transcoder for CountingCopyEncoder {
    type Input = char;
    type Output = u8;
    type Error = TranscodeEncodeError<(), char>;

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        Ok(self.lifecycle_bound.max(1))
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        self.reset_calls += 1;
        CopyEncoder.reset(output, output_index)
    }

    fn transcode(
        &mut self,
        input: &[char],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        self.transcode_calls += 1;
        CopyEncoder.transcode(input, input_index, output, output_index)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(self.lifecycle_bound.max(1))
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        self.finish_calls += 1;
        CopyEncoder.finish(output, output_index)
    }
}

impl TranscodeEncoder for CountingCopyEncoder {
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

    fn max_transcode_output_len(&self, _input_len: usize) -> Result<usize, CapacityError> {
        if self.maximum {
            Ok(usize::MAX)
        } else {
            Err(CapacityError::OutputLengthOverflow)
        }
    }

    fn reset(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
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

    fn finish(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

impl TranscodeEncoder for CapacityFailingEncoder {
    type EncodeError = ();
}

#[derive(Debug, Default)]
struct FailingEncoder;

impl Transcoder for FailingEncoder {
    type Input = char;
    type Output = u8;
    type Error = TranscodeEncodeError<(), char>;

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn reset(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn transcode(
        &mut self,
        _input: &[char],
        _input_index: usize,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Err(TranscodeEncodeError::domain_main((), 0))
    }

    fn finish(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

impl TranscodeEncoder for FailingEncoder {
    type EncodeError = ();
}

#[derive(Debug, Default)]
struct NeedInputEncoder;

impl Transcoder for NeedInputEncoder {
    type Input = char;
    type Output = u8;
    type Error = TranscodeEncodeError<(), char>;

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn reset(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn transcode(
        &mut self,
        _input: &[char],
        _input_index: usize,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Ok(TranscodeProgress::need_input(utils_crate::nonzero(2), 0, 0))
    }

    fn finish(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

impl TranscodeEncoder for NeedInputEncoder {
    type EncodeError = ();
}

#[derive(Debug, Default)]
struct LifecycleFailingEncoder {
    fail_reset: bool,
    fail_finish: bool,
    huge_reset_bound: bool,
    huge_finish_bound: bool,
    overflow_reset_bound: bool,
    overflow_finish_bound: bool,
}

impl Transcoder for LifecycleFailingEncoder {
    type Input = char;
    type Output = u8;
    type Error = TranscodeEncodeError<(), char>;

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        if self.overflow_reset_bound {
            Err(CapacityError::OutputLengthOverflow)
        } else if self.huge_reset_bound {
            Ok(usize::MAX)
        } else {
            Ok(1)
        }
    }

    fn reset(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        if self.fail_reset {
            Err(TranscodeEncodeError::domain_reset(()))
        } else {
            Ok(0)
        }
    }

    fn transcode(
        &mut self,
        _input: &[char],
        _input_index: usize,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        if self.overflow_finish_bound {
            Err(CapacityError::OutputLengthOverflow)
        } else if self.huge_finish_bound {
            Ok(usize::MAX)
        } else {
            Ok(1)
        }
    }

    fn finish(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        if self.fail_finish {
            Err(TranscodeEncodeError::domain_finish(()))
        } else {
            Ok(0)
        }
    }
}

impl TranscodeEncoder for LifecycleFailingEncoder {
    type EncodeError = ();
}

#[derive(Debug, Eq, PartialEq, ThisError)]
#[error("bad async boundary input")]
struct AsyncBoundaryError;

#[derive(Debug, Default)]
struct BoundaryEncoder {
    encoded: Vec<u8>,
}

impl Transcoder for BoundaryEncoder {
    type Input = u8;
    type Output = u8;
    type Error = TranscodeEncodeError<AsyncBoundaryError, u8>;

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn reset(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        self.encoded.clear();
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let mut read = 0;
        while input_index + read < input.len() {
            let value = input[input_index + read];
            if value == u8::MAX {
                return Err(TranscodeEncodeError::domain_main(
                    AsyncBoundaryError,
                    input_index + read,
                ));
            }
            output[output_index + read] = value;
            self.encoded.push(value);
            read += 1;
        }
        Ok(TranscodeProgress::complete(read, read))
    }

    fn finish(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

impl TranscodeEncoder for BoundaryEncoder {
    type EncodeError = AsyncBoundaryError;
}

#[test]
fn test_encode_error_commit_boundary_async_poll() -> io::Result<()> {
    let mut output = AsyncTranscodeEncodeOutput::with_capacity(ChunkedAsyncOutput::new(8), 8);
    let mut encoder = BoundaryEncoder::default();
    let mut map_error = |error: TranscodeEncodeError<AsyncBoundaryError, u8>| io::Error::other(error);
    assert_eq!(
        TranscodeProgress::complete(1, 1),
        complete(output.transcode_async(&mut encoder, &mut map_error, &[0x11], 0, 1))?,
        "the first poll commits one input and output unit"
    );
    assert_eq!(1, output.pending_len(), "the first encoded unit is pending delivery");

    let mut context = Context::from_waker(Waker::noop());
    match Pin::new(&mut output).poll_transcode(&mut context, &mut encoder, &mut map_error, &[0x22, u8::MAX], 0, 2) {
        Poll::Ready(Err(error)) => {
            assert_eq!(
                io::ErrorKind::Other,
                error.kind(),
                "domain errors map to io::ErrorKind::Other"
            );
            assert!(
                error
                    .get_ref()
                    .and_then(|source| { source.downcast_ref::<TranscodeEncodeError<AsyncBoundaryError, u8>>() })
                    .and_then(TranscodeEncodeError::domain_ref)
                    == Some(&AsyncBoundaryError)
            );
        }
        other => panic!("expected mapped encoder error, got {other:?}"),
    }
    assert_eq!(
        &[0x11, 0x22],
        encoder.encoded.as_slice(),
        "the failing call commits its valid prefix"
    );
    assert_eq!(
        1,
        output.pending_len(),
        "the earlier pending byte remains queued after the error"
    );
    complete(output.drain_async())?;
    assert_eq!(
        &[0x11],
        output.inner().bytes.as_slice(),
        "draining delivers only the queued prefix"
    );
    Ok(())
}

#[test]
fn test_async_transcode_encode_output_preserves_lifecycle_output_across_pending() -> io::Result<()> {
    let mut output = AsyncTranscodeEncodeOutput::with_capacity(ChunkedAsyncOutput::new(1), 1);
    let mut encoder = CopyEncoder;
    let mut map_error = |_| io::Error::other("copy encoder cannot fail");

    complete(output.reset_async(&mut encoder, &mut map_error))?;
    assert_eq!(
        TranscodeProgress::need_output(utils_crate::nonzero(1), 1, 1),
        complete(output.transcode_async(&mut encoder, &mut map_error, &['a', 'b'], 0, 2,))?,
        "the first call stops after filling the output buffer"
    );
    assert_eq!(
        TranscodeProgress::complete(1, 1),
        complete(output.transcode_async(&mut encoder, &mut map_error, &['a', 'b'], 1, 1,))?,
        "the resumed call encodes the remaining input"
    );
    complete(output.finish_async(&mut encoder, &mut map_error))?;
    complete(output.flush_async())?;

    let (inner, pending) = output.into_parts();
    assert!(pending.is_empty());
    assert_eq!(
        b"^ab!",
        inner.bytes.as_slice(),
        "reset, input, and finish bytes are delivered in order"
    );
    assert!(inner.flushed);
    Ok(())
}

#[test]
fn test_async_transcode_wake_drain_completes_after_pending() {
    let mut output = AsyncTranscodeEncodeOutput::with_capacity(ChunkedAsyncOutput::new(8), 8);
    let mut encoder = CopyEncoder;
    let mut mapper = |_| io::Error::other("copy encoder cannot fail");
    complete(output.reset_async(&mut encoder, &mut mapper)).expect("reset buffers output");
    complete(output.drain_async()).expect("drain completes");
    assert_eq!(b"^", output.inner().bytes.as_slice());
}

#[test]
fn test_async_transcode_wake_flush_completes_after_pending() {
    let mut output = AsyncTranscodeEncodeOutput::new(ChunkedAsyncOutput::new(8));
    output.inner_mut().flush_pending = true;
    complete(output.flush_async()).expect("flush completes");
    assert!(output.inner().flushed);
    assert_eq!(2, output.inner().flush_calls);
}

#[derive(Debug, Default)]
struct OutputWakeGate {
    ready: bool,
    waker: Option<Waker>,
}

#[derive(Debug, Clone, Default)]
struct LatestWakerOutput {
    gate: Arc<Mutex<OutputWakeGate>>,
    bytes: Vec<u8>,
}

impl LatestWakerOutput {
    fn release(&self) {
        let waker = {
            let mut gate = self.gate.lock().expect("wake gate lock");
            gate.ready = true;
            gate.waker.take()
        };
        waker.expect("pending poll registered a waker").wake();
    }
}

impl AsyncOutput for LatestWakerOutput {
    type Item = u8;

    unsafe fn poll_write_unchecked(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        input: &[u8],
        index: usize,
        count: usize,
    ) -> Poll<io::Result<usize>> {
        let ready = {
            let mut gate = self.gate.lock().expect("wake gate lock");
            if !gate.ready {
                gate.waker = Some(cx.waker().clone());
                return Poll::Pending;
            }
            true
        };
        if ready {
            self.bytes.extend_from_slice(&input[index..index + count]);
        }
        Poll::Ready(Ok(count))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let mut gate = self.gate.lock().expect("wake gate lock");
        if !gate.ready {
            gate.waker = Some(cx.waker().clone());
            return Poll::Pending;
        }
        Poll::Ready(Ok(()))
    }
}

fn latest_contexts() -> (
    Arc<super::internal::WakeCount>,
    Arc<super::internal::WakeCount>,
    Waker,
    Waker,
) {
    let a = Arc::new(super::internal::WakeCount::default());
    let b = Arc::new(super::internal::WakeCount::default());
    let wa = Waker::from(a.clone());
    let wb = Waker::from(b.clone());
    (a, b, wa, wb)
}

#[test]
fn test_async_transcode_wake_latest_drain_waker() {
    let sink = LatestWakerOutput::default();
    let mut output = AsyncTranscodeEncodeOutput::with_capacity(sink.clone(), 1);
    let mut encoder = CopyEncoder;
    let mut mapper = |_| io::Error::other("copy encoder cannot fail");
    complete(output.reset_async(&mut encoder, &mut mapper)).expect("reset buffers output");
    let (a, b, wa, wb) = latest_contexts();
    let mut ca = Context::from_waker(&wa);
    let mut cb = Context::from_waker(&wb);
    let mut future = Box::pin(output.drain_async());
    assert!(matches!(future.as_mut().poll(&mut ca), Poll::Pending));
    assert!(matches!(future.as_mut().poll(&mut cb), Poll::Pending));
    sink.release();
    assert_eq!(0, a.0.load(Ordering::SeqCst));
    assert_eq!(1, b.0.load(Ordering::SeqCst));
    assert!(matches!(future.as_mut().poll(&mut cb), Poll::Ready(Ok(()))));
}

#[test]
fn test_async_transcode_wake_latest_flush_waker() {
    let sink = LatestWakerOutput::default();
    let mut output = AsyncTranscodeEncodeOutput::new(sink.clone());
    let (a, b, wa, wb) = latest_contexts();
    let mut ca = Context::from_waker(&wa);
    let mut cb = Context::from_waker(&wb);
    let mut future = Box::pin(output.flush_async());
    assert!(matches!(future.as_mut().poll(&mut ca), Poll::Pending));
    assert!(matches!(future.as_mut().poll(&mut cb), Poll::Pending));
    sink.release();
    assert_eq!(0, a.0.load(Ordering::SeqCst));
    assert_eq!(1, b.0.load(Ordering::SeqCst));
    assert!(matches!(future.as_mut().poll(&mut cb), Poll::Ready(Ok(()))));
}

/// Verifies encoder progress is returned before later delivery can pend.
#[test]
fn test_async_transcode_encode_output_commits_progress_before_later_pending() -> io::Result<()> {
    let mut output = AsyncTranscodeEncodeOutput::with_capacity(ChunkedAsyncOutput::new(1), 1);
    let mut encoder = CopyEncoder;
    let mut map_error = |_| io::Error::other("copy encoder cannot fail");
    let mut future = Box::pin(output.transcode_async(&mut encoder, &mut map_error, &['a', 'b'], 0, 2));

    match poll_once(future.as_mut()) {
        Poll::Ready(Ok(progress)) => {
            assert_eq!(
                TranscodeProgress::need_output(utils_crate::nonzero(1), 1, 1),
                progress,
                "committed progress is returned before pending output delivery",
            );
        }
        other => panic!("expected committed encode progress, got {other:?}"),
    }
    drop(future);

    assert_eq!(
        1,
        output.pending_len(),
        "the committed byte remains buffered after cancellation"
    );
    Ok(())
}

/// Verifies pending output blocks the next encoder call and survives
/// cancellation.
#[test]
fn test_async_transcode_encode_output_applies_backpressure_before_next_step() -> io::Result<()> {
    let mut output = AsyncTranscodeEncodeOutput::with_capacity(ChunkedAsyncOutput::new(1), 2);
    let mut encoder = CountingCopyEncoder::default();
    let mut map_error = |_| io::Error::other("copy encoder cannot fail");
    assert_eq!(
        TranscodeProgress::complete(2, 2),
        complete(output.transcode_async(&mut encoder, &mut map_error, &['a', 'b'], 0, 2))?,
        "both source characters are committed before draining"
    );
    assert_eq!(2, output.pending_len(), "both encoded bytes wait in the pending buffer");

    output.inner_mut().pending = false;
    let calls_before_pending = encoder.transcode_calls;
    let mut future = Box::pin(output.transcode_async(&mut encoder, &mut map_error, &['c'], 0, 1));
    let is_pending = matches!(poll_once(future.as_mut()), Poll::Pending);
    drop(future);
    assert!(is_pending);
    assert_eq!(
        calls_before_pending, encoder.transcode_calls,
        "backpressure prevents another encoder call"
    );
    assert_eq!(
        b"a",
        output.inner().bytes.as_slice(),
        "only the first pending byte is delivered"
    );
    assert_eq!(
        1,
        output.pending_len(),
        "the second byte remains pending after cancellation"
    );

    assert_eq!(
        TranscodeProgress::complete(1, 1),
        complete(output.transcode_async(&mut encoder, &mut map_error, &['c'], 0, 1))?,
        "the next input is encoded after pending output drains"
    );
    complete(output.drain_async())?;
    let (inner, pending) = output.into_parts();
    assert!(pending.is_empty());
    assert_eq!(b"abc", inner.bytes.as_slice(), "drained output preserves input order");
    Ok(())
}

/// Verifies constructors and draining expose buffered output state correctly.
#[test]
fn test_async_transcode_encode_output_exposes_buffer_operations() -> io::Result<()> {
    let output = AsyncTranscodeEncodeOutput::try_with_capacity(ChunkedAsyncOutput::new(1), 3)?;
    assert!(output.capacity() >= 3);
    assert_eq!(0, output.pending_len(), "a new output buffer starts empty");
    assert!(format!("{output:?}").contains("AsyncTranscodeEncodeOutput"));
    assert!(AsyncTranscodeEncodeOutput::try_with_capacity(ChunkedAsyncOutput::new(1), usize::MAX,).is_err());

    let mut output = AsyncTranscodeEncodeOutput::with_capacity(ChunkedAsyncOutput::new(1), 1);
    let mut encoder = CopyEncoder;
    let mut map_error = |_| io::Error::other("copy encoder cannot fail");
    let _ = complete(output.transcode_async(&mut encoder, &mut map_error, &['a'], 0, 1))?;
    assert_eq!(1, output.pending_len(), "encoding queues one byte before draining");
    complete(output.drain_async())?;
    assert_eq!(0, output.pending_len(), "draining clears the pending buffer");
    assert_eq!(
        b"a",
        output.inner().bytes.as_slice(),
        "draining writes the encoded byte to the inner output"
    );
    output.inner_mut().flushed = true;
    assert!(output.inner().flushed);
    Ok(())
}

/// Verifies zero-length encode operations and invalid source ranges.
#[test]
fn test_async_transcode_encode_output_validates_input_range() -> io::Result<()> {
    let mut output = AsyncTranscodeEncodeOutput::new(ChunkedAsyncOutput::new(1));
    let mut encoder = CopyEncoder;
    let mut map_error = |_| io::Error::other("copy encoder cannot fail");

    assert_eq!(
        TranscodeProgress::complete(0, 0),
        complete(output.transcode_async(&mut encoder, &mut map_error, &['a'], 0, 0,))?,
        "an empty input range completes without consuming or writing"
    );
    let error = complete(output.transcode_async(&mut encoder, &mut map_error, &['a'], 1, 1))
        .expect_err("invalid input range must fail");
    assert_eq!(
        io::ErrorKind::InvalidInput,
        error.kind(),
        "an invalid source range maps to InvalidInput"
    );
    Ok(())
}

/// Verifies capacity planning failures cross the asynchronous I/O boundary.
#[test]
fn test_async_transcode_encode_output_maps_capacity_errors() -> io::Result<()> {
    let mut output = AsyncTranscodeEncodeOutput::new(ChunkedAsyncOutput::new(1));
    let mut encoder = CapacityFailingEncoder::default();
    let mut map_error = |_| io::Error::other("encoder cannot reach domain failure");

    let error = complete(output.transcode_async(&mut encoder, &mut map_error, &['a'], 0, 1))
        .expect_err("capacity failure must cross the I/O boundary");
    assert_eq!(
        io::ErrorKind::InvalidData,
        error.kind(),
        "capacity arithmetic errors map to InvalidData"
    );
    encoder.maximum = true;
    let error = complete(output.transcode_async(&mut encoder, &mut map_error, &['a'], 0, 1))
        .expect_err("impossible spare capacity must fail");
    assert_eq!(
        io::ErrorKind::OutOfMemory,
        error.kind(),
        "an impossible spare-capacity bound maps to OutOfMemory"
    );

    let mut output = AsyncTranscodeEncodeOutput::new(ChunkedAsyncOutput::new(1));
    let mut encoder = FailingEncoder;
    let error = complete(output.transcode_async(&mut encoder, &mut map_error, &['a'], 0, 1))
        .expect_err("encoder errors must cross the asynchronous I/O boundary");
    assert_eq!(io::ErrorKind::Other, error.kind(), "domain encoder errors map to Other");

    let mut output = AsyncTranscodeEncodeOutput::new(ChunkedAsyncOutput::new(1));
    let mut encoder = NeedInputEncoder;
    let error = complete(output.transcode_async(&mut encoder, &mut map_error, &['a'], 0, 1))
        .expect_err("encoder NeedInput must be rejected");
    assert_eq!(
        io::ErrorKind::InvalidData,
        error.kind(),
        "an encoder NeedInput status violates the output contract"
    );
    Ok(())
}

/// Verifies lifecycle capacity and domain errors cross the async boundary.
#[test]
fn test_async_transcode_encode_output_maps_lifecycle_errors() -> io::Result<()> {
    let mut map_error = |_| io::Error::other("lifecycle failure");

    let mut output = AsyncTranscodeEncodeOutput::new(ChunkedAsyncOutput::new(1));
    let mut encoder = LifecycleFailingEncoder {
        overflow_reset_bound: true,
        ..Default::default()
    };
    let error =
        complete(output.reset_async(&mut encoder, &mut map_error)).expect_err("reset capacity failure must be mapped");
    assert_eq!(
        io::ErrorKind::InvalidData,
        error.kind(),
        "reset capacity arithmetic errors map to InvalidData"
    );

    let mut output = AsyncTranscodeEncodeOutput::new(ChunkedAsyncOutput::new(1));
    let mut encoder = LifecycleFailingEncoder {
        fail_reset: true,
        ..Default::default()
    };
    let error =
        complete(output.reset_async(&mut encoder, &mut map_error)).expect_err("reset domain failure must be mapped");
    assert_eq!(io::ErrorKind::Other, error.kind(), "reset domain errors map to Other");

    let mut output = AsyncTranscodeEncodeOutput::new(ChunkedAsyncOutput::new(1));
    let mut encoder = LifecycleFailingEncoder {
        overflow_finish_bound: true,
        ..Default::default()
    };
    let error = complete(output.finish_async(&mut encoder, &mut map_error))
        .expect_err("finish capacity failure must be mapped");
    assert_eq!(
        io::ErrorKind::InvalidData,
        error.kind(),
        "finish capacity arithmetic errors map to InvalidData"
    );

    let mut output = AsyncTranscodeEncodeOutput::new(ChunkedAsyncOutput::new(1));
    let mut encoder = LifecycleFailingEncoder {
        fail_finish: true,
        ..Default::default()
    };
    let error =
        complete(output.finish_async(&mut encoder, &mut map_error)).expect_err("finish domain failure must be mapped");
    assert_eq!(io::ErrorKind::Other, error.kind(), "finish domain errors map to Other");

    let mut output = AsyncTranscodeEncodeOutput::new(ChunkedAsyncOutput::new(1));
    let mut encoder = LifecycleFailingEncoder {
        huge_reset_bound: true,
        ..Default::default()
    };
    let error = complete(output.reset_async(&mut encoder, &mut map_error))
        .expect_err("impossible reset storage must be rejected");
    assert_eq!(
        io::ErrorKind::OutOfMemory,
        error.kind(),
        "an impossible reset bound maps to OutOfMemory"
    );

    let mut output = AsyncTranscodeEncodeOutput::new(ChunkedAsyncOutput::new(1));
    let mut encoder = LifecycleFailingEncoder {
        huge_finish_bound: true,
        ..Default::default()
    };
    let error = complete(output.finish_async(&mut encoder, &mut map_error))
        .expect_err("impossible finish storage must be rejected");
    assert_eq!(
        io::ErrorKind::OutOfMemory,
        error.kind(),
        "an impossible finish bound maps to OutOfMemory"
    );
    Ok(())
}

/// Verifies asynchronous output delivery failures remain visible to callers.
#[test]
fn test_async_transcode_encode_output_propagates_delivery_errors() -> io::Result<()> {
    let mut output = AsyncTranscodeEncodeOutput::with_capacity(ChunkedAsyncOutput::new(1), 1);
    let mut encoder = CopyEncoder;
    let mut map_error = |_| io::Error::other("copy encoder cannot fail");

    let _ = complete(output.transcode_async(&mut encoder, &mut map_error, &['a'], 0, 1))?;
    output.inner_mut().failed = true;

    let error = complete(output.drain_async()).expect_err("delivery error must be preserved");
    assert_eq!(
        io::ErrorKind::Other,
        error.kind(),
        "asynchronous delivery failures map to Other"
    );
    assert_eq!(1, output.pending_len());
    assert!(output.inner().bytes.is_empty());
    output.inner_mut().failed = false;
    complete(output.drain_async())?;
    assert_eq!(0, output.pending_len());
    assert_eq!(b"a", output.inner().bytes.as_slice());
    Ok(())
}

#[test]
fn test_cancelled_lifecycle_preserves_pending_and_runs_once() -> io::Result<()> {
    for finishing in [false, true] {
        let mut output = AsyncTranscodeEncodeOutput::with_capacity(ChunkedAsyncOutput::new(1), 2);
        let mut encoder = CountingCopyEncoder {
            lifecycle_bound: 2,
            ..Default::default()
        };
        let mut mapper = |_| io::Error::other("copy encoder cannot fail");
        complete(output.reset_async(&mut encoder, &mut mapper))?;
        assert_eq!(
            TranscodeProgress::complete(1, 1),
            complete(output.transcode_async(&mut encoder, &mut mapper, &['a'], 0, 1))?,
        );
        assert_eq!(2, output.pending_len());
        output.inner_mut().pending = false;
        let mut future = Box::pin(async {
            if finishing {
                output.finish_async(&mut encoder, &mut mapper).await
            } else {
                output.reset_async(&mut encoder, &mut mapper).await
            }
        });
        assert!(matches!(poll_once(future.as_mut()), Poll::Pending));
        drop(future);
        assert_eq!(1, encoder.reset_calls);
        assert_eq!(0, encoder.finish_calls);
        assert_eq!(b"^", output.inner().bytes.as_slice());
        assert_eq!(1, output.pending_len());
        if finishing {
            complete(output.finish_async(&mut encoder, &mut mapper))?;
        } else {
            complete(output.reset_async(&mut encoder, &mut mapper))?;
        }
        assert_eq!(if finishing { 1 } else { 2 }, encoder.reset_calls);
        assert_eq!(usize::from(finishing), encoder.finish_calls);
        complete(output.flush_async())?;
        let (sink, pending) = output.into_parts();
        assert!(pending.is_empty());
        assert_eq!(if finishing { b"^a!" } else { b"^a^" }, sink.bytes.as_slice());
    }
    Ok(())
}

#[test]
fn test_flush_recovery_does_not_repeat_finish() -> io::Result<()> {
    let mut output = AsyncTranscodeEncodeOutput::with_capacity(ChunkedAsyncOutput::new(1), 2);
    let mut encoder = CountingCopyEncoder::default();
    let mut mapper = |_| io::Error::other("copy encoder cannot fail");
    complete(output.reset_async(&mut encoder, &mut mapper))?;
    let progress = complete(output.transcode_async(&mut encoder, &mut mapper, &['a'], 0, 1))?;
    assert_eq!(TranscodeProgress::complete(1, 1), progress);
    complete(output.finish_async(&mut encoder, &mut mapper))?;
    complete(output.drain_async())?;
    assert_eq!(b"^a!", output.inner().bytes.as_slice());
    assert_eq!(1, encoder.finish_calls);

    output.inner_mut().flush_pending = true;
    let mut future = Box::pin(output.flush_async());
    assert!(matches!(poll_once(future.as_mut()), Poll::Pending));
    drop(future);
    assert_eq!(1, encoder.finish_calls);
    assert_eq!(0, output.pending_len());

    output.inner_mut().flush_fail_once = true;
    let error = complete(output.flush_async()).expect_err("one flush failure");
    assert_eq!(io::ErrorKind::Other, error.kind());
    assert_eq!(1, encoder.finish_calls);
    assert_eq!(b"^a!", output.inner().bytes.as_slice());
    complete(output.flush_async())?;
    assert!(output.inner().flushed);
    assert_eq!(3, output.inner().flush_calls);
    assert_eq!(1, encoder.finish_calls);
    assert_eq!(b"^a!", output.inner().bytes.as_slice());
    Ok(())
}
