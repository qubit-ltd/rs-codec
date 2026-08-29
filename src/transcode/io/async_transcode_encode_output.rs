// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Buffered asynchronous output driver for streaming transcoders.

use core::fmt;
use std::collections::TryReserveError;
use std::future::poll_fn;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use qubit_io::AsyncBufferedOutput;
use qubit_io::AsyncOutput;
use qubit_io::Buffer;
use qubit_utils::SliceRange;
use qubit_utils::allocation_error;

use super::transcode_progress_validation::validate_encode_progress;
use crate::CapacityError;
use crate::TranscodeEncoder;
use crate::TranscodeProgress;
use crate::TranscodeStatus;
use crate::Transcoder;

/// Buffers asynchronous transcoder output while preserving pending units.
///
/// This adapter is the asynchronous counterpart to
/// [`super::TranscodeEncodeOutput`]. It owns the unit-level buffer, so a
/// future cancelled while the wrapped output is pending retains every unit
/// that the transcoder has already produced. Each transcode operation returns
/// after exactly one encoder invocation, so its progress is the explicit
/// source-consumption commit boundary.
///
/// # Type Parameters
///
/// * `O` - Wrapped asynchronous unit output.
pub struct AsyncTranscodeEncodeOutput<O>
where
    O: AsyncOutput,
    O::Item: Clone + Default,
{
    output: AsyncBufferedOutput<O>,
    required_spare: usize,
}

impl<O> AsyncTranscodeEncodeOutput<O>
where
    O: AsyncOutput,
    O::Item: Clone + Default,
{
    /// Creates an asynchronous encoder output with the default unit capacity.
    ///
    /// # Parameters
    ///
    /// * `inner` - Unit output receiving transcoder data.
    ///
    /// # Returns
    ///
    /// Returns a buffered asynchronous output adapter.
    #[must_use]
    pub fn new(inner: O) -> Self {
        Self {
            output: AsyncBufferedOutput::new(inner),
            required_spare: 1,
        }
    }

    /// Creates an asynchronous encoder output with at least `capacity` units.
    ///
    /// # Parameters
    ///
    /// * `inner` - Unit output receiving transcoder data.
    /// * `capacity` - Requested internal unit capacity.
    ///
    /// # Returns
    ///
    /// Returns a buffered asynchronous output adapter.
    #[must_use]
    pub fn with_capacity(inner: O, capacity: usize) -> Self {
        Self {
            output: AsyncBufferedOutput::with_capacity(inner, capacity),
            required_spare: 1,
        }
    }

    /// Tries to create an adapter with at least `capacity` buffered units.
    ///
    /// # Errors
    ///
    /// Returns an allocation error when the internal unit buffer cannot be
    /// allocated.
    pub fn try_with_capacity(inner: O, capacity: usize) -> std::result::Result<Self, TryReserveError> {
        Ok(Self {
            output: AsyncBufferedOutput::try_with_capacity(inner, capacity)?,
            required_spare: 1,
        })
    }

    /// Returns the total internal unit-buffer capacity.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.output.capacity()
    }

    /// Returns a shared reference to the wrapped asynchronous output.
    ///
    /// # Returns
    ///
    /// Returns the wrapped output. Units can still be pending in this adapter.
    #[must_use]
    pub const fn inner(&self) -> &O {
        self.output.inner()
    }

    /// Returns a mutable reference to the wrapped asynchronous output.
    ///
    /// Pending adapter units are not automatically delivered before direct
    /// output operations. Call [`Self::flush_async`] first when ordering with
    /// previously transcoded data matters.
    ///
    /// # Returns
    ///
    /// Returns the wrapped output.
    pub fn inner_mut(&mut self) -> &mut O {
        self.output.inner_mut()
    }

    /// Returns the number of units retained pending delivery.
    ///
    /// # Returns
    ///
    /// Returns the length of the pending unit window.
    #[must_use]
    pub const fn pending_len(&self) -> usize {
        self.output.pending_len()
    }

    /// Consumes this adapter without flushing the wrapped output.
    ///
    /// This method performs no asynchronous I/O. The returned buffer holds
    /// units that must be delivered before the logical stream can continue.
    ///
    /// # Returns
    ///
    /// Returns the wrapped output and its pending unit buffer.
    #[must_use = "the returned output and pending buffer must be handled"]
    pub fn into_parts(self) -> (O, Buffer<O::Item>) {
        self.output.into_parts()
    }
}

impl<O> AsyncTranscodeEncodeOutput<O>
where
    O: AsyncOutput + Unpin,
    O::Item: Clone + Default + Unpin,
{
    /// Flushes pending transcoded units and the wrapped asynchronous output.
    ///
    /// # Errors
    ///
    /// Returns output-delivery or flush errors from the wrapped output. A
    /// failed delivery retains every unit that was not accepted.
    pub async fn flush_async(&mut self) -> Result<()> {
        self.output.flush_async().await
    }

    /// Delivers pending transcoded units without flushing the wrapped output.
    ///
    /// # Errors
    ///
    /// Returns output-delivery errors from the wrapped output. A failed
    /// delivery retains every unit that was not accepted.
    pub async fn drain_async(&mut self) -> Result<()> {
        let capacity = self.output.capacity();
        self.output.ensure_spare_capacity_async(capacity).await
    }

    /// Runs encoder reset and buffers its stream-prefix units.
    ///
    /// # Parameters
    ///
    /// * `encoder` - Streaming encoder whose lifecycle is being started.
    /// * `map_error` - Maps transcoder errors into I/O errors.
    ///
    /// # Errors
    ///
    /// Returns capacity planning, output delivery, allocation, or mapped reset
    /// errors. Pending units remain retained if asynchronous delivery fails.
    ///
    /// # Panics
    ///
    /// Panics when `encoder` writes more units than its declared reset bound.
    pub async fn reset_async<E, M, Value>(&mut self, encoder: &mut E, map_error: &mut M) -> Result<()>
    where
        E: Transcoder<Input = Value, Output = O::Item>,
        M: FnMut(E::Error) -> Error,
    {
        let required = encoder.max_reset_output_len().map_err(capacity_error_to_invalid_data)?;
        self.ensure_spare_capacity_async(required).await?;
        let (units, output_index, available) = self.output.spare_raw_parts_mut();
        debug_assert!(available >= required);
        let written = encoder.reset(units, output_index).map_err(&mut *map_error)?;
        assert!(written <= required, "reset wrote beyond its bound");
        // SAFETY: `written` is bounded by the reserved spare range above.
        unsafe {
            self.output.advance(written);
        }
        Ok(())
    }

    /// Polls one cancellation-safe encoder operation.
    ///
    /// The adapter may first deliver previously pending output to make a
    /// conservative spare range available. Once it invokes `encoder`, it
    /// advances the produced units and returns immediately without another
    /// poll. The returned [`TranscodeProgress`] is therefore the exact source
    /// range committed by this call.
    ///
    /// # Parameters
    ///
    /// * `encoder` - Streaming encoder used for the conversion.
    /// * `map_error` - Maps transcoder errors into I/O errors.
    /// * `input` - Source values.
    /// * `input_index` - First source value to transcode.
    /// * `count` - Number of source values available from `input_index`.
    ///
    /// # Returns
    ///
    /// Returns one validated encoder progress report.
    ///
    /// # Errors
    ///
    /// Returns invalid input ranges, output delivery, allocation, contract, or
    /// mapped transcoder errors. Pending units remain retained after a failed
    /// asynchronous delivery.
    pub fn poll_transcode<E, M, Value>(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        encoder: &mut E,
        map_error: &mut M,
        input: &[Value],
        input_index: usize,
        count: usize,
    ) -> Poll<Result<TranscodeProgress>>
    where
        E: TranscodeEncoder<Input = Value, Output = O::Item>,
        M: FnMut(E::Error) -> Error,
    {
        let input_end = SliceRange::checked_range_end(
            input.len(),
            input_index,
            count,
            "encode input range exceeds source buffer",
        )?;
        if count == 0 {
            return Poll::Ready(Ok(TranscodeProgress::complete(0, 0)));
        }
        let input = &input[..input_end];
        let this = self.as_mut().get_mut();
        let per_value = match encoder.max_transcode_output_len(1) {
            Ok(required) => required.max(1),
            Err(error) => {
                return Poll::Ready(Err(capacity_error_to_invalid_data(error)));
            }
        };
        let required_spare = this.required_spare.max(per_value);
        let required_capacity = this.output.pending_len().saturating_add(required_spare);
        if let Err(error) = this.output.try_reserve_capacity(required_capacity) {
            return Poll::Ready(Err(allocation_error(error)));
        }
        match Pin::new(&mut this.output).poll_ensure_spare_capacity(cx, required_spare) {
            Poll::Ready(Ok(())) => {}
            Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
            Poll::Pending => return Poll::Pending,
        }
        let (units, output_index, available_output) = this.output.spare_raw_parts_mut();
        let progress = encoder
            .transcode(input, input_index, units, output_index)
            .map_err(&mut *map_error)
            .and_then(|progress| {
                validate_encode_progress(progress, input_index, count, output_index, available_output)
            });
        let progress = match progress {
            Ok(progress) => progress,
            Err(error) => return Poll::Ready(Err(error)),
        };
        // SAFETY: Progress validation proves the initialized output count fits
        // the spare window provided to the encoder.
        unsafe {
            this.output.advance(progress.written());
        }
        this.required_spare = if let TranscodeStatus::NeedOutput { required, .. } = progress.status() {
            required.get()
        } else {
            // `validate_encode_progress` rejects `NeedInput` before this
            // point.
            debug_assert!(progress.is_complete());
            1
        };
        Poll::Ready(Ok(progress))
    }

    /// Transcodes one cancellation-safe progress step into the retained buffer.
    ///
    /// This is the async wrapper for [`Self::poll_transcode`]. It returns after
    /// one encoder invocation; callers that need to consume a whole source
    /// range must advance the source index by [`TranscodeProgress::read`] and
    /// call it again.
    pub async fn transcode_async<E, M, Value>(
        &mut self,
        encoder: &mut E,
        map_error: &mut M,
        input: &[Value],
        input_index: usize,
        count: usize,
    ) -> Result<TranscodeProgress>
    where
        E: TranscodeEncoder<Input = Value, Output = O::Item>,
        M: FnMut(E::Error) -> Error,
    {
        poll_fn(|cx| Pin::new(&mut *self).poll_transcode(cx, encoder, map_error, input, input_index, count)).await
    }

    /// Finishes the encoder and retains its final output for delivery.
    ///
    /// # Parameters
    ///
    /// * `encoder` - Streaming encoder whose lifecycle is being finished.
    /// * `map_error` - Maps transcoder errors into I/O errors.
    ///
    /// # Errors
    ///
    /// Returns capacity planning, allocation, or mapped finish errors. Call
    /// [`Self::flush_async`] after this method to deliver final units. Keeping
    /// finalization separate from delivery lets a caller record the completed
    /// lifecycle before any later suspension point.
    ///
    /// # Panics
    ///
    /// Panics when `encoder` writes more units than its declared finish bound.
    pub async fn finish_async<E, M, Value>(&mut self, encoder: &mut E, map_error: &mut M) -> Result<()>
    where
        E: Transcoder<Input = Value, Output = O::Item>,
        M: FnMut(E::Error) -> Error,
    {
        let required = encoder
            .max_finish_output_len()
            .map_err(capacity_error_to_invalid_data)?;
        self.ensure_spare_capacity_async(required).await?;
        let (units, output_index, available) = self.output.spare_raw_parts_mut();
        debug_assert!(available >= required);
        let written = encoder.finish(units, output_index).map_err(&mut *map_error)?;
        assert!(written <= required, "finish wrote beyond its bound");
        // SAFETY: `written` is bounded by the reserved spare range above.
        unsafe {
            self.output.advance(written);
        }
        Ok(())
    }

    /// Reserves enough total buffer capacity and makes `count` spare slots
    /// available for one transcoder operation.
    async fn ensure_spare_capacity_async(&mut self, count: usize) -> Result<()> {
        let required_capacity = self.output.pending_len().saturating_add(count);
        self.output
            .try_reserve_capacity(required_capacity)
            .map_err(allocation_error)?;
        self.output.ensure_spare_capacity_async(count).await
    }
}

/// Converts capacity planning failures into invalid asynchronous stream data.
fn capacity_error_to_invalid_data(error: CapacityError) -> Error {
    Error::new(ErrorKind::InvalidData, error)
}

impl<O> fmt::Debug for AsyncTranscodeEncodeOutput<O>
where
    O: AsyncOutput,
    O::Item: Clone + Default,
    AsyncBufferedOutput<O>: fmt::Debug,
{
    /// Formats this asynchronous output adapter for debugging.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AsyncTranscodeEncodeOutput")
            .field("output", &self.output)
            .finish()
    }
}
