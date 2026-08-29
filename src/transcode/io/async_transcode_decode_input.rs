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
use std::task::Context;
use std::task::Poll;

use qubit_io::AsyncBufferedInput;
use qubit_io::AsyncInput;
use qubit_io::Buffer;
use qubit_utils::SliceRange;
use qubit_utils::allocation_error;

use super::async_transcode_decode_step::AsyncTranscodeDecodeStep;
use super::transcode_progress_validation::validate_decode_progress;
use crate::CapacityError;
use crate::TranscodeProgress;
use crate::Transcoder;

/// Decodes an asynchronous unit stream into values.
///
/// The adapter owns unit buffering but not decoder state. Callers supply a
/// streaming decoder and error mapper for each operation.
///
/// # Type Parameters
///
/// - `I`: Wrapped asynchronous unit input.
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
    pub fn with_capacity(inner: I, capacity: usize) -> Self {
        Self {
            input: AsyncBufferedInput::with_capacity(inner, capacity),
        }
    }

    /// Tries to create an adapter with an internal unit buffer of at least
    /// capacity.
    ///
    /// # Errors
    ///
    /// Returns the allocation error when the internal buffer cannot be
    /// allocated.
    pub fn try_with_capacity(inner: I, capacity: usize) -> std::result::Result<Self, TryReserveError> {
        Ok(Self {
            input: AsyncBufferedInput::try_with_capacity(inner, capacity)?,
        })
    }

    /// Returns a shared reference to the wrapped asynchronous input.
    ///
    /// The input can be physically positioned after units retained in this
    /// adapter's unread buffer.
    #[must_use]
    pub const fn inner(&self) -> &I {
        self.input.inner()
    }

    /// Returns a mutable reference to the wrapped asynchronous input.
    ///
    /// Direct reads can invalidate the logical stream position represented by
    /// unread buffered units.
    #[must_use]
    pub fn inner_mut(&mut self) -> &mut I {
        self.input.inner_mut()
    }

    /// Returns the number of unread units currently buffered.
    #[must_use]
    pub const fn unread_len(&self) -> usize {
        self.input.unread_len()
    }

    /// Returns the unread buffered unit window.
    #[must_use]
    pub fn unread(&self) -> &[I::Item] {
        self.input.unread()
    }

    /// Returns the total internal unit-buffer capacity.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.input.capacity()
    }

    /// Consumes unread units from the current buffer window.
    ///
    /// # Panics
    ///
    /// Panics when count exceeds the unread unit count.
    pub fn consume(&mut self, count: usize) {
        assert!(count <= self.unread_len(), "cannot consume beyond buffered input",);
        // SAFETY: The asserted bound proves count fits the unread window.
        unsafe {
            self.input.consume(count);
        }
    }

    /// Copies unread units into an indexed output range without consuming them.
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
    #[must_use = "the returned input and unread buffer must be handled"]
    pub fn into_parts(self) -> (I, Buffer<I::Item>) {
        self.input.into_parts()
    }

    /// Runs decoder reset into an indexed output range without I/O.
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
        let required = decoder.max_reset_output_len().map_err(capacity_to_io_error)?;
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
        let required = decoder.max_finish_output_len().map_err(capacity_to_io_error)?;
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
    pub async fn fill_more_async(&mut self) -> Result<bool> {
        self.input.fill_more_async().await
    }

    /// Refills until at least count unread units are available.
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
    /// # Errors
    ///
    /// Returns input, allocation, output-range, invalid-progress, or mapped
    /// decoder errors.
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
    /// # Errors
    ///
    /// Returns invalid output-range errors, or decoder errors mapped by
    /// `map_error`.
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
        if self.unread_len() == 0 {
            return Ok(TranscodeProgress::complete(0, 0));
        }
        let available_input = self.unread_len();
        let progress = decoder
            .transcode_eof(self.unread(), 0, &mut output[..output_end], output_index)
            .map_err(&mut *map_error)
            .and_then(|progress| validate_decode_progress(progress, 0, available_input, output_index, count))?;
        self.consume(progress.read());
        Ok(progress)
    }

    /// Decodes one cancellation-safe progress step into an indexed output
    /// range.
    ///
    /// This is the async wrapper for [`Self::poll_transcode`]. It returns after
    /// one decoder invocation; callers that require a complete destination
    /// range must drive successive steps and retain each returned progress.
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
    fn is_buffered(&self) -> bool {
        true
    }

    /// Polls one read through the retained unit buffer.
    ///
    /// # Safety
    ///
    /// The caller must provide a valid indexed output range.
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
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AsyncTranscodeDecodeInput")
            .field("input", &self.input)
            .finish()
    }
}

/// Converts decoder capacity failures into invalid stream data.
fn capacity_to_io_error(error: CapacityError) -> Error {
    Error::new(ErrorKind::InvalidData, error)
}
