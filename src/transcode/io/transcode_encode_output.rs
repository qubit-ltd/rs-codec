// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Buffered output driver that encodes values into units.

use core::fmt;
use std::io::{
    Error,
    ErrorKind,
    Result,
    Seek,
    SeekFrom,
    Write,
};

use qubit_io::{
    Buffer,
    BufferedOutput,
    Output,
    Seekable,
    UncheckedSlice,
};

use crate::{
    TranscodeError,
    TranscodeStatus,
    Transcoder,
};

/// Encodes an [`Output`] value stream into an [`Output`] unit stream.
///
/// This type owns only the unit-level [`qubit_io::BufferedOutput`]. Callers
/// pass a [`crate::Codec`] and error mapper to each encode operation, which
/// lets one buffered output drive different encoders without nesting buffers or
/// storing codec-specific state in the buffer owner.
///
/// [`Self::flush`] only drains already buffered units. State-aware streaming
/// encoders can use [`Self::transcode_from`] and [`Self::finish`] explicitly.
///
/// # Type Parameters
///
/// * `O` - Wrapped unit output.
pub struct TranscodeEncodeOutput<O>
where
    O: Output,
    O::Item: Copy + Default,
{
    output: BufferedOutput<O>,
}

impl<O> TranscodeEncodeOutput<O>
where
    O: Output,
    O::Item: Copy + Default,
{
    /// Creates an encoder output with the default unit buffer capacity.
    ///
    /// # Parameters
    ///
    /// * `inner` - Unit output written by this adapter.
    ///
    /// # Returns
    ///
    /// A new buffered encoder output.
    #[inline]
    #[must_use]
    pub fn new(inner: O) -> Self {
        Self {
            output: BufferedOutput::new(inner),
        }
    }

    /// Creates an encoder output with a unit buffer of at least `capacity`.
    ///
    /// # Parameters
    ///
    /// * `inner` - Unit output written by this adapter.
    /// * `capacity` - Requested internal unit buffer capacity.
    ///
    /// # Returns
    ///
    /// A new buffered encoder output.
    #[inline]
    #[must_use]
    pub fn with_capacity(inner: O, capacity: usize) -> Self {
        Self {
            output: BufferedOutput::with_capacity(inner, capacity),
        }
    }

    /// Returns a shared reference to the wrapped unit output.
    ///
    /// # Returns
    ///
    /// A shared reference to the wrapped unit output.
    #[inline(always)]
    #[must_use]
    pub const fn inner(&self) -> &O {
        self.output.inner()
    }

    /// Returns a mutable reference to the wrapped unit output.
    ///
    /// # Returns
    ///
    /// A mutable reference to the wrapped unit output.
    #[inline(always)]
    pub fn inner_mut(&mut self) -> &mut O {
        self.output.inner_mut()
    }

    /// Returns the available capacity of the spare output buffer.
    ///
    /// # Returns
    ///
    /// The number of output units that can still be appended without flushing.
    #[inline(always)]
    #[must_use]
    pub fn spare_capacity(&self) -> usize {
        self.output.spare_capacity()
    }

    /// Returns raw spare-buffer parts for the internal output buffer.
    ///
    /// # Returns
    ///
    /// The full backing storage, the spare start index, and the spare unit
    /// count.
    #[inline(always)]
    #[must_use]
    pub fn spare_raw_parts_mut(&mut self) -> (&mut [O::Item], usize, usize) {
        self.output.spare_raw_parts_mut()
    }

    /// Marks `count` units from [`Self::spare_raw_parts_mut`] as written.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `count <= Self::spare_capacity()` and
    /// that the corresponding units in the returned spare slice have been
    /// initialized.
    #[inline(always)]
    pub unsafe fn advance(&mut self, count: usize) {
        // SAFETY: The caller guarantees `count` and initialization invariants.
        unsafe { self.output.advance(count) }
    }

    /// Ensures that at least `count` spare units are available.
    ///
    /// # Parameters
    ///
    /// * `count` - Number of spare units required.
    ///
    /// # Errors
    ///
    /// Returns I/O errors from the wrapped output while flushing pending units.
    #[inline(always)]
    pub fn ensure_spare_capacity(&mut self, count: usize) -> Result<()> {
        self.output.ensure_spare_capacity(count)
    }

    /// Consumes this adapter and returns its parts.
    ///
    /// # Returns
    ///
    /// The wrapped output and the buffer holding pending units.
    #[inline]
    #[must_use]
    pub fn into_parts(self) -> (O, Buffer<O::Item>) {
        self.output.into_parts()
    }

    /// Flushes buffered units without finishing any encoder stream.
    ///
    /// # Errors
    ///
    /// Returns errors from the wrapped output while flushing pending units.
    #[inline]
    pub fn flush(&mut self) -> Result<()> {
        self.output.flush()
    }

    /// Encodes values from an indexed input range using a streaming
    /// [`Transcoder`].
    ///
    /// # Parameters
    ///
    /// * `encoder` - Streaming encoder used for this operation.
    /// * `map_error` - Function mapping encoder errors into I/O errors.
    /// * `input` - Source values.
    /// * `input_index` - Start index inside `input`.
    /// * `count` - Maximum number of values to encode.
    ///
    /// # Returns
    ///
    /// The number of source values consumed.
    ///
    /// # Errors
    ///
    /// Returns invalid input ranges, capacity, encoder, or output errors.
    #[inline]
    pub fn transcode_from<E, M, Value>(
        &mut self,
        encoder: &mut E,
        map_error: &mut M,
        input: &[Value],
        input_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        E: Transcoder<Value, O::Item>,
        M: FnMut(TranscodeError<E::Error>) -> Error,
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
        while read_total < count {
            // Each encoder step writes into the spare output window. When the
            // buffer is full of pending units, spare capacity drops to zero and
            // `transcode` cannot make progress. Reserving one spare slot drains
            // pending units to the wrapped output only when needed.
            self.output.ensure_spare_capacity(1)?;
            let (units, output_index, available_output) =
                self.output.spare_raw_parts_mut();
            let remaining_input = count - read_total;
            let progress = encoder
                .transcode(input, input_index + read_total, units, output_index)
                .map_err(&mut *map_error)?;
            progress
                .validate(
                    input_index + read_total,
                    remaining_input,
                    output_index,
                    available_output,
                )
                .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
            let read = progress.read();
            let written = progress.written();
            // SAFETY: TranscodeProgress::validate proved that the encoder
            // initialized no more than the available spare output window.
            unsafe {
                self.output.advance(written);
            }
            read_total += read;
            match progress.status() {
                TranscodeStatus::Complete => return Ok(read_total),
                TranscodeStatus::NeedOutput { required, .. } => {
                    self.output.ensure_spare_capacity(required.get())?;
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

    /// Finishes the encoder and flushes the wrapped unit output.
    ///
    /// # Parameters
    ///
    /// * `encoder` - Encoder whose final units are being collected.
    /// * `map_error` - Function mapping encoder errors into I/O errors.
    ///
    /// # Errors
    ///
    /// Returns capacity, encoder finalization, or wrapped output flush errors.
    #[inline]
    pub fn finish<E, M, Value>(
        &mut self,
        encoder: &mut E,
        map_error: &mut M,
    ) -> Result<()>
    where
        E: Transcoder<Value, O::Item>,
        M: FnMut(TranscodeError<E::Error>) -> Error,
    {
        let required = encoder
            .max_finish_output_len()
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        self.output.ensure_spare_capacity(required)?;
        let (units, output_index, available) =
            self.output.spare_raw_parts_mut();
        debug_assert!(
            available >= required,
            "insufficient finish capacity reserved in spare output buffer",
        );
        let written = encoder
            .finish(units, output_index)
            .map_err(&mut *map_error)?;
        debug_assert!(written <= required, "finish wrote beyond its bound");
        // SAFETY: The encoder reported initialized units within the spare
        // range that was reserved above.
        unsafe {
            self.output.advance(written);
        }
        self.output.flush()
    }
}

impl<O> TranscodeEncodeOutput<O>
where
    O: Output<Item = u8> + Seekable<Item = u8>,
{
    /// Flushes pending bytes, then seeks the wrapped byte output.
    ///
    /// # Parameters
    ///
    /// * `position` - Target seek position.
    ///
    /// # Returns
    ///
    /// The new stream position reported by the wrapped output.
    ///
    /// # Errors
    ///
    /// Returns flush or seek errors from the wrapped output.
    #[inline]
    pub fn seek(&mut self, position: SeekFrom) -> Result<u64> {
        self.output.seek_to(position)
    }
}

impl<O> Write for TranscodeEncodeOutput<O>
where
    O: Output<Item = u8>,
{
    /// Writes raw bytes through the internal buffer.
    #[inline]
    fn write(&mut self, input: &[u8]) -> Result<usize> {
        Output::write(&mut self.output, input)
    }

    /// Writes all raw bytes through the internal buffer.
    #[inline]
    fn write_all(&mut self, input: &[u8]) -> Result<()> {
        self.output.write_all(input)
    }

    /// Flushes buffered bytes to the wrapped output.
    #[inline]
    fn flush(&mut self) -> Result<()> {
        TranscodeEncodeOutput::flush(self)
    }
}

impl<O> Seek for TranscodeEncodeOutput<O>
where
    O: Output<Item = u8> + Seekable<Item = u8>,
{
    /// Flushes pending bytes, then seeks the wrapped byte output.
    #[inline]
    fn seek(&mut self, position: SeekFrom) -> Result<u64> {
        self.seek(position)
    }
}

impl<O> fmt::Debug for TranscodeEncodeOutput<O>
where
    O: Output,
    O::Item: Copy + Default,
    BufferedOutput<O>: fmt::Debug,
{
    /// Formats this buffered encode output for debugging.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TranscodeEncodeOutput")
            .field("output", &self.output)
            .finish()
    }
}
