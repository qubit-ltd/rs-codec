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
    TranscodeError,
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
    scratch_unread: Vec<I::Item>,
    scratch_position: usize,
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
            scratch_unread: Vec::new(),
            scratch_position: 0,
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
            scratch_unread: Vec::new(),
            scratch_position: 0,
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
        if self.has_scratch_unread() {
            return self.scratch_unread_len();
        }
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
        if self.has_scratch_unread() {
            return self.scratch_unread();
        }
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
        if self.has_scratch_unread() {
            return self.fill_scratch_until(count);
        }
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
        if self.has_scratch_unread() {
            self.consume_scratch(count);
            return;
        }
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
        let unread = self.unread();
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
        let scratch_position = self.scratch_position;
        let scratch_unread = self.scratch_unread;
        let (inner, input_buffer) = self.input.into_parts();
        let scratch = &scratch_unread[scratch_position..];
        if scratch.is_empty() {
            return (inner, input_buffer);
        }
        let input_unread = input_buffer.readable();
        let mut buffer =
            Buffer::with_capacity(scratch.len() + input_unread.len());
        unsafe {
            // SAFETY: The destination buffer was sized to hold both readable
            // ranges, and the source slices are external to `buffer`.
            buffer.copy_from(scratch, 0, scratch.len());
            buffer.copy_from(input_unread, 0, input_unread.len());
        }
        (inner, buffer)
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
        debug_assert!(
            UncheckedSlice::range_fits(output.len(), output_index, count),
            "unchecked read output range exceeds destination buffer",
        );
        if count == 0 {
            return Ok(0);
        }
        let mut total = 0;
        if self.has_scratch_unread() {
            let read = count.min(self.scratch_unread_len());
            let scratch = self.scratch_unread();
            output[output_index..output_index + read]
                .copy_from_slice(&scratch[..read]);
            self.consume_scratch(read);
            total = read;
            if total == count {
                return Ok(total);
            }
        }
        // SAFETY: The caller guarantees the original destination range is
        // valid; `total < count`, so this suffix is still in range.
        let read = unsafe {
            self.input.read_unchecked(
                output,
                output_index + total,
                count - total,
            )
        }?;
        Ok(total + read)
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
        let min_units_per_value = C::MIN_UNITS_PER_VALUE;
        let max_units_per_value =
            C::MAX_UNITS_PER_VALUE.max(min_units_per_value);
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
                    // continue to the next loop
                }
                Err(DecodeFailure::Invalid { source, consumed }) => {
                    if consumed.get() > units.len() {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "decode error consumed units exceed unread window",
                        ));
                    }
                    self.consume(consumed.get());
                    return Err(map_error(source));
                }
                Err(DecodeFailure::InvalidUnknown { source }) => {
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
    /// * `map_error` - Function mapping transcode errors into I/O errors.
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
    /// internal buffer, or transcode errors mapped by `map_error`.
    pub fn transcode_into<D, M, Value>(
        &mut self,
        decoder: &mut D,
        map_error: &mut M,
        output: &mut [Value],
        output_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        D: Transcoder<Input = I::Item, Output = Value>,
        M: FnMut(TranscodeError<D::DomainError, D::FailureValue>) -> Error,
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
            if self.unread_len() == 0 && !self.fill_more()? {
                return Ok(written_total);
            }
            let units = self.unread();
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
            self.consume(consumed);
            written_total += written;
            match progress.status() {
                TranscodeStatus::Complete => {
                    if consumed == 0 && available_input != 0 {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "decoder reported Complete without consuming non-empty input",
                        ));
                    }
                    if written_total == count {
                        return Ok(written_total);
                    }
                }
                TranscodeStatus::NeedOutput { .. } => {
                    return Ok(written_total);
                }
                TranscodeStatus::NeedInput { required, .. } => {
                    if self.fill_until(required.get())? {
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
    /// * `map_error` - Function mapping transcode errors into I/O errors.
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
    /// Returns invalid output ranges, capacity errors, or transcode
    /// finalization errors mapped to I/O errors.
    pub fn finish_transcode_into<D, M, Value>(
        &mut self,
        decoder: &mut D,
        map_error: &mut M,
        output: &mut [Value],
        output_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        D: Transcoder<Input = I::Item, Output = Value>,
        M: FnMut(TranscodeError<D::DomainError, D::FailureValue>) -> Error,
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
        let position = self.adjust_seek_for_scratch(position)?;
        let new_position = self.input.seek_to(position)?;
        self.clear_scratch_unread();
        Ok(new_position)
    }
}

impl<I> TranscodeDecodeInput<I>
where
    I: Input,
    I::Item: Copy + Default,
{
    /// Returns whether scratch-owned unread units exist.
    #[inline(always)]
    fn has_scratch_unread(&self) -> bool {
        self.scratch_position < self.scratch_unread.len()
    }

    /// Returns scratch-owned unread units.
    #[inline(always)]
    fn scratch_unread(&self) -> &[I::Item] {
        &self.scratch_unread[self.scratch_position..]
    }

    /// Returns the number of scratch-owned unread units.
    #[inline(always)]
    fn scratch_unread_len(&self) -> usize {
        self.scratch_unread.len() - self.scratch_position
    }

    /// Clears scratch-owned unread units.
    #[inline(always)]
    fn clear_scratch_unread(&mut self) {
        self.scratch_unread.clear();
        self.scratch_position = 0;
    }

    /// Consumes units from the scratch-owned unread window.
    #[inline(always)]
    fn consume_scratch(&mut self, count: usize) {
        debug_assert!(
            count <= self.scratch_unread_len(),
            "cannot consume beyond scratch unread input",
        );
        self.scratch_position += count;
        if self.scratch_position == self.scratch_unread.len() {
            self.clear_scratch_unread();
        }
    }

    /// Appends wrapped input units to the scratch-owned unread window.
    fn fill_scratch_until(&mut self, count: usize) -> Result<bool> {
        while self.scratch_unread_len() < count {
            let missing = count - self.scratch_unread_len();
            let start = self.scratch_unread.len();
            self.scratch_unread
                .resize(start + missing, I::Item::default());
            let read_result = unsafe {
                // SAFETY: The scratch vector was resized to provide the
                // destination range being filled.
                self.input.read_unchecked(
                    &mut self.scratch_unread,
                    start,
                    missing,
                )
            };
            let read = match read_result {
                Ok(read) => read,
                Err(error) => {
                    self.scratch_unread.truncate(start);
                    return Err(error);
                }
            };
            if read == 0 {
                self.scratch_unread.truncate(start);
                return Ok(false);
            }
            self.scratch_unread.truncate(start + read);
        }
        Ok(true)
    }

    /// Refills the underlying buffer.
    fn fill_more(&mut self) -> Result<bool> {
        debug_assert!(
            !self.has_scratch_unread(),
            "scratch unread units must be consumed before refilling input",
        );
        self.input.fill_more()
    }

    /// Stores unconsumed scratch units as the next unread window.
    fn store_scratch_tail(&mut self, units: &[I::Item], start: usize) {
        if start >= units.len() {
            self.clear_scratch_unread();
            return;
        }
        self.scratch_unread.clear();
        self.scratch_unread.extend_from_slice(&units[start..]);
        self.scratch_position = 0;
    }

    /// Adjusts relative seeks for scratch-owned unread units.
    fn adjust_seek_for_scratch(&self, position: SeekFrom) -> Result<SeekFrom> {
        let SeekFrom::Current(offset) = position else {
            return Ok(position);
        };
        let scratch = self.scratch_unread_len().min(i64::MAX as usize) as i64;
        let adjusted = offset.checked_sub(scratch).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidInput,
                "current seek offset underflows after scratch adjustment",
            )
        })?;
        Ok(SeekFrom::Current(adjusted))
    }
}

impl<I> Read for TranscodeDecodeInput<I>
where
    I: Input<Item = u8>,
{
    /// Reads raw bytes through the internal buffer.
    fn read(&mut self, output: &mut [u8]) -> Result<usize> {
        // SAFETY: The full output slice is a valid destination range.
        unsafe { self.read_unchecked(output, 0, output.len()) }
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
            Ok((value, consumed)) => {
                let consumed = consumed.get();
                if consumed > loaded {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "codec consumed units exceed loaded scratch window",
                    ));
                }
                input.store_scratch_tail(&units[..loaded], consumed);
                return Ok(value);
            }
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
            Err(DecodeFailure::Invalid { source, consumed }) => {
                let consumed = consumed.get();
                if consumed > loaded {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "decode error consumed units exceed loaded scratch window",
                    ));
                }
                input.store_scratch_tail(&units[..loaded], consumed);
                return Err(map_error(source));
            }
            Err(DecodeFailure::InvalidUnknown { source }) => {
                input.store_scratch_tail(&units[..loaded], 0);
                return Err(map_error(source));
            }
        }
    }
}
