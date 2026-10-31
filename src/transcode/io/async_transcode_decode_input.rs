// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Buffered asynchronous input driver that decodes units into values.

use core::fmt;
use std::collections::TryReserveError;
use std::future::poll_fn;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::pin::Pin;
use std::result::Result as StdResult;
use std::task::Context;
use std::task::Poll;

use qubit_io::AsyncBufferedInput;
use qubit_io::AsyncInput;
use qubit_io::Buffer;
use qubit_utils::SliceRange;
use qubit_utils::allocation_error;

use super::async_transcode_decode_step::AsyncTranscodeDecodeStep;
use super::internal::capacity_error_to_invalid_data;
use super::internal::validate_decode_progress;
use super::internal::validate_eof_decode_progress;
use crate::TranscodeProgress;
use crate::Transcoder;

/// Decodes an asynchronous unit stream into values.
///
/// The adapter owns unit buffering but not decoder state. Callers supply a
/// streaming decoder and error mapper for each operation.
///
/// # Driving input through EOF
///
/// Reset the decoder first and collect the number of values it writes. For
/// each [`Self::poll_transcode`] progress result, add `written()` to the output
/// cursor; the bridge has already consumed `read()` from its unread buffer.
/// When the status is `NeedInput { required }`, call
/// [`Self::fill_until_async`] with `required.get()`. If it returns `true`, poll
/// the decoder again. If it returns `false`, EOF has been confirmed and any
/// short tail remains unread; pass that tail to [`Self::transcode_eof_step`].
/// Keep stepping on `NeedOutput` with a sufficiently large output range. Once
/// EOF processing returns `Complete`, call [`Self::finish`] and collect its
/// returned output count. `EndOfInput` from polling means the unread buffer was
/// already empty when refill confirmed EOF. In both cases, call
/// [`Self::transcode_eof_step`] to dispatch the EOF event, including when no
/// unread units remain.
///
/// # Type Parameters
///
/// - `I`: Wrapped asynchronous unit input.
///
/// # Examples
///
/// ```
/// use qubit_codec::AsyncTranscodeDecodeInput;
/// use qubit_io::AsyncInput;
///
/// # use std::io::Result;
/// # use std::pin::Pin;
/// # use std::task::Context;
/// # use std::task::Poll;
/// # struct EmptyInput;
/// # impl AsyncInput for EmptyInput {
/// #     type Item = u8;
/// #     unsafe fn poll_read_unchecked(self: Pin<&mut Self>, _cx: &mut Context<'_>, _output: &mut [u8], _index: usize, _count: usize) -> Poll<Result<usize>> {
/// #         Poll::Ready(Ok(0))
/// #     }
/// # }
/// let mut input = AsyncTranscodeDecodeInput::with_capacity(EmptyInput, 8);
/// assert!(input.capacity() >= 8);
/// assert_eq!(input.unread_len(), 0);
/// input.consume(0);
/// assert!(input.unread().is_empty());
/// ```
///
/// The following driver decodes one complete pair, preserves a one-unit tail,
/// retries EOF processing after `NeedOutput`, and collects lifecycle output
/// without an async runtime. The source and decoder are small fixtures; real
/// applications use their own `AsyncInput` and `Transcoder` implementations.
///
/// ```
/// use qubit_codec::{AsyncTranscodeDecodeInput, AsyncTranscodeDecodeStep, CapacityError, TranscodeDecodeError, TranscodeProgress, Transcoder};
/// use qubit_io::AsyncInput;
/// use std::convert::Infallible;
/// use std::future::{poll_fn, Future};
/// use std::io;
/// use std::num::NonZeroUsize;
/// use std::pin::{pin, Pin};
/// use std::task::{Context, Poll, Waker};
///
/// # #[derive(Debug)] struct ThreeBytes { bytes: [u8; 3], position: usize }
/// # impl AsyncInput for ThreeBytes {
/// #     type Item = u8;
/// #     unsafe fn poll_read_unchecked(mut self: Pin<&mut Self>, _cx: &mut Context<'_>, output: &mut [u8], index: usize, count: usize) -> Poll<io::Result<usize>> {
/// #         let read = count.min(self.bytes.len() - self.position);
/// #         output[index..index + read].copy_from_slice(&self.bytes[self.position..self.position + read]);
/// #         self.position += read;
/// #         Poll::Ready(Ok(read))
/// #     }
/// # }
/// # #[derive(Debug)] struct TailDecoder;
/// # impl Transcoder for TailDecoder {
/// #     type Input = u8; type Output = u16; type Error = TranscodeDecodeError<Infallible>;
/// #     fn max_transcode_output_len(&self, n: usize) -> Result<usize, CapacityError> { Ok(n) }
/// #     fn max_reset_output_len(&self) -> Result<usize, CapacityError> { Ok(1) }
/// #     fn reset(&mut self, out: &mut [u16], index: usize) -> Result<usize, Self::Error> { out[index] = 0xaaaa; Ok(1) }
/// #     fn transcode(&mut self, input: &[u8], ii: usize, out: &mut [u16], oi: usize) -> Result<TranscodeProgress, Self::Error> {
/// #         let available = input.len() - ii;
/// #         if available < 2 { return Ok(TranscodeProgress::need_input(NonZeroUsize::new(2).unwrap(), 0, 0)); }
/// #         if out.len() == oi { return Ok(TranscodeProgress::need_output(NonZeroUsize::new(1).unwrap(), 0, 0)); }
/// #         out[oi] = u16::from_be_bytes([input[ii], input[ii + 1]]);
/// #         Ok(TranscodeProgress::complete(2, 1))
/// #     }
/// #     fn transcode_eof(&mut self, input: &[u8], ii: usize, out: &mut [u16], oi: usize) -> Result<TranscodeProgress, Self::Error> {
/// #         let available = input.len() - ii;
/// #         if available == 0 { return Ok(TranscodeProgress::complete(0, 0)); }
/// #         if available == 1 {
/// #             if out.len() - oi < 2 { return Ok(TranscodeProgress::need_output(NonZeroUsize::new(2).unwrap(), 0, 0)); }
/// #             out[oi] = u16::from(input[ii]);
/// #             return Ok(TranscodeProgress::complete(1, 1));
/// #         }
/// #         self.transcode(input, ii, out, oi)
/// #     }
/// #     fn max_finish_output_len(&self) -> Result<usize, CapacityError> { Ok(1) }
/// #     fn finish(&mut self, out: &mut [u16], index: usize) -> Result<usize, Self::Error> { out[index] = 0xbbbb; Ok(1) }
/// # }
/// # fn block_on<F: Future>(future: F) -> F::Output {
/// #     let mut future = pin!(future); let mut cx = Context::from_waker(Waker::noop());
/// #     loop { if let Poll::Ready(value) = future.as_mut().poll(&mut cx) { return value; } }
/// # }
/// let source = ThreeBytes { bytes: [0x12, 0x34, 0x56], position: 0 };
/// let mut input = AsyncTranscodeDecodeInput::with_capacity(source, 2);
/// let mut decoder = TailDecoder;
/// let mut output = [0_u16; 4];
/// let output_len = output.len();
/// let mut map_error = |error| io::Error::new(io::ErrorKind::InvalidData, error);
/// let mut written = input.reset(&mut decoder, &mut map_error, &mut output, 0, output_len)?;
/// assert_eq!(1, written);
/// let first = block_on(poll_fn(|cx| Pin::new(&mut input).poll_transcode(cx, &mut decoder, &mut map_error, &mut output, written, output_len - written)))?;
/// let AsyncTranscodeDecodeStep::Progress(progress) = first else { panic!("expected decoded pair") };
/// assert_eq!(TranscodeProgress::complete(2, 1), progress);
/// written += progress.written();
/// assert_eq!(0x1234, output[1]);
/// let second = block_on(poll_fn(|cx| Pin::new(&mut input).poll_transcode(cx, &mut decoder, &mut map_error, &mut output, written, output_len - written)))?;
/// assert!(matches!(second, AsyncTranscodeDecodeStep::Progress(progress) if matches!(progress.status(), qubit_codec::TranscodeStatus::NeedInput { .. })));
/// assert!(!block_on(input.fill_until_async(2))?);
/// assert_eq!([0x56], input.unread());
/// let eof = input.transcode_eof_step(&mut decoder, &mut map_error, &mut output, written, 1)?;
/// assert_eq!(TranscodeProgress::need_output(NonZeroUsize::new(2).unwrap(), 0, 0), eof);
/// let eof = input.transcode_eof_step(&mut decoder, &mut map_error, &mut output, written, 2)?;
/// assert_eq!(TranscodeProgress::complete(1, 1), eof);
/// written += eof.written();
/// let finished = input.finish(&mut decoder, &mut map_error, &mut output, written, output_len - written)?;
/// written += finished;
/// assert_eq!([0xaaaa, 0x1234, 0x56, 0xbbbb], output);
/// assert_eq!(4, written);
/// # Ok::<(), io::Error>(())
/// ```
#[must_use]
pub struct AsyncTranscodeDecodeInput<I>
where
    I: AsyncInput,
    I::Item: Copy + Default,
{
    /// Buffered asynchronous unit input.
    input: AsyncBufferedInput<I>,
}

impl<I> AsyncTranscodeDecodeInput<I>
where
    I: AsyncInput,
    I::Item: Copy + Default,
{
    /// Creates an adapter with the default unit-buffer capacity.
    ///
    /// # Parameters
    ///
    /// - `inner`: Unit input read by this adapter.
    ///
    /// # Returns
    ///
    /// Returns a new buffered asynchronous decoder input.
    #[inline]
    pub fn new(inner: I) -> Self {
        Self {
            input: AsyncBufferedInput::new(inner),
        }
    }

    /// Creates an adapter with an internal unit buffer of at least capacity.
    ///
    /// # Parameters
    ///
    /// - `inner`: Unit input read by this adapter.
    /// - `capacity`: Requested internal unit-buffer capacity.
    ///
    /// # Returns
    ///
    /// Returns a new buffered asynchronous decoder input.
    #[inline]
    pub fn with_capacity(inner: I, capacity: usize) -> Self {
        Self {
            input: AsyncBufferedInput::with_capacity(inner, capacity),
        }
    }

    /// Tries to create an adapter with an internal unit buffer of at least
    /// capacity.
    ///
    /// # Parameters
    ///
    /// - `inner`: Asynchronous unit source owned by the adapter.
    /// - `capacity`: Minimum requested unit-buffer capacity.
    ///
    /// # Returns
    ///
    /// The adapter with an empty unread window on successful allocation.
    ///
    /// # Errors
    ///
    /// Returns the allocation error when the internal buffer cannot be
    /// allocated.
    #[inline]
    pub fn try_with_capacity(inner: I, capacity: usize) -> StdResult<Self, TryReserveError> {
        Ok(Self {
            input: AsyncBufferedInput::try_with_capacity(inner, capacity)?,
        })
    }

    /// Returns a shared reference to the wrapped asynchronous input.
    ///
    /// The input can be physically positioned after units retained in this
    /// adapter's unread buffer.
    ///
    /// # Returns
    ///
    /// A shared borrow of the underlying source, excluding buffered units.
    #[must_use = "the wrapped input should be inspected or used"]
    #[inline]
    pub const fn inner(&self) -> &I {
        self.input.inner()
    }

    /// Returns a mutable reference to the wrapped asynchronous input.
    ///
    /// Direct reads can invalidate the logical stream position represented by
    /// unread buffered units.
    ///
    /// # Returns
    ///
    /// An exclusive borrow of the source; direct reads bypass buffered units.
    #[must_use = "the wrapped input should be inspected or used"]
    #[inline]
    pub fn inner_mut(&mut self) -> &mut I {
        self.input.inner_mut()
    }

    /// Returns the number of unread units currently buffered.
    ///
    /// # Returns
    ///
    /// The number of buffered units not yet consumed.
    #[must_use]
    #[inline]
    pub const fn unread_len(&self) -> usize {
        self.input.unread_len()
    }

    /// Returns the unread buffered unit window.
    ///
    /// # Returns
    ///
    /// The currently retained units borrowed without allocation.
    #[must_use]
    #[inline]
    pub fn unread(&self) -> &[I::Item] {
        self.input.unread()
    }

    /// Returns the total internal unit-buffer capacity.
    ///
    /// # Returns
    ///
    /// The total number of units the internal buffer can hold.
    #[must_use]
    #[inline]
    pub fn capacity(&self) -> usize {
        self.input.capacity()
    }

    /// Consumes unread units from the current buffer window.
    ///
    /// # Parameters
    ///
    /// - `count`: Number of retained units to discard from the unread prefix.
    ///
    /// # Panics
    ///
    /// Panics when count exceeds the unread unit count.
    #[inline]
    pub fn consume(&mut self, count: usize) {
        assert!(count <= self.unread_len(), "cannot consume beyond buffered input",);
        // SAFETY: The asserted bound proves count fits the unread window.
        unsafe {
            self.input.consume(count);
        }
    }

    /// Copies unread units into an indexed output range without consuming them.
    ///
    /// # Parameters
    ///
    /// - `output`: Destination slice receiving copies of retained units.
    /// - `output_index`: First destination index.
    /// - `count`: Number of unread units to copy without consuming them.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the indexed range fits in output, does
    /// not overflow, holds no overlapping unread units, and count is no
    /// greater than the unread unit count.
    pub unsafe fn copy_unread_to(&self, output: &mut [I::Item], output_index: usize, count: usize) {
        // SAFETY: The caller upholds the delegated buffer-copy contract.
        unsafe {
            self.input.copy_unread_to(output, output_index, count);
        }
    }

    /// Consumes this adapter and returns the input plus its unread buffer.
    ///
    /// # Returns
    ///
    /// The owned source and buffer, preserving all unread units.
    #[must_use = "the returned input and unread buffer must be handled"]
    #[inline]
    pub fn into_parts(self) -> (I, Buffer<I::Item>) {
        self.input.into_parts()
    }

    /// Runs decoder reset into an indexed output range without I/O.
    ///
    /// # Type Parameters
    ///
    /// - `D`: Stateful decoder from source units into values.
    /// - `M`: Callback mapping decoder errors to I/O errors.
    /// - `Value`: Destination value type.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Decoder whose state may change during this operation.
    /// - `map_error`: Callback invoked for decoder errors.
    /// - `output`: Destination value slice.
    /// - `output_index`: First destination index.
    /// - `count`: Number of writable destination positions.
    ///
    /// # Returns
    ///
    /// Number of values emitted during decoder reset.
    ///
    /// # Errors
    ///
    /// Returns invalid output-range, capacity, or mapped decoder-reset errors.
    ///
    /// # Panics
    ///
    /// Panics if the decoder writes more values than its reset bound.
    pub fn reset<D, M, Value>(
        &self,
        decoder: &mut D,
        map_error: &mut M,
        output: &mut [Value],
        output_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        D: Transcoder<Input = I::Item, Output = Value>,
        M: FnMut(D::Error) -> Error,
    {
        let required = decoder.max_reset_output_len().map_err(capacity_error_to_invalid_data)?;
        let output_end = SliceRange::checked_range_end(
            output.len(),
            output_index,
            count,
            "reset output range exceeds destination buffer",
        )?;
        if count < required {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "insufficient output for decoder reset bound",
            ));
        }
        let output = &mut output[..output_end];
        let written = decoder.reset(output, output_index).map_err(&mut *map_error)?;
        assert!(written <= required, "reset wrote beyond its bound");
        Ok(written)
    }

    /// Finishes a decoder into an indexed output range without I/O.
    ///
    /// # Type Parameters
    ///
    /// - `D`: Stateful decoder from source units into values.
    /// - `M`: Callback mapping decoder errors to I/O errors.
    /// - `Value`: Destination value type.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Decoder whose state may change during this operation.
    /// - `map_error`: Callback invoked for decoder errors.
    /// - `output`: Destination value slice.
    /// - `output_index`: First destination index.
    /// - `count`: Number of writable destination positions.
    ///
    /// # Returns
    ///
    /// Number of values emitted while finishing the decoder.
    ///
    /// # Errors
    ///
    /// Returns invalid output-range, capacity, or mapped decoder-finish
    /// errors.
    ///
    /// # Panics
    ///
    /// Panics if the decoder writes more values than its finish bound.
    pub fn finish<D, M, Value>(
        &self,
        decoder: &mut D,
        map_error: &mut M,
        output: &mut [Value],
        output_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        D: Transcoder<Input = I::Item, Output = Value>,
        M: FnMut(D::Error) -> Error,
    {
        let required = decoder
            .max_finish_output_len()
            .map_err(capacity_error_to_invalid_data)?;
        let output_end = SliceRange::checked_range_end(
            output.len(),
            output_index,
            count,
            "finish output range exceeds destination buffer",
        )?;
        if count < required {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "insufficient output for decoder finish bound",
            ));
        }
        let output = &mut output[..output_end];
        let written = decoder.finish(output, output_index).map_err(&mut *map_error)?;
        assert!(written <= required, "finish wrote beyond its bound");
        Ok(written)
    }
}

impl<I> AsyncTranscodeDecodeInput<I>
where
    I: AsyncInput + Unpin,
    I::Item: Copy + Default + Unpin,
{
    /// Appends at least one unit to the unread buffer.
    ///
    /// # Returns
    ///
    /// Returns true when data was appended, or false at end of input.
    ///
    /// # Errors
    ///
    /// Returns input or buffer-management errors while refilling.
    #[inline]
    pub async fn fill_more_async(&mut self) -> Result<bool> {
        self.input.fill_more_async().await
    }

    /// Refills until at least count unread units are available.
    ///
    /// # Parameters
    ///
    /// - `count`: Minimum desired unread unit count; grows the buffer if
    ///   needed.
    ///
    /// # Returns
    ///
    /// Returns true when count units are available, or false when EOF occurs
    /// first.
    ///
    /// # Errors
    ///
    /// Returns allocation, input, or buffer-management errors while refilling.
    pub async fn fill_until_async(&mut self, count: usize) -> Result<bool> {
        if count > self.input.capacity() {
            self.input.try_reserve_capacity(count).map_err(allocation_error)?;
        }
        self.input.fill_until_async(count).await
    }

    /// Polls one cancellation-safe decoder operation.
    ///
    /// The operation first refills only when no unread unit is buffered. After
    /// a decoder call changes either the decoder or the buffered input, it
    /// immediately returns [`AsyncTranscodeDecodeStep::Progress`] and never
    /// polls the input again. This makes the returned progress the commit
    /// boundary for cancellation: resume with the adapter's current state,
    /// rather than replaying the previous source range.
    ///
    /// # Type Parameters
    ///
    /// - `D`: Stateful decoder from source units into values.
    /// - `M`: Callback mapping decoder errors to I/O errors.
    /// - `Value`: Destination value type.
    ///
    /// # Parameters
    ///
    /// - `cx`: Task context used when polling the source.
    /// - `decoder`: Decoder whose state may change during this operation.
    /// - `map_error`: Callback invoked for decoder errors.
    /// - `output`: Destination value slice.
    /// - `output_index`: First destination index.
    /// - `count`: Number of writable destination positions.
    ///
    /// # Returns
    ///
    /// `Pending` only while awaiting source input before decoding; otherwise
    /// ready progress or end-of-input. The bridge has already applied the
    /// returned `read()` count, which may be zero; do not consume it again.
    /// Both progress counts may be zero. Follow the stop status before
    /// retrying. A zero `count` returns zero progress without calling the
    /// decoder and does not establish EOF.
    ///
    /// # Errors
    ///
    /// Returns input, allocation, output-range, invalid-progress, or mapped
    /// decoder errors.
    /// Successful validated progress commits its `read()` count. If decoding
    /// or progress validation fails, this call does not consume the unread
    /// range. The decoder and destination may already have changed; neither is
    /// rolled back, and no complete progress report is available. A refill
    /// performed before the failure is not rolled back at the wrapped input.
    /// Cancellation safety applies only to cancellation while this method is
    /// pending; it does not imply rollback after an error. Do not replay the
    /// original input unless the decoder defines an explicit recovery protocol.
    pub fn poll_transcode<D, M, Value>(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        decoder: &mut D,
        map_error: &mut M,
        output: &mut [Value],
        output_index: usize,
        count: usize,
    ) -> Poll<Result<AsyncTranscodeDecodeStep>>
    where
        D: Transcoder<Input = I::Item, Output = Value>,
        M: FnMut(D::Error) -> Error,
    {
        let output_end = SliceRange::checked_range_end(
            output.len(),
            output_index,
            count,
            "decoded output range exceeds destination buffer",
        )?;
        if count == 0 {
            return Poll::Ready(Ok(AsyncTranscodeDecodeStep::Progress(TranscodeProgress::complete(
                0, 0,
            ))));
        }
        let output = &mut output[..output_end];
        if self.as_ref().get_ref().unread_len() == 0 {
            let this = self.as_mut().get_mut();
            match Pin::new(&mut this.input).poll_fill_more(cx) {
                Poll::Ready(Ok(true)) => {}
                Poll::Ready(Ok(false)) => {
                    return Poll::Ready(Ok(AsyncTranscodeDecodeStep::EndOfInput));
                }
                Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
                Poll::Pending => return Poll::Pending,
            }
        }
        let this = self.as_mut().get_mut();
        let available_input = this.unread_len();
        let progress = decoder
            .transcode(this.unread(), 0, output, output_index)
            .map_err(&mut *map_error)
            .and_then(|progress| validate_decode_progress(progress, 0, available_input, output_index, count));
        let progress = match progress {
            Ok(progress) => progress,
            Err(error) => return Poll::Ready(Err(error)),
        };
        this.consume(progress.read());
        Poll::Ready(Ok(AsyncTranscodeDecodeStep::Progress(progress)))
    }

    /// Performs one decode step after the caller has established EOF.
    ///
    /// This method performs no I/O. It passes the current unread buffer to
    /// [`Transcoder::transcode_eof`], validates progress, and commits consumed
    /// source units before returning.
    ///
    /// # Type Parameters
    ///
    /// - `D`: Stateful decoder from source units into values.
    /// - `M`: Callback mapping decoder errors to I/O errors.
    /// - `Value`: Destination value type.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Decoder whose state may change during this operation.
    /// - `map_error`: Callback invoked for decoder errors.
    /// - `output`: Destination value slice.
    /// - `output_index`: First destination index.
    /// - `count`: Number of writable destination positions.
    ///
    /// # Returns
    ///
    /// Validated progress with source consumption committed. A zero output
    /// count returns zero progress without calling the decoder; an empty input
    /// is still dispatched to `transcode_eof`.
    ///
    /// # Errors
    ///
    /// Returns invalid output-range errors, decoder errors mapped by
    /// `map_error`, or `InvalidData` if the decoder requests more input after
    /// EOF. Invalid progress does not commit source consumption, but decoder
    /// state and caller-owned output are not rolled back.
    pub fn transcode_eof_step<D, M, Value>(
        &mut self,
        decoder: &mut D,
        map_error: &mut M,
        output: &mut [Value],
        output_index: usize,
        count: usize,
    ) -> Result<TranscodeProgress>
    where
        D: Transcoder<Input = I::Item, Output = Value>,
        M: FnMut(D::Error) -> Error,
    {
        let output_end = SliceRange::checked_range_end(
            output.len(),
            output_index,
            count,
            "decoded EOF output range exceeds destination buffer",
        )?;
        if count == 0 {
            return Ok(TranscodeProgress::complete(0, 0));
        }
        let available_input = self.unread_len();
        let progress = decoder
            .transcode_eof(self.unread(), 0, &mut output[..output_end], output_index)
            .map_err(&mut *map_error)
            .and_then(|progress| validate_eof_decode_progress(progress, 0, available_input, output_index, count))?;
        self.consume(progress.read());
        Ok(progress)
    }

    /// Decodes one cancellation-safe progress step into an indexed output
    /// range.
    ///
    /// This is the async wrapper for [`Self::poll_transcode`]. It returns after
    /// one decoder invocation; callers that require a complete destination
    /// range must drive successive steps and retain each returned progress.
    ///
    /// # Type Parameters
    ///
    /// - `D`: Stateful decoder from source units into values.
    /// - `M`: Callback mapping decoder errors to I/O errors.
    /// - `Value`: Destination value type.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Decoder whose state may change during this operation.
    /// - `map_error`: Callback invoked for decoder errors.
    /// - `output`: Destination value slice.
    /// - `output_index`: First destination index.
    /// - `count`: Number of writable destination positions.
    ///
    /// # Returns
    ///
    /// One committed progress step, or end-of-input when the source is
    /// exhausted.
    ///
    /// # Errors
    ///
    /// Returns input, allocation, output-range, invalid-progress, or mapped
    /// decoder errors from `poll_transcode`.
    #[inline]
    pub async fn transcode_async<D, M, Value>(
        &mut self,
        decoder: &mut D,
        map_error: &mut M,
        output: &mut [Value],
        output_index: usize,
        count: usize,
    ) -> Result<AsyncTranscodeDecodeStep>
    where
        D: Transcoder<Input = I::Item, Output = Value>,
        M: FnMut(D::Error) -> Error,
    {
        poll_fn(|cx| Pin::new(&mut *self).poll_transcode(cx, decoder, map_error, output, output_index, count)).await
    }
}

impl<I> AsyncInput for AsyncTranscodeDecodeInput<I>
where
    I: AsyncInput,
    I::Item: Copy + Default,
{
    /// Item type read through the retained unit buffer.
    type Item = I::Item;

    /// Reports that this input retains unread units.
    ///
    /// # Returns
    ///
    /// Always `true`, because this adapter retains unread units.
    #[inline]
    fn is_buffered(&self) -> bool {
        true
    }

    /// Polls one read through the retained unit buffer.
    ///
    /// # Parameters
    ///
    /// - `cx`: Task context used to poll the buffered source.
    /// - `output`: Destination unit slice.
    /// - `index`: First destination index.
    /// - `count`: Maximum number of units to read.
    ///
    /// # Returns
    ///
    /// `Pending` while awaiting source data, or a ready read count; zero
    /// indicates EOF.
    ///
    /// # Errors
    ///
    /// Returns errors from the buffered source or its buffer management.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `index..index + count` is a valid range
    /// inside `output` and that the addition does not overflow.
    unsafe fn poll_read_unchecked(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        output: &mut [Self::Item],
        index: usize,
        count: usize,
    ) -> Poll<Result<usize>> {
        // SAFETY: Pinning this wrapper pins its buffered input field.
        let input = unsafe { Pin::new_unchecked(&mut self.as_mut().get_unchecked_mut().input) };
        // SAFETY: The caller upholds the delegated indexed-read contract.
        unsafe { input.poll_read_unchecked(cx, output, index, count) }
    }
}

impl<I> fmt::Debug for AsyncTranscodeDecodeInput<I>
where
    I: AsyncInput,
    I::Item: Copy + Default,
    AsyncBufferedInput<I>: fmt::Debug,
{
    /// Formats this asynchronous decoder input for debugging.
    ///
    /// # Parameters
    ///
    /// - `formatter`: Debug output destination.
    ///
    /// # Returns
    ///
    /// The result of formatting the buffered input.
    ///
    /// # Errors
    ///
    /// Returns a formatting error if the destination rejects output.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AsyncTranscodeDecodeInput")
            .field("input", &self.input)
            .finish()
    }
}
