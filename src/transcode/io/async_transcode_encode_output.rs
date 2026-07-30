// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Buffered asynchronous output driver for streaming transcoders.

use core::{
    fmt,
    num::NonZeroUsize,
};
use std::io::{
    Error,
    ErrorKind,
    Result,
};

use qubit_io::{
    AsyncBufferedOutput,
    AsyncOutput,
    Buffer,
    UncheckedSlice,
};

use crate::{
    CapacityError,
    TranscodeEncoder,
    Transcoder,
};

use super::transcode_progress_driver::{
    EncodeStep,
    encode_progress,
};

/// Buffers asynchronous transcoder output while preserving pending units.
///
/// This adapter is the asynchronous counterpart to
/// [`super::TranscodeEncodeOutput`]. It owns the unit-level buffer, so a
/// future cancelled while the wrapped output is pending retains every unit
/// that the transcoder has already produced. Callers can retry the same
/// lifecycle operation to resume delivery without re-running completed output
/// phases.
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
        }
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
    pub async fn reset_async<E, M, Value>(
        &mut self,
        encoder: &mut E,
        map_error: &mut M,
    ) -> Result<()>
    where
        E: Transcoder<Input = Value, Output = O::Item>,
        M: FnMut(E::Error) -> Error,
    {
        let required = encoder
            .max_reset_output_len()
            .map_err(capacity_error_to_invalid_data)?;
        self.ensure_spare_capacity_async(required).await?;
        let (units, output_index, available) =
            self.output.spare_raw_parts_mut();
        debug_assert!(available >= required);
        let written = encoder
            .reset(units, output_index)
            .map_err(&mut *map_error)?;
        assert!(written <= required, "reset wrote beyond its bound");
        // SAFETY: `written` is bounded by the reserved spare range above.
        unsafe {
            self.output.advance(written);
        }
        Ok(())
    }

    /// Transcodes an indexed input range into the retained asynchronous buffer.
    ///
    /// The adapter grows its persistent unit buffer when the encoder reports a
    /// larger `NeedOutput` requirement, and it delivers pending output before
    /// retrying. It never exposes a `NeedInput` result from an encoder because
    /// the complete caller-supplied input range is already available.
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
    /// Returns the number of source values consumed.
    ///
    /// # Errors
    ///
    /// Returns invalid input ranges, output delivery, allocation, contract, or
    /// mapped transcoder errors. Pending units remain retained after a failed
    /// asynchronous delivery.
    pub async fn transcode_async<E, M, Value>(
        &mut self,
        encoder: &mut E,
        map_error: &mut M,
        input: &[Value],
        input_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        E: TranscodeEncoder<Input = Value, Output = O::Item>,
        M: FnMut(E::Error) -> Error,
    {
        let input_end = UncheckedSlice::checked_range_end(
            input.len(),
            input_index,
            count,
            "encode input range exceeds source buffer",
        )?;
        if count == 0 {
            return Ok(0);
        }
        let input = &input[..input_end];
        let mut read_total = 0;
        let mut required_spare = NonZeroUsize::MIN;
        while read_total < count {
            self.ensure_spare_capacity_async(required_spare.get())
                .await?;
            let (units, output_index, available_output) =
                self.output.spare_raw_parts_mut();
            debug_assert!(available_output >= required_spare.get());
            let remaining_input = count - read_total;
            let progress = encoder
                .transcode(input, input_index + read_total, units, output_index)
                .map_err(&mut *map_error)?;
            let progress = encode_progress(
                progress,
                input_index + read_total,
                remaining_input,
                output_index,
                available_output,
            )?;
            let read = progress.read;
            let written = progress.written;
            // SAFETY: Progress validation proves the initialized output count
            // fits the current spare range.
            unsafe {
                self.output.advance(written);
            }
            read_total += read;
            match progress.step {
                EncodeStep::Complete => return Ok(read_total),
                EncodeStep::NeedOutput(required) => {
                    required_spare = required;
                    if read_total == count {
                        self.ensure_spare_capacity_async(required.get())
                            .await?;
                    }
                }
            }
        }
        Ok(read_total)
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
    pub async fn finish_async<E, M, Value>(
        &mut self,
        encoder: &mut E,
        map_error: &mut M,
    ) -> Result<()>
    where
        E: Transcoder<Input = Value, Output = O::Item>,
        M: FnMut(E::Error) -> Error,
    {
        let required = encoder
            .max_finish_output_len()
            .map_err(capacity_error_to_invalid_data)?;
        self.ensure_spare_capacity_async(required).await?;
        let (units, output_index, available) =
            self.output.spare_raw_parts_mut();
        debug_assert!(available >= required);
        let written = encoder
            .finish(units, output_index)
            .map_err(&mut *map_error)?;
        assert!(written <= required, "finish wrote beyond its bound");
        // SAFETY: `written` is bounded by the reserved spare range above.
        unsafe {
            self.output.advance(written);
        }
        Ok(())
    }

    /// Reserves enough total buffer capacity and makes `count` spare slots
    /// available for one transcoder operation.
    async fn ensure_spare_capacity_async(
        &mut self,
        count: usize,
    ) -> Result<()> {
        let required_capacity = self.output.pending_len().saturating_add(count);
        self.output
            .try_reserve_capacity(required_capacity)
            .map_err(allocation_to_io_error)?;
        self.output.ensure_spare_capacity_async(count).await
    }
}

/// Converts allocation failures into errors at the asynchronous I/O boundary.
fn allocation_to_io_error(error: std::collections::TryReserveError) -> Error {
    Error::new(ErrorKind::OutOfMemory, error)
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
