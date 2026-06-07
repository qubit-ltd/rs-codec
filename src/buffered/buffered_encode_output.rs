// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Output adapter that encodes values into buffered units.

use core::{fmt, marker::PhantomData};
use std::io::{Error, ErrorKind, Result};

use qubit_io::{BufferedOutput, Output};

use super::{BufferedTranscoder, FinishError, TranscodeStatus};

/// Encodes an [`Output`] value stream into an [`Output`] unit stream.
///
/// This adapter owns a unit-level [`qubit_io::BufferedOutput`] and a buffered
/// encoder. Writes to this adapter drive the encoder from caller-provided
/// values into the internal unit buffer, flushing that unit buffer when more
/// capacity is needed. Call [`Self::finish`] after the last value when the
/// encoder may emit retained final units such as reset bytes, digests, or
/// trailers.
///
/// [`Output::flush`] only drains already buffered units. [`Self::finish`] closes
/// the encoder's logical stream under the [`BufferedTranscoder`] lifecycle.
/// After finishing, callers must use a reset or new encoder state before writing
/// another logical stream; this adapter does not expose reset and does not add a
/// closed-state branch to the write hot path.
///
/// # Type Parameters
///
/// * `O` - Wrapped unit output.
/// * `E` - Buffered encoder from `Value` to the wrapped output item.
/// * `M` - Error mapping function from encoder errors to I/O errors.
/// * `Value` - Logical value type accepted by this output.
pub struct BufferedEncodeOutput<O, E, M, Value>
where
    O: Output,
    O::Item: Copy + Default,
{
    output: BufferedOutput<O>,
    encoder: E,
    map_error: M,
    marker: PhantomData<fn(Value)>,
}

impl<O, E, M, Value> fmt::Debug for BufferedEncodeOutput<O, E, M, Value>
where
    O: Output,
    O::Item: Copy + Default,
    BufferedOutput<O>: fmt::Debug,
    E: fmt::Debug,
    M: fmt::Debug,
{
    /// Formats this buffered encode output for debugging.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BufferedEncodeOutput")
            .field("output", &self.output)
            .field("encoder", &self.encoder)
            .field("map_error", &self.map_error)
            .finish()
    }
}

impl<O, E, M, Value> BufferedEncodeOutput<O, E, M, Value>
where
    O: Output,
    O::Item: Copy + Default,
{
    /// Creates an encoder output with a unit buffer of at least `capacity`.
    ///
    /// # Parameters
    ///
    /// * `inner` - Unit output written by this adapter.
    /// * `encoder` - Buffered encoder that converts values into units.
    /// * `capacity` - Requested internal unit buffer capacity.
    /// * `map_error` - Function that maps encoder errors into I/O errors.
    ///
    /// # Returns
    ///
    /// A new buffered encoder output.
    #[must_use]
    #[inline]
    pub fn with_capacity(inner: O, encoder: E, capacity: usize, map_error: M) -> Self {
        Self {
            output: BufferedOutput::with_capacity(inner, capacity),
            encoder,
            map_error,
            marker: PhantomData,
        }
    }

    /// Returns a shared reference to the wrapped unit output.
    ///
    /// # Returns
    ///
    /// A shared reference to the wrapped unit output.
    #[must_use]
    #[inline(always)]
    pub const fn inner(&self) -> &O {
        self.output.inner()
    }

    /// Returns a shared reference to the buffered encoder.
    ///
    /// # Returns
    ///
    /// A shared reference to the buffered encoder.
    #[must_use]
    #[inline(always)]
    pub const fn encoder(&self) -> &E {
        &self.encoder
    }

    /// Consumes this adapter and returns its parts.
    ///
    /// # Returns
    ///
    /// The wrapped output, pending units, encoder, and error mapper.
    #[must_use]
    #[inline]
    pub fn into_parts(self) -> (O, Vec<O::Item>, E, M) {
        let (inner, pending) = self.output.into_parts();
        (inner, pending, self.encoder, self.map_error)
    }
}

impl<O, E, M, Value> BufferedEncodeOutput<O, E, M, Value>
where
    O: Output,
    E: BufferedTranscoder<Value, O::Item>,
    M: FnMut(E::Error) -> Error,
    O::Item: Copy + Default,
{
    /// Finishes the encoder into the internal unit buffer.
    ///
    /// # Errors
    ///
    /// Returns capacity, encoder, or output errors.
    fn finish_encoder(&mut self) -> Result<()> {
        let required = self
            .encoder
            .max_finish_output_len()
            .map_err(capacity_to_io_error)?;
        self.output.ensure_spare_capacity(required)?;
        let (units, unit_index, available) = self.output.spare_raw_parts_mut();
        debug_assert!(available >= required, "insufficient finish capacity");
        let written = self
            .encoder
            .finish(units, unit_index)
            .map_err(|error| finish_to_io_error(error, &mut self.map_error))?;
        assert!(written <= required, "finish wrote beyond its bound");
        // SAFETY: The encoder reported initialized units within the spare
        // range that was reserved above.
        unsafe {
            self.output.advance_unchecked(written);
        }
        Ok(())
    }

    /// Finishes the encoder and flushes the wrapped unit output.
    ///
    /// Call this after the last logical value has been written. Ordinary
    /// [`Output::flush`] calls only drain buffered units and do not close the
    /// encoder stream. After this method succeeds, further writes are caller
    /// misuse unless the encoder state is reset outside this adapter.
    ///
    /// # Errors
    ///
    /// Returns capacity, encoder finalization, or wrapped output flush errors.
    pub fn finish(&mut self) -> Result<()> {
        self.finish_encoder()?;
        self.output.flush()
    }
}

impl<O, E, M, Value> Output for BufferedEncodeOutput<O, E, M, Value>
where
    O: Output,
    E: BufferedTranscoder<Value, O::Item>,
    M: FnMut(E::Error) -> Error,
    O::Item: Copy + Default,
{
    type Item = Value;

    /// Encodes values from an indexed input range.
    unsafe fn write_unchecked(
        &mut self,
        input: &[Value],
        input_index: usize,
        count: usize,
    ) -> Result<usize> {
        debug_assert!(
            input_index
                .checked_add(count)
                .is_some_and(|end| end <= input.len()),
            "unchecked encode input range exceeds source buffer",
        );
        if count == 0 {
            return Ok(0);
        }
        let input_end = input_index + count;
        let input = &input[..input_end];
        let mut read_total = 0;
        while read_total < count {
            if self.output.spare_capacity() == 0 {
                self.output.flush_buffer()?;
            }
            let (units, unit_index, available) = self.output.spare_raw_parts_mut();
            let units = &mut units[..unit_index + available];
            let progress = self
                .encoder
                .transcode(input, input_index + read_total, units, unit_index)
                .map_err(&mut self.map_error)?;
            let read = progress.read();
            let written = progress.written();
            // SAFETY: The encoder reported initialized units in the spare
            // output window.
            unsafe {
                self.output.advance_unchecked(written);
            }
            read_total += read;
            match progress.status() {
                TranscodeStatus::Complete => return Ok(read_total),
                TranscodeStatus::NeedOutput {
                    additional,
                    available,
                    ..
                } => {
                    let required = available + additional.get();
                    if read == 0 && written == 0 {
                        self.output.ensure_spare_capacity(required)?;
                    } else {
                        self.output.flush_buffer()?;
                    }
                }
                TranscodeStatus::NeedInput { .. } => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "encoder unexpectedly requested more input",
                    ));
                }
            }
        }
        Ok(read_total)
    }

    /// Flushes buffered units without finishing the encoder stream.
    fn flush(&mut self) -> Result<()> {
        self.output.flush()
    }
}

/// Converts a capacity planning failure into an I/O error.
fn capacity_to_io_error(error: super::CapacityError) -> Error {
    Error::new(ErrorKind::InvalidData, error)
}

/// Converts a finish failure into an I/O error.
fn finish_to_io_error<E, M>(error: FinishError<E>, map_error: &mut M) -> Error
where
    M: FnMut(E) -> Error,
{
    match error {
        FinishError::Capacity { source } => capacity_to_io_error(source),
        FinishError::InvalidOutputIndex { index, len } => Error::new(
            ErrorKind::InvalidData,
            format!("invalid finish output index {index} for output length {len}"),
        ),
        FinishError::InsufficientOutput {
            output_index,
            required,
            available,
        } => Error::new(
            ErrorKind::InvalidData,
            format!(
                "insufficient finish output at index {output_index}: required {required} units, available {available}"
            ),
        ),
        FinishError::Source { source } => map_error(source),
    }
}
