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
}

impl ChunkedAsyncOutput {
    /// Creates an output that accepts at most `max_chunk` bytes per poll.
    fn new(max_chunk: usize) -> Self {
        Self {
            bytes: Vec::new(),
            max_chunk,
            pending: true,
            flushed: false,
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
        2,
        complete(output.transcode_async(
            &mut encoder,
            &mut map_error,
            &['a', 'b'],
            0,
            2,
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
