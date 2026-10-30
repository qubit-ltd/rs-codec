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
use std::pin::pin;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

use qubit_codec::AsyncTranscodeDecodeInput;
use qubit_codec::AsyncTranscodeDecodeStep;
use qubit_codec::CapacityError;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeProgress;
use qubit_codec::TranscodeStatus;
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
        output[index..index + read].copy_from_slice(&self.bytes[self.position..self.position + read]);
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
    let mut future = pin!(future);
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

#[derive(Debug, Default)]
struct FailedDecodeProgress {
    calls: usize,
    invalid_progress: bool,
}

impl Transcoder for FailedDecodeProgress {
    type Input = u8;
    type Output = u16;
    type Error = TranscodeDecodeError<TestDecodeError>;

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn reset(&mut self, _output: &mut [u16], _output_index: usize) -> Result<usize, Self::Error> {
        self.calls = 0;
        Ok(0)
    }

    fn transcode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        self.calls += 1;
        output[output_index] = 42;
        if self.invalid_progress {
            Ok(TranscodeProgress::complete(2, 1))
        } else {
            Err(TranscodeDecodeError::domain_main(TestDecodeError, 0))
        }
    }

    fn finish(&mut self, _output: &mut [u16], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

#[test]
fn test_poll_decode_error_preserves_unread_without_rolling_back_decoder_or_output() {
    let mut input = AsyncTranscodeDecodeInput::with_capacity(ChunkedAsyncInput::new(vec![7], 1), 1);
    assert!(complete(input.fill_until_async(1)).expect("prefill input before polling"));
    let mut decoder = FailedDecodeProgress::default();
    let mut reset_output = [];
    decoder.reset(&mut reset_output, 0).expect("reset decoder");
    let mut mapped_errors = 0;
    let mut mapper = |error| {
        mapped_errors += 1;
        io::Error::new(io::ErrorKind::InvalidData, error)
    };
    let mut output = [u16::MAX];
    let mut context = Context::from_waker(Waker::noop());

    let result = AsyncTranscodeDecodeInput::<ChunkedAsyncInput>::poll_transcode(
        Pin::new(&mut input),
        &mut context,
        &mut decoder,
        &mut mapper,
        &mut output,
        0,
        1,
    );

    let error = match result {
        Poll::Ready(Err(error)) => error,
        other => panic!("expected a ready decoder error, got {other:?}"),
    };
    assert_eq!(io::ErrorKind::InvalidData, error.kind());
    assert_eq!(1, mapped_errors);
    assert_eq!(1, decoder.calls);
    assert_eq!([42], output);
    assert_eq!([7], input.unread());
}

#[test]
fn test_poll_decode_invalid_progress_preserves_unread_after_output_write() {
    let mut input = AsyncTranscodeDecodeInput::with_capacity(ChunkedAsyncInput::new(vec![7], 1), 1);
    assert!(complete(input.fill_until_async(1)).expect("prefill input before polling"));
    let mut decoder = FailedDecodeProgress {
        invalid_progress: true,
        ..FailedDecodeProgress::default()
    };
    let mut reset_output = [];
    decoder.reset(&mut reset_output, 0).expect("reset decoder");
    let mut mapper = |error| io::Error::new(io::ErrorKind::InvalidData, error);
    let mut output = [u16::MAX];
    let mut context = Context::from_waker(Waker::noop());

    let result = AsyncTranscodeDecodeInput::<ChunkedAsyncInput>::poll_transcode(
        Pin::new(&mut input),
        &mut context,
        &mut decoder,
        &mut mapper,
        &mut output,
        0,
        1,
    );

    let error = match result {
        Poll::Ready(Err(error)) => error,
        other => panic!("expected a ready invalid-progress error, got {other:?}"),
    };
    assert_eq!(io::ErrorKind::InvalidData, error.kind());
    assert_eq!(1, decoder.calls);
    assert_eq!([42], output);
    assert_eq!([7], input.unread());
}

impl Transcoder for PairDecoder {
    type Input = u8;
    type Output = u16;
    type Error = TranscodeDecodeError<TestDecodeError>;

    /// Reports the maximum output length for a supplied byte input.
    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len / 2)
    }

    /// Resets this stateless decoder.
    fn reset(&mut self, _output: &mut [u16], _output_index: usize) -> Result<usize, Self::Error> {
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
            return Ok(TranscodeProgress::need_input(utils_crate::nonzero(2), 0, 0));
        }
        if available_output == 0 {
            return Ok(TranscodeProgress::need_output(utils_crate::nonzero(1), 0, 0));
        }
        output[output_index] = u16::from_be_bytes([input[input_index], input[input_index + 1]]);
        Ok(TranscodeProgress::complete(2, 1))
    }

    /// Finishes this stateless decoder without output.
    fn finish(&mut self, _output: &mut [u16], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

/// Decoder that preserves an incomplete pair until EOF, then emits its last
/// byte.
#[derive(Debug, Default)]
struct EofTailDecoder;

impl Transcoder for EofTailDecoder {
    type Input = u8;
    type Output = u16;
    type Error = TranscodeDecodeError<TestDecodeError>;

    /// Reserves enough output for EOF handling of every input byte.
    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    /// Resets the stateless decoder without writing output.
    fn reset(&mut self, _output: &mut [u16], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }

    /// Requests more input while retaining an incomplete pair.
    fn transcode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Ok(TranscodeProgress::need_input(utils_crate::nonzero(2), 0, 0))
    }

    /// Emits a single remaining byte as a value when the source reaches EOF.
    fn transcode_eof(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if input_index == input.len() {
            return Ok(TranscodeProgress::complete(0, 0));
        }
        output[output_index] = u16::from(input[input_index]);
        Ok(TranscodeProgress::complete(1, 1))
    }

    /// Finishes without additional output after EOF-tail processing.
    fn finish(&mut self, _output: &mut [u16], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

/// Decoder fixture for exercising EOF after empty and non-empty input.
///
/// It emits a two-value marker once at EOF, using the retained byte when one
/// was consumed or starting at `42` when the stream was empty.
#[derive(Debug, Default)]
struct EmptyEofDecoder {
    /// Last source byte consumed before EOF, if any.
    retained: Option<u8>,
    /// Number of EOF decode attempts observed by the fixture.
    eof_calls: usize,
    /// Whether the two-value EOF marker has already been emitted.
    emitted: bool,
}

impl Transcoder for EmptyEofDecoder {
    type Input = u8;
    type Output = u16;
    type Error = TranscodeDecodeError<TestDecodeError>;

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        input_len.checked_add(2).ok_or(CapacityError::OutputLengthOverflow)
    }

    fn reset(&mut self, _output: &mut [u16], _output_index: usize) -> Result<usize, Self::Error> {
        self.retained = None;
        self.eof_calls = 0;
        self.emitted = false;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let consumed = input.len() - input_index;
        if consumed > 0 {
            self.retained = Some(input[input_index + consumed - 1]);
        }
        Ok(TranscodeProgress::complete(consumed, 0))
    }

    fn transcode_eof(
        &mut self,
        _input: &[u8],
        _input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        self.eof_calls += 1;
        if self.emitted {
            return Ok(TranscodeProgress::complete(0, 0));
        }
        if output.len() - output_index < 2 {
            return Ok(TranscodeProgress::need_output(utils_crate::nonzero(2), 0, 0));
        }
        let value = self.retained.take().map_or(42, u16::from);
        output[output_index] = value;
        output[output_index + 1] = value + 1;
        self.emitted = true;
        Ok(TranscodeProgress::complete(0, 2))
    }

    fn finish(&mut self, _output: &mut [u16], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

/// Decoder that handles complete pairs and resolves one byte at EOF.
#[derive(Debug, Default)]
struct PairAndTailDecoder;

impl Transcoder for PairAndTailDecoder {
    type Input = u8;
    type Output = u16;
    type Error = TranscodeDecodeError<TestDecodeError>;

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        // The ceiling covers an EOF tail; the extra slot covers this fixture's
        // conservative two-slot preflight.
        input_len
            .div_ceil(2)
            .checked_add(1)
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    fn reset(&mut self, output: &mut [u16], output_index: usize) -> Result<usize, Self::Error> {
        output[output_index] = 0xaaaa;
        Ok(1)
    }

    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let available = input.len() - input_index;
        if available < 2 {
            return Ok(TranscodeProgress::need_input(utils_crate::nonzero(2), 0, 0));
        }
        if output.len() == output_index {
            return Ok(TranscodeProgress::need_output(utils_crate::nonzero(1), 0, 0));
        }
        output[output_index] = u16::from_be_bytes([input[input_index], input[input_index + 1]]);
        Ok(TranscodeProgress::complete(2, 1))
    }

    fn transcode_eof(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u16],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let available = input.len() - input_index;
        if available == 0 {
            return Ok(TranscodeProgress::complete(0, 0));
        }
        if available == 1 && output.len() - output_index < 2 {
            return Ok(TranscodeProgress::need_output(utils_crate::nonzero(2), 0, 0));
        }
        if available == 1 {
            output[output_index] = u16::from(input[input_index]);
            return Ok(TranscodeProgress::complete(1, 1));
        }
        <Self as Transcoder>::transcode(self, input, input_index, output, output_index)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }

    fn finish(&mut self, output: &mut [u16], output_index: usize) -> Result<usize, Self::Error> {
        output[output_index] = 0xbbbb;
        Ok(1)
    }
}

#[derive(Debug, Default)]
struct EofNeedInputDecoder {
    eof_read: usize,
}

impl Transcoder for EofNeedInputDecoder {
    type Input = u8;
    type Output = u16;
    type Error = TranscodeDecodeError<TestDecodeError>;

    fn max_transcode_output_len(&self, len: usize) -> Result<usize, CapacityError> {
        Ok(len)
    }

    fn reset(&mut self, _out: &mut [u16], _index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        index: usize,
        _out: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Ok(TranscodeProgress::need_input(
            utils_crate::nonzero(input.len() - index + 1),
            0,
            0,
        ))
    }

    fn transcode_eof(
        &mut self,
        input: &[u8],
        index: usize,
        _out: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let remaining = input.len() - index - self.eof_read;
        Ok(TranscodeProgress::need_input(
            utils_crate::nonzero(remaining + 1),
            self.eof_read,
            0,
        ))
    }

    fn finish(&mut self, _out: &mut [u16], _index: usize) -> Result<usize, Self::Error> {
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

    fn max_transcode_output_len(&self, _input_len: usize) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }

    fn reset(&mut self, output: &mut [u16], output_index: usize) -> Result<usize, Self::Error> {
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

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }

    fn finish(&mut self, output: &mut [u16], output_index: usize) -> Result<usize, Self::Error> {
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

    fn max_transcode_output_len(&self, _input_len: usize) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        Err(CapacityError::OutputLengthOverflow)
    }

    fn reset(&mut self, _output: &mut [u16], _output_index: usize) -> Result<usize, Self::Error> {
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

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Err(CapacityError::OutputLengthOverflow)
    }

    fn finish(&mut self, _output: &mut [u16], _output_index: usize) -> Result<usize, Self::Error> {
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
    fn max_transcode_output_len(&self, _input_len: usize) -> Result<usize, CapacityError> {
        Ok(0)
    }

    /// Resets this stateless decoder.
    fn reset(&mut self, _output: &mut [u16], _output_index: usize) -> Result<usize, Self::Error> {
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
    fn finish(&mut self, _output: &mut [u16], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

/// Decoder that reports zero lifecycle capacity but still writes one value.
#[derive(Debug, Default)]
struct OverreportingLifecycleDecoder;

impl Transcoder for OverreportingLifecycleDecoder {
    type Input = u8;
    type Output = u16;
    type Error = TranscodeDecodeError<TestDecodeError>;

    /// Reports no output requirement.
    fn max_transcode_output_len(&self, _input_len: usize) -> Result<usize, CapacityError> {
        Ok(0)
    }

    /// Under-reports the reset bound so the engine sees an over-report.
    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        Ok(0)
    }

    /// Resets while writing one value past the reported bound of zero.
    fn reset(&mut self, output: &mut [u16], output_index: usize) -> Result<usize, Self::Error> {
        output[output_index] = 0xcccc;
        Ok(1)
    }

    /// Decodes nothing.
    fn transcode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
        _output: &mut [u16],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Ok(TranscodeProgress::complete(0, 0))
    }

    /// Under-reports the finish bound so the engine sees an over-report.
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(0)
    }

    /// Finishes while writing one value past the reported bound of zero.
    fn finish(&mut self, output: &mut [u16], output_index: usize) -> Result<usize, Self::Error> {
        output[output_index] = 0xdddd;
        Ok(1)
    }
}

/// Verifies reset over-reporting violates the reserved reset bound.
#[test]
#[should_panic(expected = "reset wrote beyond its bound")]
fn test_async_transcode_decode_input_panics_when_reset_overreports_bound() {
    let input = AsyncTranscodeDecodeInput::new(ChunkedAsyncInput::new(Vec::new(), 1));
    let mut decoder = OverreportingLifecycleDecoder;
    let mut map_error = |error| io::Error::new(io::ErrorKind::InvalidData, error);
    let mut output = [0_u16; 1];

    let _ = input.reset(&mut decoder, &mut map_error, &mut output, 0, 1);
}

/// Verifies finish over-reporting violates the reserved finish bound.
#[test]
#[should_panic(expected = "finish wrote beyond its bound")]
fn test_async_transcode_decode_input_panics_when_finish_overreports_bound() {
    let input = AsyncTranscodeDecodeInput::new(ChunkedAsyncInput::new(Vec::new(), 1));
    let mut decoder = OverreportingLifecycleDecoder;
    let mut map_error = |error| io::Error::new(io::ErrorKind::InvalidData, error);
    let mut output = [0_u16; 1];

    let _ = input.finish(&mut decoder, &mut map_error, &mut output, 0, 1);
}

/// Verifies consuming past the unread window violates the documented bound.
#[test]
#[should_panic(expected = "cannot consume beyond buffered input")]
fn test_async_transcode_decode_input_consume_panics_beyond_unread_window() {
    let mut input = AsyncTranscodeDecodeInput::new(ChunkedAsyncInput::new(vec![0x12], 1));
    complete(input.fill_more_async()).expect("initial fill should succeed");
    assert_eq!(1, input.unread_len());

    input.consume(2);
}

/// Verifies refilling across pending chunk boundaries before decoding.
#[test]
fn test_async_transcode_decode_input_refills_and_decodes() -> io::Result<()> {
    let mut input = AsyncTranscodeDecodeInput::with_capacity(ChunkedAsyncInput::new(vec![0x12, 0x34], 1), 2);
    let mut decoder = PairDecoder;
    let mut output = [0_u16; 1];
    let mut map_error = |error| io::Error::new(io::ErrorKind::InvalidData, error);

    let first = complete(input.transcode_async(&mut decoder, &mut map_error, &mut output, 0, 1))?;
    assert!(matches!(
        first,
        AsyncTranscodeDecodeStep::Progress(progress)
            if matches!(progress.status(), TranscodeStatus::NeedInput { .. })
    ));
    assert!(complete(input.fill_until_async(2))?);
    let step = complete(input.transcode_async(&mut decoder, &mut map_error, &mut output, 0, 1))?;

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
fn test_async_transcode_decode_input_preserves_incomplete_eof_suffix() -> io::Result<()> {
    let mut input = AsyncTranscodeDecodeInput::with_capacity(ChunkedAsyncInput::new(vec![0x12], 1), 2);
    assert!(!complete(input.fill_until_async(2))?);
    let mut decoder = PairDecoder;
    let mut output = [0_u16; 1];
    let mut map_error = |error| io::Error::new(io::ErrorKind::InvalidData, error);

    let step = complete(input.transcode_async(&mut decoder, &mut map_error, &mut output, 0, 1))?;

    assert_eq!(
        AsyncTranscodeDecodeStep::Progress(TranscodeProgress::need_input(utils_crate::nonzero(2), 0, 0,)),
        step
    );
    assert_eq!([0x12], input.unread());
    Ok(())
}

/// Verifies the full async lifecycle with a complete prefix and retained EOF
/// tail.
#[test]
fn test_async_transcode_decode_input_drives_complete_prefix_and_eof_tail() -> io::Result<()> {
    let source = ChunkedAsyncInput::new(vec![0x12, 0x34, 0x56], 2);
    let mut input = AsyncTranscodeDecodeInput::with_capacity(source, 2);
    let mut decoder = PairAndTailDecoder;
    let mut output = [0_u16; 4];
    assert_eq!(
        AsyncTranscodeDecodeStep::Progress(TranscodeProgress::complete(0, 0)),
        complete(input.transcode_async(
            &mut decoder,
            &mut |error| io::Error::new(io::ErrorKind::InvalidData, error),
            &mut output,
            0,
            0,
        ))?,
        "zero output capacity is not evidence of EOF",
    );
    let mut written = input.reset(
        &mut decoder,
        &mut |error| io::Error::new(io::ErrorKind::InvalidData, error),
        &mut output,
        0,
        4,
    )?;
    assert_eq!(1, written);

    let first = complete(input.transcode_async(
        &mut decoder,
        &mut |error| io::Error::new(io::ErrorKind::InvalidData, error),
        &mut output,
        written,
        4 - written,
    ))?;
    let AsyncTranscodeDecodeStep::Progress(progress) = first else {
        panic!("first read should produce progress");
    };
    assert_eq!(TranscodeStatus::Complete, progress.status());
    written += progress.written();
    assert_eq!(0, input.unread_len(), "bridge already applied progress.read()");
    assert_eq!(0x1234, output[1]);

    let second = complete(input.transcode_async(
        &mut decoder,
        &mut |error| io::Error::new(io::ErrorKind::InvalidData, error),
        &mut output,
        written,
        4 - written,
    ))?;
    let AsyncTranscodeDecodeStep::Progress(progress) = second else {
        panic!("the final source chunk should be decoded before EOF");
    };
    assert!(matches!(progress.status(), TranscodeStatus::NeedInput { .. }));
    assert_eq!(0, progress.read());
    written += progress.written();
    assert_eq!([0x56], input.unread(), "NeedInput retains the incomplete suffix");

    assert!(
        !complete(input.fill_until_async(2))?,
        "one retained byte cannot satisfy a pair"
    );
    assert_eq!([0x56], input.unread());
    let tail = input.transcode_eof_step(
        &mut decoder,
        &mut |error| io::Error::new(io::ErrorKind::InvalidData, error),
        &mut output,
        written,
        1,
    )?;
    assert_eq!(TranscodeProgress::need_output(utils_crate::nonzero(2), 0, 0), tail);
    let tail = input.transcode_eof_step(
        &mut decoder,
        &mut |error| io::Error::new(io::ErrorKind::InvalidData, error),
        &mut output,
        written,
        2,
    )?;
    assert_eq!(TranscodeProgress::complete(1, 1), tail);
    written += tail.written();
    assert_eq!(0, input.unread_len(), "EOF tail must be consumed exactly once");

    written += input.finish(
        &mut decoder,
        &mut |error| io::Error::new(io::ErrorKind::InvalidData, error),
        &mut output,
        written,
        4 - written,
    )?;
    assert_eq!([0xaaaa, 0x1234, 0x56, 0xbbbb], output);
    assert_eq!(4, written);
    Ok(())
}

#[test]
fn test_pair_and_tail_complete_stream_fits_the_public_bound() {
    let mut decoder = PairAndTailDecoder;
    let capacity = decoder.max_total_output_len(1).unwrap();
    let mut output = vec![0_u16; capacity];
    let written = decoder.transcode_complete_into(&[0x56], &mut output).unwrap();
    output.truncate(written);
    assert_eq!(output, [0xaaaa, 0x56, 0xbbbb]);
    assert_eq!(written, 3);
}

/// Verifies the default EOF policy rejects an incomplete retained suffix.
#[test]
fn test_async_transcode_decode_input_strict_eof_rejects_incomplete_tail() -> io::Result<()> {
    let mut input = AsyncTranscodeDecodeInput::with_capacity(ChunkedAsyncInput::new(vec![0x12], 1), 2);
    assert!(!complete(input.fill_until_async(2))?);
    let mut decoder = PairDecoder;
    let mut output = [0_u16; 1];
    let error = input
        .transcode_eof_step(
            &mut decoder,
            &mut |error| io::Error::new(io::ErrorKind::InvalidData, error),
            &mut output,
            0,
            1,
        )
        .expect_err("the default EOF policy must reject an incomplete pair");
    assert_eq!(io::ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("incomplete input"));
    assert_eq!([0x12], input.unread());
    Ok(())
}

/// Verifies a decoded value is returned before a later input poll can pend.
#[test]
fn test_async_transcode_decode_input_commits_progress_before_later_pending() -> io::Result<()> {
    let mut source = ChunkedAsyncInput::new(vec![0x12, 0x34, 0x56], 2);
    source.pending = false;
    let mut input = AsyncTranscodeDecodeInput::with_capacity(source, 2);
    let mut decoder = PairDecoder;
    let mut output = [0_u16; 2];
    let mut map_error = |error| io::Error::new(io::ErrorKind::InvalidData, error);

    let mut future = Box::pin(input.transcode_async(&mut decoder, &mut map_error, &mut output, 0, 2));
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
fn test_async_transcode_decode_input_rejects_invalid_progress() -> io::Result<()> {
    let mut input = AsyncTranscodeDecodeInput::with_capacity(ChunkedAsyncInput::new(vec![0x12], 1), 1);
    let mut decoder = NoProgressDecoder;
    let mut output = [0_u16; 1];
    let mut map_error = |error| io::Error::new(io::ErrorKind::InvalidData, error);

    let error = complete(input.transcode_async(&mut decoder, &mut map_error, &mut output, 0, 1))
        .expect_err("invalid progress must be rejected");

    assert_eq!(io::ErrorKind::InvalidData, error.kind());
    Ok(())
}

/// Verifies buffered-input operations preserve the unread window.
#[test]
fn test_async_transcode_decode_input_exposes_buffer_operations() -> io::Result<()> {
    let mut input = AsyncTranscodeDecodeInput::new(ChunkedAsyncInput::new(vec![0x12, 0x34], 2));

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
    let input = AsyncTranscodeDecodeInput::try_with_capacity(ChunkedAsyncInput::new(Vec::new(), 1), 3)?;
    assert!(input.capacity() >= 3);
    assert!(format!("{input:?}").contains("AsyncTranscodeDecodeInput"));

    assert!(AsyncTranscodeDecodeInput::try_with_capacity(ChunkedAsyncInput::new(Vec::new(), 1), usize::MAX,).is_err());

    let mut input = AsyncTranscodeDecodeInput::with_capacity(ChunkedAsyncInput::new(vec![0x12], 1), 1);
    assert!(!complete(input.fill_until_async(2))?);
    assert_eq!([0x12], input.unread());
    assert!(!complete(input.fill_more_async())?);
    Ok(())
}

/// Verifies zero-length decode operations and invalid destination ranges.
#[test]
fn test_async_transcode_decode_input_validates_output_range() -> io::Result<()> {
    let mut input = AsyncTranscodeDecodeInput::new(ChunkedAsyncInput::new(Vec::new(), 1));
    let mut decoder = PairDecoder;
    let mut map_error = |error| io::Error::new(io::ErrorKind::InvalidData, error);
    let mut output = [0_u16; 1];

    assert_eq!(
        AsyncTranscodeDecodeStep::Progress(TranscodeProgress::complete(0, 0)),
        complete(input.transcode_async(&mut decoder, &mut map_error, &mut output, 0, 0,))?,
    );
    let error = complete(input.transcode_async(&mut decoder, &mut map_error, &mut output, 1, 1))
        .expect_err("invalid output range must fail");
    assert_eq!(io::ErrorKind::InvalidInput, error.kind());
    Ok(())
}

/// Verifies lifecycle output uses the caller's indexed destination range.
#[test]
fn test_async_transcode_decode_input_runs_decoder_lifecycle() -> io::Result<()> {
    let input = AsyncTranscodeDecodeInput::new(ChunkedAsyncInput::new(Vec::new(), 1));
    let mut decoder = LifecycleDecoder;
    let mut map_error = |error| io::Error::new(io::ErrorKind::InvalidData, error);
    let mut output = [0_u16; 3];

    assert_eq!(1, input.reset(&mut decoder, &mut map_error, &mut output, 1, 1)?);
    assert_eq!(1, input.finish(&mut decoder, &mut map_error, &mut output, 2, 1)?);
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
    let mut input = AsyncTranscodeDecodeInput::new(ChunkedAsyncInput::new(vec![0x12], 1));
    let mut units = [0_u8; 1];
    assert_eq!(1, complete(input.read_async(&mut units))?);
    assert_eq!([0x12], units);

    let mut decoder = PairDecoder;
    let mut map_error = |error| io::Error::new(io::ErrorKind::InvalidData, error);
    let mut output = [0_u16; 1];
    assert_eq!(
        AsyncTranscodeDecodeStep::EndOfInput,
        complete(input.transcode_async(&mut decoder, &mut map_error, &mut output, 0, 1,))?,
    );
    Ok(())
}

#[test]
fn test_async_transcode_decode_input_transcode_eof_step_consumes_buffered_tail() -> io::Result<()> {
    let mut input = AsyncTranscodeDecodeInput::with_capacity(ChunkedAsyncInput::new(vec![0x5a], 1), 1);
    assert!(complete(input.fill_more_async())?);
    let mut decoder = EofTailDecoder;
    let mut map_error = |error| io::Error::new(io::ErrorKind::InvalidData, error);
    let mut output = [0_u16; 1];

    let progress = input.transcode_eof_step(&mut decoder, &mut map_error, &mut output, 0, 1)?;

    assert_eq!(TranscodeProgress::complete(1, 1), progress);
    assert_eq!([0x5a], output);
    assert_eq!(0, input.unread_len());

    assert_eq!(
        TranscodeProgress::complete(0, 0),
        input.transcode_eof_step(&mut decoder, &mut map_error, &mut output, 0, 0,)?,
    );
    assert_eq!(
        TranscodeProgress::complete(0, 0),
        input.transcode_eof_step(&mut decoder, &mut map_error, &mut output, 0, 1,)?,
    );
    let error = input
        .transcode_eof_step(&mut decoder, &mut map_error, &mut output, 1, 1)
        .expect_err("invalid EOF output range must fail");
    assert_eq!(io::ErrorKind::InvalidInput, error.kind());
    Ok(())
}

#[test]
fn test_async_eof_need_input_is_rejected_before_consumption() -> io::Result<()> {
    let source = ChunkedAsyncInput::new(vec![1, 2], 2);
    let mut input = AsyncTranscodeDecodeInput::with_capacity(source, 2);
    assert!(complete(input.fill_until_async(2))?);
    let mut decoder = EofNeedInputDecoder { eof_read: 1 };
    let mut mapper = |_| io::Error::other("unexpected decoder error");
    let error = input
        .transcode_eof_step(&mut decoder, &mut mapper, &mut [0_u16; 1], 0, 1)
        .expect_err("NeedInput is invalid at EOF");
    assert_eq!(io::ErrorKind::InvalidData, error.kind());
    assert_eq!(&[1, 2], input.unread());
    Ok(())
}

#[test]
fn test_async_eof_need_input_without_consumption_is_rejected() -> io::Result<()> {
    let source = ChunkedAsyncInput::new(vec![1], 1);
    let mut input = AsyncTranscodeDecodeInput::with_capacity(source, 1);
    assert!(complete(input.fill_until_async(1))?);
    let mut decoder = EofNeedInputDecoder::default();
    let mut mapper = |_| io::Error::other("unexpected decoder error");
    let error = input
        .transcode_eof_step(&mut decoder, &mut mapper, &mut [0_u16; 1], 0, 1)
        .expect_err("NeedInput is invalid at EOF");
    assert_eq!(io::ErrorKind::InvalidData, error.kind());
    assert_eq!(&[1], input.unread());
    Ok(())
}

#[test]
fn test_async_transcode_decode_input_dispatches_empty_eof_and_retries_output() -> io::Result<()> {
    let mut input = AsyncTranscodeDecodeInput::new(ChunkedAsyncInput::new(Vec::new(), 1));
    let mut decoder = EmptyEofDecoder::default();
    let mut map_error = |error| io::Error::new(io::ErrorKind::InvalidData, error);
    let mut output = [0_u16; 2];

    assert_eq!(
        AsyncTranscodeDecodeStep::EndOfInput,
        complete(input.transcode_async(&mut decoder, &mut map_error, &mut output, 0, 2,))?
    );
    assert_eq!(
        TranscodeProgress::complete(0, 0),
        input.transcode_eof_step(&mut decoder, &mut map_error, &mut output, 0, 0)?,
    );
    assert_eq!(0, decoder.eof_calls);
    assert_eq!(
        TranscodeProgress::need_output(utils_crate::nonzero(2), 0, 0),
        input.transcode_eof_step(&mut decoder, &mut map_error, &mut output, 0, 1)?,
    );
    assert_eq!(1, decoder.eof_calls);

    assert_eq!(
        TranscodeProgress::complete(0, 2),
        input.transcode_eof_step(&mut decoder, &mut map_error, &mut output, 0, 2)?,
    );
    assert_eq!([42, 43], output);
    assert_eq!(2, decoder.eof_calls);
    Ok(())
}

#[test]
fn test_async_transcode_decode_input_empty_eof_flushes_decoder_state() -> io::Result<()> {
    let mut input = AsyncTranscodeDecodeInput::new(ChunkedAsyncInput::new(vec![9], 1));
    let mut decoder = EmptyEofDecoder::default();
    let mut map_error = |error| io::Error::new(io::ErrorKind::InvalidData, error);
    let mut output = [0_u16; 2];

    assert!(complete(input.fill_more_async())?);
    let AsyncTranscodeDecodeStep::Progress(progress) =
        complete(input.transcode_async(&mut decoder, &mut map_error, &mut output, 0, 2))?
    else {
        panic!("the input byte should be consumed before EOF");
    };
    assert_eq!(TranscodeProgress::complete(1, 0), progress);
    assert!(!complete(input.fill_more_async())?);
    assert_eq!(
        TranscodeProgress::complete(0, 2),
        input.transcode_eof_step(&mut decoder, &mut map_error, &mut output, 0, 2)?,
    );
    assert_eq!([9, 10], output);
    assert_eq!(1, decoder.eof_calls);
    Ok(())
}

/// Verifies decoder capacity failures become invalid-data I/O errors.
#[test]
fn test_async_transcode_decode_input_maps_lifecycle_capacity_errors() -> io::Result<()> {
    let input = AsyncTranscodeDecodeInput::new(ChunkedAsyncInput::new(Vec::new(), 1));
    let mut decoder = CapacityFailingDecoder;
    let mut map_error = |error| io::Error::new(io::ErrorKind::InvalidData, error);
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
    let error = complete(input.fill_more_async()).expect_err("input failure must be preserved");
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

    let mut input = AsyncTranscodeDecodeInput::new(ChunkedAsyncInput::new(Vec::new(), 1));
    let error = complete(input.fill_until_async(usize::MAX)).expect_err("impossible capacity must fail");
    assert_eq!(io::ErrorKind::OutOfMemory, error.kind());
    Ok(())
}

/// Verifies the poll-based entry point surfaces pending and then commits
/// progress.
#[test]
fn test_async_transcode_decode_input_poll_transcode_commits_progress_after_pending() {
    let mut input = AsyncTranscodeDecodeInput::with_capacity(ChunkedAsyncInput::new(vec![0x12, 0x34], 1), 2);
    let mut context = Context::from_waker(Waker::noop());
    let mut decoder = PairDecoder;
    let mut map_error = |error| io::Error::new(io::ErrorKind::InvalidData, error);
    let mut output = [0_u16; 1];

    let pending = AsyncTranscodeDecodeInput::<ChunkedAsyncInput>::poll_transcode(
        Pin::new(&mut input),
        &mut context,
        &mut decoder,
        &mut map_error,
        &mut output,
        0,
        1,
    );

    assert!(
        matches!(pending, Poll::Pending),
        "the poll-based entry point must surface the source pending state, got {pending:?}"
    );

    let ready = AsyncTranscodeDecodeInput::<ChunkedAsyncInput>::poll_transcode(
        Pin::new(&mut input),
        &mut context,
        &mut decoder,
        &mut map_error,
        &mut output,
        0,
        1,
    );

    assert!(
        matches!(ready, Poll::Ready(Ok(AsyncTranscodeDecodeStep::Progress(_)))),
        "polling again after the source unblocks must commit one progress step, got {ready:?}"
    );
}
