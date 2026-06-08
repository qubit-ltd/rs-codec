// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Buffered input driver that decodes units into values.

use core::fmt;
use std::io::{
    Error,
    ErrorKind,
    Read,
    Result,
    Seek,
    SeekFrom,
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
/// This type owns only the unit-level [`qubit_io::BufferedInput`]. Callers pass
/// the decoder and error mapper to each decode operation, which lets one
/// buffered input drive different decoders without nesting buffers or storing
/// codec-specific state in the buffer owner.
///
/// Decoding does not finish the decoder automatically at clean EOF. Decoder
/// finish state belongs to the caller-owned decoder, so callers that need
/// final output must call [`Self::finish_into`] or
/// [`Self::finish_into_unchecked`] exactly when their logical stream ends.
/// Incomplete tails remain buffered when EOF prevents a
/// [`TranscodeStatus::NeedInput`] request from being satisfied; the caller can
/// then decide whether to reject, replace, or otherwise handle the tail.
///
/// # Type Parameters
///
/// * `I` - Wrapped unit input.
pub struct BufferedDecodeInput<I>
where
    I: Input,
    I::Item: Copy + Default,
{
    input: BufferedInput<I>,
}

impl<I> fmt::Debug for BufferedDecodeInput<I>
where
    I: Input,
    I::Item: Copy + Default,
    BufferedInput<I>: fmt::Debug,
{
    /// Formats this buffered decode input for debugging.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BufferedDecodeInput")
            .field("input", &self.input)
            .finish()
    }
}

impl<I> BufferedDecodeInput<I>
where
    I: Input,
    I::Item: Copy + Default,
{
    /// Creates a decoder input with the default unit buffer capacity.
    ///
    /// # Parameters
    ///
    /// * `inner` - Unit input read by this adapter.
    ///
    /// # Returns
    ///
    /// A new buffered decoder input.
    #[must_use]
    #[inline]
    pub fn new(inner: I) -> Self {
        Self {
            input: BufferedInput::new(inner),
        }
    }

    /// Creates a decoder input with a unit buffer of at least `capacity`.
    ///
    /// # Parameters
    ///
    /// * `inner` - Unit input read by this adapter.
    /// * `capacity` - Requested internal unit buffer capacity.
    ///
    /// # Returns
    ///
    /// A new buffered decoder input.
    #[must_use]
    #[inline]
    pub fn with_capacity(inner: I, capacity: usize) -> Self {
        Self {
            input: BufferedInput::with_capacity(inner, capacity),
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

    /// Returns a mutable reference to the wrapped unit input.
    ///
    /// # Returns
    ///
    /// A mutable reference to the wrapped unit input.
    #[inline(always)]
    pub fn inner_mut(&mut self) -> &mut I {
        self.input.inner_mut()
    }

    /// Returns the number of unread units currently buffered.
    ///
    /// # Returns
    ///
    /// The number of buffered units available before reading from the wrapped
    /// input again.
    #[must_use]
    #[inline(always)]
    pub fn available(&self) -> usize {
        self.input.available()
    }

    /// Consumes all currently buffered unread units.
    ///
    /// # Returns
    ///
    /// The number of units discarded from the unread buffer.
    #[inline]
    pub fn consume_available(&mut self) -> usize {
        let available = self.input.available();
        self.input.consume(available);
        available
    }

    /// Consumes `count` buffered unread units.
    ///
    /// # Parameters
    ///
    /// * `count` - Number of currently buffered units to consume.
    ///
    /// # Panics
    ///
    /// Panics when `count` exceeds [`Self::available`].
    #[inline]
    pub fn consume_units(&mut self, count: usize) {
        assert!(
            count <= self.input.available(),
            "cannot consume beyond available buffered input",
        );
        self.input.consume(count);
    }

    /// Reads raw units through the internal buffer.
    ///
    /// # Parameters
    ///
    /// * `output` - Destination unit storage.
    ///
    /// # Returns
    ///
    /// The number of raw units read.
    ///
    /// # Errors
    ///
    /// Returns errors reported by the wrapped input.
    #[inline]
    pub fn read_units(&mut self, output: &mut [I::Item]) -> Result<usize> {
        // SAFETY: The full output slice is a valid destination range.
        unsafe { self.read_units_unchecked(output, 0, output.len()) }
    }

    /// Returns raw unread-buffer parts for hot-path callers.
    ///
    /// The returned tuple contains the full internal backing storage, the start
    /// index of unread units, and the unread unit count.
    #[inline(always)]
    #[must_use]
    pub fn unread_raw_parts(&self) -> (&[I::Item], usize, usize) {
        let (units, unit_index, available) = self.input.unread_raw_parts();
        debug_assert!(unit_index <= units.len());
        debug_assert!(unit_index + available <= units.len());
        (&units[..unit_index + available], unit_index, available)
    }

    /// Refills the internal buffer until at least `count` unread units are
    /// available, returning whether the requested amount was reached.
    ///
    /// # Parameters
    ///
    /// * `count` - Minimum unread units required.
    ///
    /// # Returns
    ///
    /// `Ok(true)` if at least `count` unread units are available, or
    /// `Ok(false)` at EOF.
    ///
    /// # Errors
    ///
    /// Returns any non-interrupted I/O error reported by the wrapped input.
    #[inline]
    pub fn fill_until(&mut self, count: usize) -> Result<bool> {
        self.input.fill_until(count)
    }

    /// Reads raw units through the internal buffer into an indexed range.
    ///
    /// # Parameters
    ///
    /// * `output` - Destination unit storage.
    /// * `output_index` - Start index inside `output`.
    /// * `count` - Maximum number of units to read.
    ///
    /// # Returns
    ///
    /// The number of raw units read.
    ///
    /// # Errors
    ///
    /// Returns errors reported by the wrapped input.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `output_index..output_index + count` is
    /// a valid range inside `output` and that the addition does not overflow.
    #[inline]
    pub unsafe fn read_units_unchecked(
        &mut self,
        output: &mut [I::Item],
        output_index: usize,
        count: usize,
    ) -> Result<usize> {
        // SAFETY: The caller guarantees that the destination range is valid.
        unsafe { self.input.read_into_unchecked(output, output_index, count) }
    }

    /// Consumes this adapter and returns its parts.
    ///
    /// # Returns
    ///
    /// The wrapped input and unread units.
    #[must_use]
    #[inline]
    pub fn into_parts(self) -> (I, Vec<I::Item>) {
        self.input.into_parts()
    }
}

impl<I> BufferedDecodeInput<I>
where
    I: Input<Item = u8> + Seek,
{
    /// Seeks the wrapped byte input and discards buffered bytes after success.
    ///
    /// # Parameters
    ///
    /// * `position` - Target seek position.
    ///
    /// # Returns
    ///
    /// The new stream position reported by the wrapped input.
    ///
    /// # Errors
    ///
    /// Returns seek errors from the wrapped input.
    #[inline]
    pub fn seek(&mut self, position: SeekFrom) -> Result<u64> {
        Seek::seek(&mut self.input, position)
    }
}

impl<I> Read for BufferedDecodeInput<I>
where
    I: Input<Item = u8>,
{
    /// Reads raw bytes through the internal buffer.
    #[inline]
    fn read(&mut self, output: &mut [u8]) -> Result<usize> {
        self.read_units(output)
    }
}

impl<I> Seek for BufferedDecodeInput<I>
where
    I: Input<Item = u8> + Seek,
{
    /// Seeks the wrapped byte input and discards buffered bytes after success.
    #[inline]
    fn seek(&mut self, position: SeekFrom) -> Result<u64> {
        self.seek(position)
    }
}

impl<I> BufferedDecodeInput<I>
where
    I: Input,
    I::Item: Copy + Default,
{
    /// Decodes values into a checked output range.
    ///
    /// # Parameters
    ///
    /// * `decoder` - Decoder used for this operation.
    /// * `map_error` - Function mapping decoder errors into I/O errors.
    /// * `output` - Destination value storage.
    /// * `output_index` - Start index inside `output`.
    /// * `count` - Maximum number of values to write.
    ///
    /// # Returns
    ///
    /// The number of values written. A zero result means either `count` was
    /// zero, clean EOF was reached, or an incomplete tail remains buffered at
    /// EOF.
    ///
    /// # Errors
    ///
    /// Returns input errors, capacity errors from the internal buffer, or
    /// decoder errors mapped by `map_error`.
    pub fn decode_into<D, M, Value>(
        &mut self,
        decoder: &mut D,
        map_error: &mut M,
        output: &mut [Value],
        output_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        D: BufferedTranscoder<I::Item, Value>,
        M: FnMut(D::Error) -> Error,
    {
        assert!(
            output_index
                .checked_add(count)
                .is_some_and(|end| end <= output.len()),
            "decoded output range exceeds destination buffer",
        );
        // SAFETY: The assertion proves that the requested output range is
        // valid.
        unsafe {
            self.decode_into_unchecked(
                decoder,
                map_error,
                output,
                output_index,
                count,
            )
        }
    }

    /// Decodes values into an indexed output range without checking bounds.
    ///
    /// # Parameters
    ///
    /// * `decoder` - Decoder used for this operation.
    /// * `map_error` - Function mapping decoder errors into I/O errors.
    /// * `output` - Destination value storage.
    /// * `output_index` - Start index inside `output`.
    /// * `count` - Maximum number of values to write.
    ///
    /// # Returns
    ///
    /// The number of values written. Incomplete EOF tails are left buffered
    /// and reported as `Ok(written)`, so callers can apply their own EOF
    /// policy.
    ///
    /// # Errors
    ///
    /// Returns input errors, capacity errors from the internal buffer, or
    /// decoder errors mapped by `map_error`.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `output_index..output_index + count` is
    /// a valid range inside `output` and that the addition does not overflow.
    pub unsafe fn decode_into_unchecked<D, M, Value>(
        &mut self,
        decoder: &mut D,
        map_error: &mut M,
        output: &mut [Value],
        output_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        D: BufferedTranscoder<I::Item, Value>,
        M: FnMut(D::Error) -> Error,
    {
        debug_assert!(
            output_index
                .checked_add(count)
                .is_some_and(|end| end <= output.len()),
            "unchecked decoded output range exceeds destination buffer",
        );
        if count == 0 {
            return Ok(0);
        }
        let output_end = output_index + count;
        let output = &mut output[..output_end];
        let mut written_total = 0;
        loop {
            if self.input.available() == 0 && !self.input.fill_more()? {
                return Ok(written_total);
            }
            let (units, unit_index, available) = self.input.unread_raw_parts();
            let units = &units[..unit_index + available];
            let progress = decoder
                .transcode(
                    units,
                    unit_index,
                    output,
                    output_index + written_total,
                )
                .map_err(&mut *map_error)?;
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
                    return Ok(written_total);
                }
            }
        }
    }

    /// Finishes the decoder into the caller-provided output slice.
    ///
    /// The caller-provided output range must be able to accept the decoder's
    /// advertised finish bound.
    ///
    /// # Parameters
    ///
    /// * `decoder` - Decoder whose final output is being collected.
    /// * `map_error` - Function mapping decoder errors into I/O errors.
    /// * `output` - Destination value storage.
    /// * `output_index` - Start index inside `output`.
    /// * `count` - Maximum number of finish values to write.
    ///
    /// # Returns
    ///
    /// The number of values written by the decoder finish operation.
    ///
    /// # Errors
    ///
    /// Returns capacity or decoder finalization errors mapped to I/O errors.
    pub fn finish_into<D, M, Value>(
        &mut self,
        decoder: &mut D,
        map_error: &mut M,
        output: &mut [Value],
        output_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        D: BufferedTranscoder<I::Item, Value>,
        M: FnMut(D::Error) -> Error,
    {
        assert!(
            output_index
                .checked_add(count)
                .is_some_and(|end| end <= output.len()),
            "finish output range exceeds destination buffer",
        );
        // SAFETY: The assertion proves that the requested output range is
        // valid.
        unsafe {
            self.finish_into_unchecked(
                decoder,
                map_error,
                output,
                output_index,
                count,
            )
        }
    }

    /// Finishes the decoder into an indexed output range without bounds
    /// checks.
    ///
    /// # Parameters
    ///
    /// * `decoder` - Decoder whose final output is being collected.
    /// * `map_error` - Function mapping decoder errors into I/O errors.
    /// * `output` - Destination value storage.
    /// * `output_index` - Start index inside `output`.
    /// * `count` - Maximum number of finish values to write.
    ///
    /// # Returns
    ///
    /// The number of values written by the decoder finish operation.
    ///
    /// # Errors
    ///
    /// Returns capacity or decoder finalization errors mapped to I/O errors.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `output_index..output_index + count` is
    /// a valid range inside `output` and that the addition does not overflow.
    pub unsafe fn finish_into_unchecked<D, M, Value>(
        &mut self,
        decoder: &mut D,
        map_error: &mut M,
        output: &mut [Value],
        output_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        D: BufferedTranscoder<I::Item, Value>,
        M: FnMut(D::Error) -> Error,
    {
        debug_assert!(
            output_index
                .checked_add(count)
                .is_some_and(|end| end <= output.len()),
            "unchecked finish output range exceeds destination buffer",
        );
        let required = decoder
            .max_finish_output_len()
            .map_err(capacity_to_io_error)?;
        if required > count {
            return Err(finish_to_io_error(
                FinishError::insufficient_output(output_index, required, count),
                map_error,
            ));
        }
        let output_end = output_index + count;
        let output = &mut output[..output_end];
        let written = decoder
            .finish(output, output_index)
            .map_err(|error| finish_to_io_error(error, map_error))?;
        assert!(written <= required, "finish wrote beyond its bound");
        Ok(written)
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
