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

use crate::codec::assert_unit_bounds;
use crate::{
    Codec,
    TranscodeError,
    TranscodeStatus,
    Transcoder,
};

/// Decodes an [`Input`] unit stream into an [`Input`] value stream.
///
/// This type owns only the unit-level [`qubit_io::BufferedInput`]. Callers pass
/// a [`Codec`] and error mapper to each decode operation, which lets one
/// buffered input drive different decoders without nesting buffers or storing
/// codec-specific state in the buffer owner.
///
/// A [`Codec`] has no decoder-owned finish state, so [`Self::finish_into`] is
/// a no-op retained for API symmetry. Callers that need a stateful streaming
/// decoder can use [`Self::transcode_into`] and [`Self::finish_transcode_into`]
/// explicitly.
///
/// # Type Parameters
///
/// * `I` - Wrapped unit input.
pub struct TranscodeDecodeInput<I>
where
    I: Input,
    I::Item: Copy + Default,
{
    input: BufferedInput<I>,
}

impl<I> TranscodeDecodeInput<I>
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
    /// The number of unread units in the internal buffer.
    #[must_use]
    #[inline(always)]
    pub fn available(&self) -> usize {
        self.input.available()
    }

    /// Returns the internal unit buffer capacity.
    ///
    /// # Returns
    ///
    /// The maximum number of units retained in the internal buffer.
    #[must_use]
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.input.capacity()
    }

    /// Refills the internal buffer until at least `count` unread units are
    /// available.
    ///
    /// # Parameters
    ///
    /// * `count` - Minimum number of unread units required.
    ///
    /// # Errors
    ///
    /// Returns I/O errors from the wrapped input while refilling.
    #[inline(always)]
    pub fn fill_until(&mut self, count: usize) -> std::io::Result<bool> {
        self.input.fill_until(count)
    }

    /// Consumes unread units from the current buffer window.
    ///
    /// # Parameters
    ///
    /// * `count` - Number of unread units to discard.
    ///
    /// # Panics
    ///
    /// Panics when `count` exceeds [`Self::available`].
    #[inline(always)]
    pub fn consume(&mut self, count: usize) {
        self.input.consume(count)
    }

    /// Copies unread units into an indexed output range without consuming them.
    ///
    /// # Parameters
    ///
    /// * `output` - Destination storage that receives a copy of unread units.
    /// * `output_index` - Start index inside `output`.
    /// * `count` - Number of unread units to copy.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `output_index..output_index + count` is
    /// a valid range inside `output`, that the addition does not overflow, that
    /// `count <= self.available()`, and that the destination range does not
    /// overlap with the unread units stored inside this buffer.
    #[inline(always)]
    pub unsafe fn copy_unread_to_unchecked(
        &self,
        output: &mut [I::Item],
        output_index: usize,
        count: usize,
    ) {
        // SAFETY: The caller guarantees the destination range and non-overlap
        // requirements for the unread copy.
        unsafe {
            self.input
                .read_into_unchecked(output, output_index, count);
        }
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

    /// Reads buffered units into an indexed output range.
    ///
    /// # Parameters
    ///
    /// * `output` - Destination unit storage.
    /// * `output_index` - Start index inside `output`.
    /// * `count` - Maximum number of units to read.
    ///
    /// # Returns
    ///
    /// The number of units copied into `output`.
    ///
    /// # Errors
    ///
    /// Returns input or buffer validation errors from the wrapped
    /// [`qubit_io::BufferedInput`].
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `output_index..output_index + count` is
    /// a valid range inside `output` and that the addition does not overflow.
    #[inline(always)]
    pub unsafe fn read_into_unchecked(
        &mut self,
        output: &mut [I::Item],
        output_index: usize,
        count: usize,
    ) -> Result<usize> {
        // SAFETY: The caller guarantees the destination range is valid.
        unsafe { self.input.read_into_unchecked(output, output_index, count) }
    }

    /// Decodes values into an indexed output range using a [`Codec`].
    ///
    /// # Parameters
    ///
    /// * `decoder` - Codec used for this operation.
    /// * `map_error` - Function mapping decoder errors into I/O errors.
    /// * `output` - Destination value storage.
    /// * `output_index` - Start index inside `output`.
    /// * `count` - Maximum number of values to write.
    ///
    /// # Returns
    ///
    /// The number of values written. If EOF occurs before
    /// [`Codec::min_units_per_value`] units are available for the next value,
    /// the incomplete tail is left buffered and `Ok(written)` is returned.
    ///
    /// # Errors
    ///
    /// Returns input errors, buffer refill errors, or decoder errors mapped by
    /// `map_error`.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `output_index..output_index + count` is
    /// a valid range inside `output` and that the addition does not overflow.
    pub unsafe fn decode_into<C, M>(
        &mut self,
        decoder: &mut C,
        map_error: &mut M,
        output: &mut [C::Value],
        output_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        C: Codec<Unit = I::Item>,
        M: FnMut(C::DecodeError) -> Error,
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
        assert_unit_bounds::<C>(decoder);
        let min_units = decoder.min_units_per_value().get();
        let max_units = decoder.max_units_per_value().get();
        let mut written_total = 0;

        while written_total < count {
            if self.input.available() < min_units
                && !self.input.fill_until(min_units)?
            {
                return Ok(written_total);
            }

            if self.input.available() < max_units
                && max_units <= self.input.capacity()
            {
                let _ = self.input.fill_until(max_units)?;
            }

            let available = self.input.available();
            let (value, consumed) = unsafe {
                // SAFETY: The unread window contains at least
                // `min_units_per_value` units from index zero.
                decoder.decode(self.input.unread_slice(), 0)
            }
            .map_err(&mut *map_error)?;
            let consumed = consumed.get();
            assert!(
                consumed <= available,
                "Codec::decode consumed beyond available input",
            );
            output[output_index + written_total] = value;
            unsafe {
                // SAFETY: The codec-reported consumed count was checked
                // against the current unread input window.
                self.input.consume_unchecked(consumed);
            }
            written_total += 1;
        }
        Ok(written_total)
    }

    /// Decodes values into an indexed output range using a streaming
    /// [`Transcoder`].
    ///
    /// # Parameters
    ///
    /// * `decoder` - Streaming decoder used for this operation.
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
    pub unsafe fn transcode_into<D, M, Value>(
        &mut self,
        decoder: &mut D,
        map_error: &mut M,
        output: &mut [Value],
        output_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        D: Transcoder<I::Item, Value>,
        M: FnMut(TranscodeError<D::Error>) -> Error,
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
        let mut units = Vec::new();
        loop {
            if self.input.available() == 0 && !self.input.fill_more()? {
                return Ok(written_total);
            }
            let available = self.input.available();
            if units.len() < available {
                units.resize(available, I::Item::default());
            }
            // SAFETY: The first `available` slots in `units` are a valid
            // destination range, and the copied range does not overlap with
            // the buffered input storage.
            unsafe {
                self.input
                    .read_into_unchecked(&mut units, 0, available);
            }
            let units = &units[..available];
            let progress = decoder
                .transcode(units, 0, output, output_index + written_total)
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

    /// Finishes a codec decode operation into an indexed output range.
    ///
    /// # Parameters
    ///
    /// * `decoder` - Codec whose decode operation is being finished.
    /// * `map_error` - Function mapping decoder errors into I/O errors.
    /// * `output` - Destination value storage.
    /// * `output_index` - Start index inside `output`.
    /// * `count` - Maximum number of finish values to write.
    ///
    /// # Returns
    ///
    /// The number of values written by the finish operation. Since [`Codec`]
    /// has no decoder-owned finish state, this method always returns `0`.
    ///
    /// # Errors
    ///
    /// This method performs no I/O and returns no codec errors.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `output_index..output_index + count` is
    /// a valid range inside `output` and that the addition does not overflow.
    pub unsafe fn finish_into<C, M>(
        &mut self,
        _decoder: &C,
        _map_error: &mut M,
        output: &mut [C::Value],
        output_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        C: Codec<Unit = I::Item>,
        M: FnMut(C::DecodeError) -> Error,
    {
        debug_assert!(
            output_index
                .checked_add(count)
                .is_some_and(|end| end <= output.len()),
            "unchecked finish output range exceeds destination buffer",
        );
        Ok(0)
    }

    /// Finishes a streaming decoder into an indexed output range.
    ///
    /// # Parameters
    ///
    /// * `decoder` - Streaming decoder whose final output is being collected.
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
    pub unsafe fn finish_transcode_into<D, M, Value>(
        &mut self,
        decoder: &mut D,
        map_error: &mut M,
        output: &mut [Value],
        output_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        D: Transcoder<I::Item, Value>,
        M: FnMut(TranscodeError<D::Error>) -> Error,
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
        TranscodeError::<core::convert::Infallible>::ensure_output_range(
            output.len(),
            output_index,
            count,
            required,
        )
        .map_err(transcode_contract_to_io_error)?;
        let output_end = output_index + count;
        let output = &mut output[..output_end];
        let written = decoder
            .finish(output, output_index)
            .map_err(&mut *map_error)?;
        assert!(written <= required, "finish wrote beyond its bound");
        Ok(written)
    }
}

impl<I> TranscodeDecodeInput<I>
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

impl<I> Read for TranscodeDecodeInput<I>
where
    I: Input<Item = u8>,
{
    /// Reads raw bytes through the internal buffer.
    #[inline]
    fn read(&mut self, output: &mut [u8]) -> Result<usize> {
        // SAFETY: The full output slice is a valid destination range.
        unsafe { self.input.read_into_unchecked(output, 0, output.len()) }
    }
}

impl<I> Seek for TranscodeDecodeInput<I>
where
    I: Input<Item = u8> + Seek,
{
    /// Seeks the wrapped byte input and discards buffered bytes after success.
    #[inline]
    fn seek(&mut self, position: SeekFrom) -> Result<u64> {
        self.seek(position)
    }
}

impl<I> fmt::Debug for TranscodeDecodeInput<I>
where
    I: Input,
    I::Item: Copy + Default,
    BufferedInput<I>: fmt::Debug,
{
    /// Formats this buffered decode input for debugging.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TranscodeDecodeInput")
            .field("input", &self.input)
            .finish()
    }
}

/// Converts a capacity planning failure into an I/O error.
fn capacity_to_io_error(error: crate::CapacityError) -> Error {
    Error::new(ErrorKind::InvalidData, error)
}

/// Converts a framework transcode contract failure into an I/O error.
fn transcode_contract_to_io_error(
    error: TranscodeError<core::convert::Infallible>,
) -> Error {
    Error::new(ErrorKind::InvalidData, error)
}
