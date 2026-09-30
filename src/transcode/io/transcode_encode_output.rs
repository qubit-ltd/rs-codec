// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Buffered output driver that encodes values into units.

use core::fmt;
use core::num::NonZeroUsize;
use core::result::Result as CoreResult;
use std::collections::TryReserveError;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

use qubit_io::Buffer;
use qubit_io::BufferedOutput;
use qubit_io::Output;
use qubit_io::Seekable;
use qubit_utils::SliceRange;
use qubit_utils::allocation_error;

use super::transcode_progress_validation::validate_encode_progress;
use crate::CapacityError;
use crate::Codec;
use crate::TranscodeEncodeError;
use crate::TranscodeEncodeErrorOf;
use crate::TranscodeEncoder;
use crate::TranscodeStatus;
use crate::Transcoder;
use crate::value::codec_value_lifecycle::encode_complete_value_into_reserved;
use crate::value::codec_value_lifecycle::max_complete_encode_units;

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
///
/// # Examples
///
/// ```
/// use std::io::Cursor;
///
/// use qubit_codec::TranscodeEncodeOutput;
///
/// let output = TranscodeEncodeOutput::with_capacity(Cursor::new(Vec::<u8>::new()), 8);
/// assert!(output.spare_capacity() >= 8);
/// ```
pub struct TranscodeEncodeOutput<O>
where
    O: Output,
    O::Item: Copy + Default,
{
    /// Owns the wrapped unit sink and pending encoded units awaiting delivery.
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
    ///
    /// # Returns
    ///
    /// A buffered adapter owning `inner` on success.
    pub fn try_with_capacity(inner: O, capacity: usize) -> CoreResult<Self, TryReserveError> {
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
    #[inline]
    pub const fn inner(&self) -> &O {
        self.output.inner()
    }

    /// Returns the available capacity of the spare output buffer.
    ///
    /// # Returns
    ///
    /// The number of output units that can still be appended without flushing.
    #[must_use]
    #[inline]
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
    #[inline]
    pub fn spare_raw_parts_mut(&mut self) -> (&mut [O::Item], usize, usize) {
        self.output.spare_raw_parts_mut()
    }

    /// Marks `count` units from [`Self::spare_raw_parts_mut`] as written.
    ///
    /// # Parameters
    ///
    /// * `count` - Initialized units to commit from the spare window.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `count <= Self::spare_capacity()` and
    /// that the corresponding units in the returned spare slice have been
    /// initialized.
    #[inline]
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
            .map_err(allocation_error)?;
        self.output.ensure_spare_capacity(count)
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
    #[inline]
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
    /// # Type Parameters
    ///
    /// * `C` - Codec producing the wrapped output unit type.
    /// * `M` - Mapper from codec errors into I/O errors.
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
    pub fn write_encoded_with<C, M>(&mut self, codec: &mut C, value: &C::Value, mut map_error: M) -> Result<()>
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
    /// # Type Parameters
    ///
    /// * `E` - Transcoder producing the wrapped output unit type.
    /// * `M` - Mapper from transcoder errors into I/O errors.
    /// * `Value` - Source value type accepted by the transcoder.
    ///
    /// # Parameters
    ///
    /// * `encoder` - Transcoder whose stream prefix is being collected.
    /// * `map_error` - Mapper for reset failures.
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
        let required = encoder.max_reset_output_len().map_err(capacity_error_to_invalid_data)?;
        self.ensure_spare_capacity(required)?;
        let (units, output_index, available) = self.output.spare_raw_parts_mut();
        debug_assert!(
            available >= required,
            "insufficient reset capacity reserved in spare output buffer",
        );
        let written = encoder.reset(units, output_index).map_err(&mut *map_error)?;
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
    /// # Type Parameters
    ///
    /// * `E` - Encoder producing the wrapped output unit type.
    /// * `M` - Mapper from encoder errors into I/O errors.
    /// * `Value` - Source value type accepted by the encoder.
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
    /// use std::io::Error;
    /// use std::io::sink;
    ///
    /// use qubit_codec::CapacityError;
    /// use qubit_codec::TranscodeEncodeError;
    /// use qubit_codec::TranscodeEncodeOutput;
    /// use qubit_codec::TranscodeProgress;
    /// use qubit_codec::Transcoder;
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
    /// let mut output = TranscodeEncodeOutput::with_capacity(sink(), 1);
    /// let mut transcoder = GenericTranscoder;
    /// let mut map_error = |_| Error::other("transcode error");
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
        let input_end = SliceRange::checked_range_end(
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
            let progress = validate_encode_progress(
                progress,
                input_index + read_total,
                remaining_input,
                output_index,
                available_output,
            )?;
            let read = progress.read();
            let written = progress.written();
            // SAFETY: The progress bounds check above proved that the encoder
            // initialized no more than the available spare output window.
            unsafe {
                self.output.advance(written);
            }
            read_total += read;
            if progress.is_complete() {
                return Ok(read_total);
            }
            if let TranscodeStatus::NeedOutput { required, .. } = progress.status() {
                required_spare = required;
                if read_total == count {
                    self.ensure_transcode_spare_capacity(required)?;
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
    /// # Type Parameters
    ///
    /// * `E` - Transcoder producing the wrapped output unit type.
    /// * `M` - Mapper from transcoder errors into I/O errors.
    /// * `Value` - Source value type accepted by the transcoder.
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
    ///
    /// # Panics
    ///
    /// Panics when the encoder exceeds its declared finish-output bound.
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
    /// # Type Parameters
    ///
    /// * `E` - Transcoder producing the wrapped output unit type.
    /// * `M` - Mapper from transcoder errors into I/O errors.
    /// * `Value` - Source value type accepted by the transcoder.
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
    pub fn finish_to_buffer<E, M, Value>(&mut self, encoder: &mut E, map_error: &mut M) -> Result<()>
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
        let written = encoder.finish(units, output_index).map_err(&mut *map_error)?;
        assert!(written <= required, "finish wrote beyond its bound");
        // SAFETY: The encoder reported initialized units within the spare
        // range that was reserved above.
        unsafe {
            self.output.advance(written);
        }
        Ok(())
    }

    /// Ensures enough output capacity for one non-zero required unit count.
    ///
    /// The non-zero wrapper preserves the engine's progress contract while
    /// this adapter performs any necessary allocation.
    ///
    /// # Parameters
    ///
    /// * `required` - Minimum writable unit count requested by the engine.
    ///
    /// # Errors
    ///
    /// Propagates buffer allocation or wrapped-output flush errors.
    fn ensure_transcode_spare_capacity(&mut self, required: NonZeroUsize) -> Result<()> {
        self.ensure_spare_capacity(required.get())
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
    /// Writes raw bytes through the internal buffer without encoding them.
    ///
    /// # Parameters
    ///
    /// * `input` - Bytes to append to the buffered unit output.
    ///
    /// # Returns
    ///
    /// The number of bytes accepted, which may be shorter than `input`.
    ///
    /// # Errors
    ///
    /// Propagates wrapped-output errors when buffer delivery is needed.
    fn write(&mut self, input: &[u8]) -> Result<usize> {
        Output::write(&mut self.output, input)
    }

    /// Writes all raw bytes through the internal buffer without encoding them.
    ///
    /// # Parameters
    ///
    /// * `input` - Bytes that must all be accepted before success.
    ///
    /// # Errors
    ///
    /// Propagates output errors and returns `WriteZero` if output stops
    /// progressing.
    ///
    /// # Panics
    ///
    /// Panics when the wrapped output reports more bytes than supplied.
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

    /// Flushes buffered bytes to the wrapped output without finishing an
    /// encoder.
    ///
    /// # Errors
    ///
    /// Propagates wrapped-output flush errors.
    fn flush(&mut self) -> Result<()> {
        TranscodeEncodeOutput::flush(self)
    }
}

impl<O> Seek for TranscodeEncodeOutput<O>
where
    O: Output<Item = u8> + Seekable<Unit = u8>,
{
    /// Flushes pending bytes, then seeks the wrapped byte output.
    ///
    /// # Parameters
    ///
    /// * `position` - Target position relative to the selected seek origin.
    ///
    /// # Returns
    ///
    /// The new absolute byte position.
    ///
    /// # Errors
    ///
    /// Propagates flush or seek errors from the wrapped output.
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
    /// Formats the owned buffer and underlying output for diagnostics.
    ///
    /// # Parameters
    ///
    /// * `formatter` - Destination and formatting options.
    ///
    /// # Errors
    ///
    /// Returns a formatting error if the destination rejects output.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TranscodeEncodeOutput")
            .field("output", &self.output)
            .finish()
    }
}

/// Wraps a streaming capacity failure as `InvalidData`, retaining its source.
///
/// # Parameters
///
/// * `error` - Capacity planning failure from a streaming transcoder.
///
/// # Returns
///
/// An owned I/O error carrying the original capacity error.
fn capacity_error_to_invalid_data(error: CapacityError) -> Error {
    Error::new(ErrorKind::InvalidData, error)
}

/// Preserves successful values and maps one-value failures into I/O errors.
///
/// # Type Parameters
///
/// * `T` - Successful result value.
/// * `E` - Codec-domain error passed to the mapper.
/// * `Value` - Source value type carried by an unencodable error.
///
/// # Parameters
///
/// * `result` - Codec lifecycle result to translate.
/// * `map_error` - Called only for a codec-domain failure.
///
/// # Returns
///
/// The original successful value without transformation.
///
/// # Errors
///
/// Returns mapped domain errors or `InvalidInput` for bounds and domain
/// rejection.
fn map_encode_value_result<T, E, Value>(
    result: CoreResult<T, TranscodeEncodeError<E, Value>>,
    map_error: &mut dyn FnMut(E) -> Error,
) -> Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => Err(map_encode_value_error(error, map_error)),
    }
}

/// Maps domain failures through the caller and other failures to
/// `InvalidInput`.
///
/// # Type Parameters
///
/// * `E` - Codec-domain error passed to the mapper.
/// * `Value` - Rejected source value type, discarded during I/O translation.
///
/// # Parameters
///
/// * `error` - Encoding failure consumed by this translation.
/// * `map_error` - Invoked only for the `Domain` variant.
///
/// # Returns
///
/// An I/O error retaining mapped domain detail or a bound/rejection message.
#[inline(never)]
fn map_encode_value_error<E, Value>(
    error: TranscodeEncodeError<E, Value>,
    map_error: &mut dyn FnMut(E) -> Error,
) -> Error {
    match error {
        TranscodeEncodeError::Domain(error) => map_error(error.into_source()),
        TranscodeEncodeError::Unencodable { .. } => Error::new(ErrorKind::InvalidInput, "codec cannot encode value"),
        TranscodeEncodeError::Failure(_) => Error::new(ErrorKind::InvalidInput, "codec output bound overflow"),
    }
}
