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
    BufferedOutput,
    Output,
};

use super::{
    BufferedTranscoder,
    FinishError,
    TranscodeStatus,
};

/// Encodes an [`Output`] value stream into an [`Output`] unit stream.
///
/// This type owns only the unit-level [`qubit_io::BufferedOutput`]. Callers
/// pass the encoder and error mapper to each encode operation, which lets one
/// buffered output drive different encoders without nesting buffers or storing
/// codec-specific state in the buffer owner.
///
/// [`Self::flush`] only drains already buffered units. [`Self::finish`] accepts
/// a caller-owned encoder and closes that encoder's logical stream under the
/// [`BufferedTranscoder`] lifecycle. Because encoder state is not stored here,
/// closed/open state remains the responsibility of the caller-owned encoder or
/// wrapper.
///
/// # Type Parameters
///
/// * `O` - Wrapped unit output.
pub struct BufferedEncodeOutput<O>
where
    O: Output,
    O::Item: Copy + Default,
{
    output: BufferedOutput<O>,
}

impl<O> fmt::Debug for BufferedEncodeOutput<O>
where
    O: Output,
    O::Item: Copy + Default,
    BufferedOutput<O>: fmt::Debug,
{
    /// Formats this buffered encode output for debugging.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BufferedEncodeOutput")
            .field("output", &self.output)
            .finish()
    }
}

impl<O> BufferedEncodeOutput<O>
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
    #[must_use]
    #[inline]
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
    #[must_use]
    #[inline]
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
    #[must_use]
    #[inline(always)]
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

    /// Consumes this adapter and returns its parts.
    ///
    /// # Returns
    ///
    /// The wrapped output and pending units.
    #[must_use]
    #[inline]
    pub fn into_parts(self) -> (O, Vec<O::Item>) {
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

    /// Writes raw units through the internal output buffer.
    ///
    /// # Parameters
    ///
    /// * `input` - Source units to write.
    ///
    /// # Errors
    ///
    /// Returns errors from the wrapped output while accepting or flushing
    /// units.
    #[inline]
    pub fn write_units(&mut self, input: &[O::Item]) -> Result<()> {
        // SAFETY: The full input slice is a valid source range.
        unsafe { self.output.write_all_unchecked(input, 0, input.len()) }
    }

    /// Writes raw units from an indexed input range.
    ///
    /// # Parameters
    ///
    /// * `input` - Source units.
    /// * `input_index` - Start index inside `input`.
    /// * `count` - Number of units to write.
    ///
    /// # Returns
    ///
    /// The number of units accepted by the buffered output.
    ///
    /// # Errors
    ///
    /// Returns errors from the wrapped output while accepting or flushing
    /// units.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `input_index..input_index + count` is a
    /// valid range inside `input` and that the addition does not overflow.
    #[inline]
    pub unsafe fn write_units_unchecked(
        &mut self,
        input: &[O::Item],
        input_index: usize,
        count: usize,
    ) -> Result<usize> {
        // SAFETY: The caller guarantees that the source range is valid.
        unsafe { self.output.write_from_unchecked(input, input_index, count) }
    }
}

impl<O> BufferedEncodeOutput<O>
where
    O: Output<Item = u8> + Seek,
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
        Seek::seek(&mut self.output, position)
    }
}

impl<O> Write for BufferedEncodeOutput<O>
where
    O: Output<Item = u8>,
{
    /// Writes raw bytes through the internal buffer.
    #[inline]
    fn write(&mut self, input: &[u8]) -> Result<usize> {
        // SAFETY: The full input slice is a valid source range.
        unsafe { self.write_units_unchecked(input, 0, input.len()) }
    }

    /// Writes all raw bytes through the internal buffer.
    #[inline]
    fn write_all(&mut self, input: &[u8]) -> Result<()> {
        self.write_units(input)
    }

    /// Flushes buffered bytes to the wrapped output.
    #[inline]
    fn flush(&mut self) -> Result<()> {
        BufferedEncodeOutput::flush(self)
    }
}

impl<O> Seek for BufferedEncodeOutput<O>
where
    O: Output<Item = u8> + Seek,
{
    /// Flushes pending bytes, then seeks the wrapped byte output.
    #[inline]
    fn seek(&mut self, position: SeekFrom) -> Result<u64> {
        self.seek(position)
    }
}

impl<O> BufferedEncodeOutput<O>
where
    O: Output,
    O::Item: Copy + Default,
{
    /// Encodes values from a checked input range.
    ///
    /// # Parameters
    ///
    /// * `encoder` - Encoder used for this operation.
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
    /// Returns capacity, encoder, or output errors.
    pub fn encode_from<E, M, Value>(
        &mut self,
        encoder: &mut E,
        map_error: &mut M,
        input: &[Value],
        input_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        E: BufferedTranscoder<Value, O::Item>,
        M: FnMut(E::Error) -> Error,
    {
        assert!(
            input_index
                .checked_add(count)
                .is_some_and(|end| end <= input.len()),
            "encode input range exceeds source buffer",
        );
        // SAFETY: The assertion proves that the requested input range is
        // valid.
        unsafe {
            self.encode_from_unchecked(
                encoder,
                map_error,
                input,
                input_index,
                count,
            )
        }
    }

    /// Encodes values from an indexed input range without checking bounds.
    ///
    /// # Parameters
    ///
    /// * `encoder` - Encoder used for this operation.
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
    /// Returns capacity, encoder, or output errors.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `input_index..input_index + count` is a
    /// valid range inside `input` and that the addition does not overflow.
    pub unsafe fn encode_from_unchecked<E, M, Value>(
        &mut self,
        encoder: &mut E,
        map_error: &mut M,
        input: &[Value],
        input_index: usize,
        count: usize,
    ) -> Result<usize>
    where
        E: BufferedTranscoder<Value, O::Item>,
        M: FnMut(E::Error) -> Error,
    {
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
            let (units, unit_index, available) =
                self.output.spare_raw_parts_mut();
            let units = &mut units[..unit_index + available];
            let progress = encoder
                .transcode(input, input_index + read_total, units, unit_index)
                .map_err(&mut *map_error)?;
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

    /// Finishes the encoder into the internal unit buffer.
    ///
    /// # Parameters
    ///
    /// * `encoder` - Encoder whose final units are being collected.
    /// * `map_error` - Function mapping encoder errors into I/O errors.
    ///
    /// # Errors
    ///
    /// Returns capacity, encoder, or output errors.
    pub fn finish_encoder<E, M, Value>(
        &mut self,
        encoder: &mut E,
        map_error: &mut M,
    ) -> Result<()>
    where
        E: BufferedTranscoder<Value, O::Item>,
        M: FnMut(E::Error) -> Error,
    {
        let required = encoder
            .max_finish_output_len()
            .map_err(capacity_to_io_error)?;
        self.output.ensure_spare_capacity(required)?;
        let (units, unit_index, available) = self.output.spare_raw_parts_mut();
        debug_assert!(available >= required, "insufficient finish capacity");
        let written = encoder
            .finish(units, unit_index)
            .map_err(|error| finish_to_io_error(error, map_error))?;
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
    /// # Parameters
    ///
    /// * `encoder` - Encoder whose final units are being collected.
    /// * `map_error` - Function mapping encoder errors into I/O errors.
    ///
    /// # Errors
    ///
    /// Returns capacity, encoder finalization, or wrapped output flush errors.
    pub fn finish<E, M, Value>(
        &mut self,
        encoder: &mut E,
        map_error: &mut M,
    ) -> Result<()>
    where
        E: BufferedTranscoder<Value, O::Item>,
        M: FnMut(E::Error) -> Error,
    {
        self.finish_encoder(encoder, map_error)?;
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
