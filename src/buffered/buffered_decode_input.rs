// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Input adapter that decodes buffered units into values.

use core::{
    fmt,
    marker::PhantomData,
};
use std::io::{
    Error,
    ErrorKind,
    Result,
};

use qubit_io::{
    BufferedInput,
    Input,
};

use super::{
    BufferedTranscoder,
    FinishError,
    TranscodeStatus,
};

/// Decodes an [`Input`] unit stream into an [`Input`] value stream.
///
/// This adapter owns a unit-level [`qubit_io::BufferedInput`] and a buffered
/// decoder. Reads from this adapter fill the internal unit buffer, drive the
/// decoder into the caller-provided value slice, and consume only the source
/// units reported by the decoder progress.
///
/// At clean EOF, the adapter calls the decoder's finish operation exactly once
/// and returns EOF on later reads. If finishing can emit values, the current
/// caller-provided output range must have enough remaining slots for
/// [`BufferedTranscoder::max_finish_output_len`]. The adapter intentionally
/// does not allocate a pending final-output buffer, so finalization keeps the
/// same caller-buffer ownership model as normal transcoding. Incomplete tails
/// reported by the decoder at EOF are returned as [`ErrorKind::UnexpectedEof`]
/// and are not finalized.
///
/// # Type Parameters
///
/// * `I` - Wrapped unit input.
/// * `D` - Buffered decoder from the wrapped input item to `Value`.
/// * `M` - Error mapping function from decoder errors to I/O errors.
/// * `Value` - Decoded value type written to callers.
pub struct BufferedDecodeInput<I, D, M, Value>
where
    I: Input,
    I::Item: Copy + Default,
{
    input: BufferedInput<I>,
    decoder: D,
    map_error: M,
    finished: bool,
    marker: PhantomData<fn() -> Value>,
}

impl<I, D, M, Value> fmt::Debug for BufferedDecodeInput<I, D, M, Value>
where
    I: Input,
    I::Item: Copy + Default,
    BufferedInput<I>: fmt::Debug,
    D: fmt::Debug,
    M: fmt::Debug,
{
    /// Formats this buffered decode input for debugging.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BufferedDecodeInput")
            .field("input", &self.input)
            .field("decoder", &self.decoder)
            .field("map_error", &self.map_error)
            .field("finished", &self.finished)
            .finish()
    }
}

impl<I, D, M, Value> BufferedDecodeInput<I, D, M, Value>
where
    I: Input,
    I::Item: Copy + Default,
{
    /// Creates a decoder input with a unit buffer of at least `capacity`.
    ///
    /// # Parameters
    ///
    /// * `inner` - Unit input read by this adapter.
    /// * `decoder` - Buffered decoder that converts units into values.
    /// * `capacity` - Requested internal unit buffer capacity.
    /// * `map_error` - Function that maps decoder errors into I/O errors.
    ///
    /// # Returns
    ///
    /// A new buffered decoder input.
    #[must_use]
    #[inline]
    pub fn with_capacity(
        inner: I,
        decoder: D,
        capacity: usize,
        map_error: M,
    ) -> Self {
        Self {
            input: BufferedInput::with_capacity(inner, capacity),
            decoder,
            map_error,
            finished: false,
            marker: PhantomData,
        }
    }

    /// Returns a shared reference to the wrapped unit input.
    ///
    /// # Returns
    ///
    /// A shared reference to the wrapped unit input.
    #[must_use]
    #[inline(always)]
    pub const fn inner(&self) -> &I {
        self.input.inner()
    }

    /// Returns a shared reference to the buffered decoder.
    ///
    /// # Returns
    ///
    /// A shared reference to the buffered decoder.
    #[must_use]
    #[inline(always)]
    pub const fn decoder(&self) -> &D {
        &self.decoder
    }

    /// Consumes this adapter and returns its parts.
    ///
    /// # Returns
    ///
    /// The wrapped input, unread units, decoder, and error mapper.
    #[must_use]
    #[inline]
    pub fn into_parts(self) -> (I, Vec<I::Item>, D, M) {
        let (inner, unread) = self.input.into_parts();
        (inner, unread, self.decoder, self.map_error)
    }
}

impl<I, D, M, Value> BufferedDecodeInput<I, D, M, Value>
where
    I: Input,
    D: BufferedTranscoder<I::Item, Value>,
    M: FnMut(D::Error) -> Error,
    I::Item: Copy + Default,
{
    /// Finishes the decoder into the caller-provided output slice.
    ///
    /// Finalization is a one-shot operation. The caller-provided output range
    /// must be able to accept the decoder's advertised finish bound.
    ///
    /// # Errors
    ///
    /// Returns capacity or decoder finalization errors mapped to I/O errors.
    fn finish_decoder(
        &mut self,
        output: &mut [Value],
        output_index: usize,
    ) -> Result<usize> {
        let required = self
            .decoder
            .max_finish_output_len()
            .map_err(capacity_to_io_error)?;
        let available = output.len().saturating_sub(output_index);
        if required > available {
            return Err(finish_to_io_error(
                FinishError::insufficient_output(
                    output_index,
                    required,
                    available,
                ),
                &mut self.map_error,
            ));
        }
        let written = self
            .decoder
            .finish(output, output_index)
            .map_err(|error| finish_to_io_error(error, &mut self.map_error))?;
        assert!(written <= required, "finish wrote beyond its bound");
        self.finished = true;
        Ok(written)
    }
}

impl<I, D, M, Value> Input for BufferedDecodeInput<I, D, M, Value>
where
    I: Input,
    D: BufferedTranscoder<I::Item, Value>,
    M: FnMut(D::Error) -> Error,
    I::Item: Copy + Default,
{
    type Item = Value;

    /// Reads decoded values into an indexed output range.
    unsafe fn read_unchecked(
        &mut self,
        output: &mut [Value],
        output_index: usize,
        count: usize,
    ) -> Result<usize> {
        debug_assert!(
            output_index
                .checked_add(count)
                .is_some_and(|end| end <= output.len()),
            "unchecked decoded output range exceeds destination buffer",
        );
        if count == 0 {
            return Ok(0);
        }
        if self.finished {
            return Ok(0);
        }
        let output_end = output_index + count;
        let output = &mut output[..output_end];
        let mut written_total = 0;
        loop {
            if self.input.available() == 0 && !self.input.fill_more()? {
                if written_total > 0 {
                    return Ok(written_total);
                }
                let written = self.finish_decoder(output, output_index)?;
                return Ok(written);
            }
            let (units, unit_index, available) = self.input.unread_raw_parts();
            let units = &units[..unit_index + available];
            let progress = self
                .decoder
                .transcode(
                    units,
                    unit_index,
                    output,
                    output_index + written_total,
                )
                .map_err(&mut self.map_error)?;
            let consumed = progress.read();
            let written = progress.written();
            // SAFETY: The decoder reported consumed units from the currently
            // unread input window.
            unsafe {
                self.input.consume_unchecked(consumed);
            }
            written_total += written;
            match progress.status() {
                TranscodeStatus::Complete => {
                    if written_total == count || consumed == 0 {
                        return Ok(written_total);
                    }
                }
                TranscodeStatus::NeedOutput { .. } => {
                    return Ok(written_total);
                }
                TranscodeStatus::NeedInput {
                    additional,
                    available,
                    ..
                } => {
                    let required = available + additional.get();
                    if self.input.fill_until(required)? {
                        continue;
                    }
                    if written_total > 0 {
                        return Ok(written_total);
                    }
                    return Err(Error::new(
                        ErrorKind::UnexpectedEof,
                        "incomplete encoded input at EOF",
                    ));
                }
            }
        }
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
            format!(
                "invalid finish output index {index} for output length {len}"
            ),
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
