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
    Buffer,
    BufferedInput,
    Input,
    Seekable,
    UncheckedSlice,
};

use crate::{
    Codec,
    DecodeFailure,
    TranscodeStatus,
    Transcoder,
};

/// Decodes an [`Input`] unit stream into an [`Input`] value stream.
///
/// This type owns only the unit-level [`qubit_io::BufferedInput`]. Callers pass
/// a streaming [`Transcoder`] and error mapper to each decode operation, which
/// lets one buffered input drive different decoders without nesting buffers or
/// storing decoder-specific state in the buffer owner.
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
    pub const fn inner(&self) -> &I {
        self.input.inner()
    }

    /// Returns a mutable reference to the wrapped unit input.
    ///
    /// # Returns
    ///
    /// A mutable reference to the wrapped unit input.
    pub fn inner_mut(&mut self) -> &mut I {
        self.input.inner_mut()
    }

    /// Returns the number of unread units currently buffered.
    ///
    /// # Returns
    ///
    /// The number of unread units in the internal buffer.
    #[must_use]
    pub fn unread_len(&self) -> usize {
        self.input.unread_len()
    }

    /// Returns the currently buffered unread units.
    ///
    /// # Returns
    ///
    /// Returns a shared slice over the unread portion of the internal unit
    /// buffer. The slice is valid until this adapter is mutated.
    #[must_use]
    pub fn unread(&self) -> &[I::Item] {
        self.input.unread()
    }

    /// Returns the internal unit buffer capacity.
    ///
    /// # Returns
    ///
    /// The maximum number of units retained in the internal buffer.
    #[must_use]
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
    /// In debug builds, panics when `count` exceeds [`Self::unread_len`].
    pub fn consume(&mut self, count: usize) {
        debug_assert!(
            count <= self.unread_len(),
            "cannot consume beyond buffered input",
        );
        // SAFETY: The caller-provided count is within the unread window.
        unsafe {
            self.input.consume(count);
        }
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
    /// `count <= self.unread_len()`, and that the destination range does not
    /// overlap with the unread units stored inside this buffer.
    pub unsafe fn copy_unread_to(
        &mut self,
        output: &mut [I::Item],
        output_index: usize,
        count: usize,
    ) {
        // SAFETY: The caller guarantees the destination range and non-overlap
        // requirements for the unread copy.
        let unread = self.input.unread();
        debug_assert!(
            UncheckedSlice::range_fits(unread.len(), 0, count),
            "unchecked unread copy range exceeds unread source",
        );
        debug_assert!(
            UncheckedSlice::range_fits(output.len(), output_index, count),
            "unchecked copy destination range exceeds output buffer",
        );
        unsafe {
            UncheckedSlice::copy_nonoverlapping(
                unread,
                0,
                output,
                output_index,
                count,
            );
        }
    }

    /// Consumes this adapter and returns its parts.
    ///
    /// # Returns
    ///
    /// The wrapped input and the buffer holding unread units.
    #[must_use]
    pub fn into_parts(self) -> (I, Buffer<I::Item>) {
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
    pub unsafe fn read_unchecked(
        &mut self,
        output: &mut [I::Item],
        output_index: usize,
        count: usize,
    ) -> Result<usize> {
        // SAFETY: The caller guarantees the destination range is valid.
        unsafe { self.input.read_unchecked(output, output_index, count) }
    }

    /// Decodes one codec value from the buffered unit input.
    ///
    /// The method refills the internal input buffer until the supplied codec
    /// can decode one complete value or until the wrapped input reaches
    /// EOF.
    ///
    /// # Parameters
    ///
    /// * `codec` - Codec used for this single-value decode.
    /// * `map_error` - Mapper for codec-domain invalid-input errors.
    ///
    /// # Returns
    ///
    /// Returns one decoded codec value.
    ///
    /// # Errors
    ///
    /// Returns I/O errors from the wrapped input, `UnexpectedEof` when EOF
    /// occurs before a complete value is available, `InvalidData` when the
    /// codec reports an impossible incomplete state, or the error returned
    /// by `map_error` for invalid codec input.
    pub fn read_decoded_with<C, M>(
        &mut self,
        codec: &mut C,
        mut map_error: M,
    ) -> Result<C::Value>
    where
        C: Codec<Unit = I::Item>,
        M: FnMut(C::DecodeError) -> Error,
    {
        let min_units_per_value = C::MIN_UNITS_PER_VALUE.get();
        let max_units_per_value =
            C::MAX_UNITS_PER_VALUE.get().max(min_units_per_value);
        if min_units_per_value > self.capacity() {
            return read_decoded_via_scratch(
                self,
                codec,
                min_units_per_value,
                &mut map_error,
            );
        }

        loop {
            let available = self.unread_len();
            if available < min_units_per_value
                && !self.fill_until(min_units_per_value)?
            {
                let available = self.unread_len();
                self.consume(available);
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "failed to decode complete value",
                ));
            }

            if self.unread_len() < max_units_per_value
                && max_units_per_value <= self.capacity()
            {
                let _ = self.fill_until(max_units_per_value)?;
            }

            let available = self.unread_len();
            let unit_count = available.min(max_units_per_value);
            let units = &self.unread()[..unit_count];
            debug_assert!(units.len() >= min_units_per_value);
            let decode_result = unsafe {
                // SAFETY: `min_units_per_value <= units.len()` guarantees
                // `decode` preconditions for this slice.
                codec.decode(units, 0)
            };
            match decode_result {
                Ok((value, consumed)) => {
                    if consumed.get() > units.len() {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "codec consumed units exceed unread window",
                        ));
                    }
                    self.consume(consumed.get());
                    return Ok(value);
                }
                Err(DecodeFailure::Incomplete { required_total }) => {
                    let required_total = required_total.get();
                    if units.len() >= required_total {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "codec reported incomplete input within available window",
                        ));
                    }
                    if !self.fill_until(required_total)? {
                        let available = self.unread_len();
                        self.consume(available);
                        return Err(Error::new(
                            ErrorKind::UnexpectedEof,
                            "failed to decode complete value",
                        ));
                    }
                }
                Err(DecodeFailure::Invalid { source, consumed }) => {
                    if let Some(consumed) = consumed {
                        if consumed.get() > units.len() {
                            return Err(Error::new(
                                ErrorKind::InvalidData,
                                "decode error consumed units exceed unread window",
                            ));
                        }
                        self.consume(consumed.get());
                    }
                    return Err(map_error(source));
                }
            }
        }
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
    /// Returns input errors, invalid output ranges, capacity errors from the
    /// internal buffer, or decoder errors mapped by `map_error`.
    pub fn transcode_into<D, M, Value>(
        &mut self,
        decoder: &mut D,
        map_error: &mut M,
        output: &mut [Value],
        output_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        D: Transcoder<I::Item, Value>,
        M: FnMut(D::Error) -> Error,
    {
        let output_end = UncheckedSlice::checked_range_end(
            output.len(),
            output_index,
            count,
            "decoded output range exceeds destination buffer",
        )?;
        if count == 0 {
            return Ok(0);
        }
        let output = &mut output[..output_end];
        let mut written_total = 0;
        loop {
            if self.input.unread_len() == 0 && !self.input.fill_more()? {
                return Ok(written_total);
            }
            let units = self.input.unread();
            let available_input = units.len();
            let remaining_output = count - written_total;
            let progress = decoder
                .transcode(units, 0, output, output_index + written_total)
                .map_err(&mut *map_error)?;
            progress
                .validate(
                    0,
                    available_input,
                    output_index + written_total,
                    remaining_output,
                )
                .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
            let consumed = progress.read();
            let written = progress.written();
            // SAFETY: The progress bounds check above proved that the decoder
            // consumed no more than the currently unread input window.
            unsafe {
                self.input.consume(consumed);
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
                TranscodeStatus::NeedInput { required, .. } => {
                    if self.input.fill_until(required.get())? {
                        continue;
                    }
                    return Ok(written_total);
                }
            }
        }
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
    /// Returns invalid output ranges, capacity errors, or decoder finalization
    /// errors mapped to I/O errors.
    pub fn finish_transcode_into<D, M, Value>(
        &mut self,
        decoder: &mut D,
        map_error: &mut M,
        output: &mut [Value],
        output_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        D: Transcoder<I::Item, Value>,
        M: FnMut(D::Error) -> Error,
    {
        let required = decoder
            .max_finish_output_len()
            .map_err(capacity_to_io_error)?;
        // Validate the caller-supplied count range first (InvalidInput).
        let output_end = UncheckedSlice::checked_range_end(
            output.len(),
            output_index,
            count,
            "finish output range exceeds destination buffer",
        )?;
        // `count` is the caller's declared writable finish range.  The
        // destination slice may be larger, but passing extra capacity to the
        // decoder would allow it to write beyond the range the caller granted.
        if count < required {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "insufficient output for decoder finish bound",
            ));
        }
        let output = &mut output[..output_end];
        let written = decoder
            .finish(output, output_index)
            .map_err(&mut *map_error)?;
        debug_assert!(written <= required, "finish wrote beyond its bound");
        Ok(written)
    }
}

impl<I> TranscodeDecodeInput<I>
where
    I: Input<Item = u8> + Seekable<Item = u8>,
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
    pub fn seek(&mut self, position: SeekFrom) -> Result<u64> {
        self.input.seek_to(position)
    }
}

impl<I> Read for TranscodeDecodeInput<I>
where
    I: Input<Item = u8>,
{
    /// Reads raw bytes through the internal buffer.
    fn read(&mut self, output: &mut [u8]) -> Result<usize> {
        // SAFETY: The full output slice is a valid destination range.
        unsafe { self.input.read_unchecked(output, 0, output.len()) }
    }
}

impl<I> Seek for TranscodeDecodeInput<I>
where
    I: Input<Item = u8> + Seekable<Item = u8>,
{
    /// Seeks the wrapped byte input and discards buffered bytes after success.
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

/// Decodes one value through caller-owned scratch storage.
fn read_decoded_via_scratch<I, C, M>(
    input: &mut TranscodeDecodeInput<I>,
    codec: &mut C,
    mut required_total: usize,
    map_error: &mut M,
) -> Result<C::Value>
where
    I: Input,
    I::Item: Copy + Default,
    C: Codec<Unit = I::Item>,
    M: FnMut(C::DecodeError) -> Error,
{
    let mut units = vec![I::Item::default(); required_total];
    let mut loaded = 0;
    loop {
        while loaded < required_total {
            let remaining = required_total - loaded;
            let read = unsafe {
                // SAFETY: `units` was resized to at least `required_total`, so
                // `loaded..loaded + remaining` is a valid destination range.
                input.read_unchecked(&mut units, loaded, remaining)
            }?;
            if read == 0 {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "failed to decode complete value",
                ));
            }
            loaded += read;
        }
        let decode_result = unsafe {
            // SAFETY: `loaded >= required_total >= min_units_per_value`, so the
            // scratch buffer contains the required prefix for decoding.
            codec.decode(&units, 0)
        };
        match decode_result {
            Ok((value, _)) => return Ok(value),
            Err(DecodeFailure::Incomplete {
                required_total: next_required_total,
            }) => {
                let next_required_total = next_required_total.get();
                if next_required_total <= loaded {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "codec reported incomplete input within loaded scratch window",
                    ));
                }
                units.resize(next_required_total, I::Item::default());
                required_total = next_required_total;
            }
            Err(DecodeFailure::Invalid { source, .. }) => {
                return Err(map_error(source));
            }
        }
    }
}
