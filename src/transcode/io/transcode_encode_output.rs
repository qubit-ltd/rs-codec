// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Buffered output driver that encodes values into units.

use core::{fmt, num::NonZeroUsize};
use std::collections::TryReserveError;
use std::io::{Error, ErrorKind, Result, Seek, SeekFrom, Write};

use qubit_io::{Buffer, BufferedOutput, Output, Seekable, UncheckedSlice};

use crate::{
    CapacityError, Codec, TranscodeEncodeError, TranscodeEncodeErrorOf, TranscodeEncoder,
    Transcoder,
    value::codec_value_lifecycle::{
        encode_complete_value_into_reserved, max_complete_encode_units,
    },
};

use super::transcode_progress_driver::{EncodeStep, encode_progress};

/// Encodes an [`Output`] value stream into an [`Output`] unit stream.
///
/// This type owns only the unit-level [`qubit_io::BufferedOutput`]. Callers
/// pass a [`crate::Codec`] and error mapper to each encode operation, which
/// lets one buffered output drive different encoders without nesting buffers or
/// storing codec-specific state in the buffer owner.
///
/// [`Self::flush`] only drains already buffered units. State-aware streaming
/// encoders can use [`Self::reset`], [`Self::transcode`], and [`Self::finish`]
/// explicitly.
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
    #[must_use]
    pub fn with_capacity(inner: O, capacity: usize) -> Self {
        Self {
            output: BufferedOutput::with_capacity(inner, capacity),
        }
    }

    /// Creates an encoder output with a unit buffer of at least `capacity`.
    ///
    /// # Parameters
    ///
    /// * `inner` - Unit output written by this adapter.
    /// * `capacity` - Requested internal unit buffer capacity.
    ///
    /// # Errors
    ///
    /// Returns an allocation error when the requested buffer cannot be
    /// allocated.
    #[inline]
    pub fn try_with_capacity(
        inner: O,
        capacity: usize,
    ) -> std::result::Result<Self, TryReserveError> {
        Ok(Self {
            output: BufferedOutput::try_with_capacity(inner, capacity)?,
        })
    }

    /// Returns a shared reference to the wrapped unit output.
    ///
    /// Pending units remain in this adapter's internal buffer and are not
    /// visible through the returned output until [`Self::flush`] succeeds.
    ///
    /// # Returns
    ///
    /// A shared reference to the wrapped unit output.
    #[must_use]
    pub const fn inner(&self) -> &O {
        self.output.inner()
    }

    /// Returns the available capacity of the spare output buffer.
    ///
    /// # Returns
    ///
    /// The number of output units that can still be appended without flushing.
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
    /// Returns allocation errors mapped to [`ErrorKind::OutOfMemory`], or I/O
    /// errors from the wrapped output while flushing pending units.
    pub fn ensure_spare_capacity(&mut self, count: usize) -> Result<()> {
        let pending = self.output.capacity() - self.output.spare_capacity();
        let required_capacity = pending.saturating_add(count);
        self.output
            .try_reserve_capacity(required_capacity)
            .map_err(allocation_to_io_error)?;
        self.output.ensure_spare_capacity(count)
    }

    fn ensure_transcode_spare_capacity(&mut self, required: NonZeroUsize) -> Result<()> {
        self.ensure_spare_capacity(required.get())
    }

    /// Consumes this adapter without flushing the wrapped output.
    ///
    /// This method does not call [`Self::flush`] and performs no I/O. Pending
    /// units remain in the returned buffer, which transfers their delivery
    /// responsibility to the caller.
    ///
    /// # Returns
    ///
    /// The wrapped output and the buffer holding pending units.
    #[must_use = "the returned output and pending buffer must be handled"]
    pub fn into_parts(self) -> (O, Buffer<O::Item>) {
        self.output.into_parts()
    }

    /// Flushes buffered units without finishing any encoder stream.
    ///
    /// # Errors
    ///
    /// Returns errors from the wrapped output while flushing pending units.
    pub fn flush(&mut self) -> Result<()> {
        self.output.flush()
    }

    /// Encodes one codec value into this buffered unit output.
    ///
    /// The method grows the persistent internal buffer when necessary, then
    /// writes the complete encoded value into its spare window.
    ///
    /// # Parameters
    ///
    /// * `codec` - Codec used for this single-value encode.
    /// * `value` - Value to encode.
    /// * `map_error` - Mapper for codec-domain encode errors.
    ///
    /// # Errors
    ///
    /// Returns I/O errors from the wrapped output, `InvalidInput` when the
    /// codec output bound overflows or the value is outside the codec domain,
    /// or the error returned by `map_error` for codec encode, reset, and finish
    /// failures.
    ///
    /// # Panics
    ///
    /// Panics when the codec violates its declared reset, value-width, or
    /// finish bounds, or when `encode` writes a length different from
    /// `encode_len` in the same reset state.
    pub fn write_encoded_with<C, M>(
        &mut self,
        codec: &mut C,
        value: &C::Value,
        mut map_error: M,
    ) -> Result<()>
    where
        C: Codec<Unit = O::Item>,
        M: FnMut(C::EncodeError) -> Error,
    {
        let max_units = map_encode_value_result(
            max_complete_encode_units::<C>().map_err(TranscodeEncodeErrorOf::<C>::from),
            &mut map_error,
        )?;
        self.ensure_spare_capacity(max_units)?;
        let (units, output_index, available) = self.output.spare_raw_parts_mut();
        debug_assert!(
            available >= max_units,
            "reserved spare buffer is smaller than codec upper bound",
        );
        let written = map_encode_value_result(
            encode_complete_value_into_reserved(codec, value, units, output_index, max_units),
            &mut map_error,
        )?;
        // SAFETY: The spare buffer has the conservative complete lifecycle
        // capacity, and the helper reports how many initialized units it
        // actually wrote.
        unsafe {
            self.output.advance(written);
        }
        Ok(())
    }

    /// Runs encoder reset and buffers any stream-prefix units.
    ///
    /// This method reserves enough persistent buffer capacity for all reset
    /// output before calling `encoder`. It does not flush pending units.
    ///
    /// # Errors
    ///
    /// Returns capacity errors, allocation errors, or reset errors mapped by
    /// `map_error`.
    ///
    /// # Panics
    ///
    /// Panics when `encoder` writes more than its declared reset bound.
    pub fn reset<E, M, Value>(&mut self, encoder: &mut E, map_error: &mut M) -> Result<()>
    where
        E: Transcoder<Input = Value, Output = O::Item>,
        M: FnMut(E::Error) -> Error,
    {
        let required = encoder
            .max_reset_output_len()
            .map_err(capacity_error_to_invalid_data)?;
        self.ensure_spare_capacity(required)?;
        let (units, output_index, available) = self.output.spare_raw_parts_mut();
        debug_assert!(
            available >= required,
            "insufficient reset capacity reserved in spare output buffer",
        );
        let written = encoder
            .reset(units, output_index)
            .map_err(&mut *map_error)?;
        assert!(written <= required, "reset wrote beyond its bound");
        // SAFETY: The encoder reported initialized units within the spare
        // range reserved above.
        unsafe {
            self.output.advance(written);
        }
        Ok(())
    }

    /// Encodes values from an indexed input range using a streaming
    /// [`Transcoder`].
    ///
    /// This streaming path writes directly into the internal spare buffer and
    /// grows it for any larger `NeedOutput.required` value reported by
    /// `encoder`.
    ///
    /// # Parameters
    ///
    /// * `encoder` - Streaming encoder used for this operation.
    /// * `map_error` - Function mapping transcode errors into I/O errors.
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
    /// Returns invalid input ranges, capacity, transcode, or output errors.
    ///
    /// # Examples
    ///
    /// A value that only implements [`Transcoder`] is not an encoder and cannot
    /// be passed to this method.
    ///
    /// ```compile_fail
    /// use qubit_codec::{
    ///     CapacityError, TranscodeEncodeError, TranscodeEncodeOutput,
    ///     TranscodeProgress, Transcoder,
    /// };
    ///
    /// struct GenericTranscoder;
    ///
    /// impl Transcoder for GenericTranscoder {
    ///     type Input = u8;
    ///     type Output = u8;
    ///     type Error = TranscodeEncodeError<(), u8>;
    ///
    ///     fn max_transcode_output_len(
    ///         &self,
    ///         input_len: usize,
    ///     ) -> Result<usize, CapacityError> {
    ///         Ok(input_len)
    ///     }
    ///
    ///     fn reset(
    ///         &mut self,
    ///         _output: &mut [u8],
    ///         _output_index: usize,
    ///     ) -> Result<usize, Self::Error> {
    ///         Ok(0)
    ///     }
    ///
    ///     fn transcode(
    ///         &mut self,
    ///         input: &[u8],
    ///         input_index: usize,
    ///         _output: &mut [u8],
    ///         _output_index: usize,
    ///     ) -> Result<TranscodeProgress, Self::Error> {
    ///         Ok(TranscodeProgress::complete(input.len() - input_index, 0))
    ///     }
    ///
    ///     fn finish(
    ///         &mut self,
    ///         _output: &mut [u8],
    ///         _output_index: usize,
    ///     ) -> Result<usize, Self::Error> {
    ///         Ok(0)
    ///     }
    /// }
    ///
    /// let mut output = TranscodeEncodeOutput::with_capacity(std::io::sink(), 1);
    /// let mut transcoder = GenericTranscoder;
    /// let mut map_error = |_| std::io::Error::other("transcode error");
    /// let _ = output.transcode(
    ///     &mut transcoder,
    ///     &mut map_error,
    ///     &[0_u8],
    ///     0,
    ///     1,
    /// );
    /// ```
    pub fn transcode<E, M, Value>(
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
            self.ensure_transcode_spare_capacity(required_spare)?;
            let (units, output_index, available_output) = self.output.spare_raw_parts_mut();
            debug_assert!(
                available_output >= required_spare.get(),
                "reserved spare buffer is smaller than required encoder output",
            );
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
            // SAFETY: The progress bounds check above proved that the encoder
            // initialized no more than the available spare output window.
            unsafe {
                self.output.advance(written);
            }
            read_total += read;
            match progress.step {
                EncodeStep::Complete => return Ok(read_total),
                EncodeStep::NeedOutput(required) => {
                    required_spare = required;
                    if read_total == count {
                        self.ensure_transcode_spare_capacity(required)?;
                    }
                }
            }
        }
        Ok(read_total)
    }

    /// Finishes the encoder and flushes the wrapped unit output.
    ///
    /// This method writes final units directly into the internal spare buffer,
    /// growing it when [`Transcoder::max_finish_output_len`] exceeds the
    /// current capacity.
    ///
    /// # Parameters
    ///
    /// * `encoder` - Encoder whose final units are being collected.
    /// * `map_error` - Function mapping transcode errors into I/O errors.
    ///
    /// # Errors
    ///
    /// Returns capacity, transcode finalization, or wrapped output flush
    /// errors.
    pub fn finish<E, M, Value>(&mut self, encoder: &mut E, map_error: &mut M) -> Result<()>
    where
        E: Transcoder<Input = Value, Output = O::Item>,
        M: FnMut(E::Error) -> Error,
    {
        self.finish_to_buffer(encoder, map_error)?;
        self.output.flush()
    }

    /// Finishes the encoder while retaining final units in the output buffer.
    ///
    /// This method separates encoder finalization from output delivery. It is
    /// useful when a caller must record successful finalization before a later
    /// flush can fail. Call [`Self::flush`] to deliver the retained units.
    ///
    /// # Parameters
    ///
    /// * `encoder` - Encoder whose final units are being collected.
    /// * `map_error` - Function mapping transcode errors into I/O errors.
    ///
    /// # Errors
    ///
    /// Returns capacity planning, allocation, or transcoder finalization
    /// errors. It does not perform output I/O.
    ///
    /// # Panics
    ///
    /// Panics when `encoder` writes more units than its declared finish bound.
    pub fn finish_to_buffer<E, M, Value>(
        &mut self,
        encoder: &mut E,
        map_error: &mut M,
    ) -> Result<()>
    where
        E: Transcoder<Input = Value, Output = O::Item>,
        M: FnMut(E::Error) -> Error,
    {
        let required = match encoder.max_finish_output_len() {
            Ok(required) => required,
            Err(error) => return Err(capacity_error_to_invalid_data(error)),
        };
        self.ensure_spare_capacity(required)?;
        let (units, output_index, available) = self.output.spare_raw_parts_mut();
        debug_assert!(
            available >= required,
            "insufficient finish capacity reserved in spare output buffer",
        );
        let written = encoder
            .finish(units, output_index)
            .map_err(&mut *map_error)?;
        assert!(written <= required, "finish wrote beyond its bound");
        // SAFETY: The encoder reported initialized units within the spare
        // range that was reserved above.
        unsafe {
            self.output.advance(written);
        }
        Ok(())
    }
}

impl<O> TranscodeEncodeOutput<O>
where
    O: Output<Item = u8> + Seekable<Unit = u8>,
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
    pub fn seek(&mut self, position: SeekFrom) -> Result<u64> {
        self.output.seek_to(position)
    }
}

impl<O> Write for TranscodeEncodeOutput<O>
where
    O: Output<Item = u8>,
{
    /// Writes raw bytes through the internal buffer.
    fn write(&mut self, input: &[u8]) -> Result<usize> {
        Output::write(&mut self.output, input)
    }

    /// Writes all raw bytes through the internal buffer.
    fn write_all(&mut self, input: &[u8]) -> Result<()> {
        let mut written = 0;
        while written < input.len() {
            let count = Output::write(&mut self.output, &input[written..])?;
            if count == 0 {
                return Err(Error::from(ErrorKind::WriteZero));
            }
            assert!(
                count <= input.len() - written,
                "Output::write returned a count beyond the input length",
            );
            written += count;
        }
        Ok(())
    }

    /// Flushes buffered bytes to the wrapped output.
    fn flush(&mut self) -> Result<()> {
        TranscodeEncodeOutput::flush(self)
    }
}

impl<O> Seek for TranscodeEncodeOutput<O>
where
    O: Output<Item = u8> + Seekable<Unit = u8>,
{
    /// Flushes pending bytes, then seeks the wrapped byte output.
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

/// Converts an allocation failure into an I/O boundary error.
fn allocation_to_io_error(error: std::collections::TryReserveError) -> Error {
    Error::new(ErrorKind::OutOfMemory, error)
}

/// Maps a streaming capacity error into an I/O error.
fn capacity_error_to_invalid_data(error: CapacityError) -> Error {
    Error::new(ErrorKind::InvalidData, error)
}

/// Maps a one-value codec result into the I/O surface used by this adapter.
fn map_encode_value_result<T, E, Value>(
    result: core::result::Result<T, TranscodeEncodeError<E, Value>>,
    map_error: &mut dyn FnMut(E) -> Error,
) -> Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => Err(map_encode_value_error(error, map_error)),
    }
}

/// Maps one-value codec encode errors into the I/O surface used by this
/// adapter.
#[inline(never)]
fn map_encode_value_error<E, Value>(
    error: TranscodeEncodeError<E, Value>,
    map_error: &mut dyn FnMut(E) -> Error,
) -> Error {
    match error {
        TranscodeEncodeError::Domain(error) => map_error(error.into_source()),
        TranscodeEncodeError::Unencodable { .. } => {
            Error::new(ErrorKind::InvalidInput, "codec cannot encode value")
        }
        TranscodeEncodeError::Failure(_) => {
            Error::new(ErrorKind::InvalidInput, "codec output bound overflow")
        }
    }
}
