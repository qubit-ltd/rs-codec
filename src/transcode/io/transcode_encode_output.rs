// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Buffered output driver that encodes values into units.

use core::fmt;
use std::collections::TryReserveError;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::result::Result as CoreResult;

use qubit_io::BufferedOutput;
use qubit_io::BufferedOutputRecovery;
use qubit_io::ByteOutput;
use qubit_io::Output;
use qubit_io::SeekableByteOutput;
use qubit_utils::SliceRange;
use qubit_utils::allocation_error;

use super::internal::capacity_error_to_invalid_data;
use super::internal::map_encode_value_result;
use super::transcode_progress_validation::validate_encode_progress;
use crate::CapacityError;
use crate::Codec;
use crate::TranscodeEncodeErrorOf;
use crate::TranscodeEncoder;
use crate::TranscodeProgress;
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
    /// errors from the wrapped output while delivering pending units.
    pub fn ensure_spare_capacity(&mut self, count: usize) -> Result<()> {
        self.output.try_reserve_capacity(count).map_err(allocation_error)?;
        self.output.ensure_spare_capacity(count)
    }

    /// Consumes this adapter without flushing the wrapped output.
    ///
    /// This method does not call [`Self::flush`] and performs no I/O. `Pending`
    /// units were not reported as written and may be retried. `Indeterminate`
    /// means an inner write panicked and may have produced an external prefix;
    /// do not replay it automatically.
    ///
    /// # Returns
    ///
    /// The wrapped output and pending or indeterminate units.
    #[must_use = "the returned output and recovery status must be handled"]
    #[inline]
    pub fn into_parts(self) -> (O, BufferedOutputRecovery<O::Item>) {
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
    /// output before calling `encoder`. Preparing that space may deliver
    /// previously buffered units to the wrapped output and can return an I/O
    /// error before `encoder.reset` runs. This delivery does not call the
    /// wrapped output's `flush` method.
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

    /// Encodes at most one step of the selected input range.
    ///
    /// Unlike [`Self::transcode`], this method does not reserve output space
    /// for the entire range and calls the encoder exactly once. The returned
    /// progress is relative to this call's input and output windows. Advance
    /// the input index by [`TranscodeProgress::read`]; if the status is
    /// `NeedOutput`, provide the requested spare capacity before retrying.
    /// Written units have already been committed to this adapter's pending
    /// buffer and must not be advanced again. A zero count validates the range
    /// and returns complete progress without delivering pending units or
    /// calling the encoder.
    ///
    /// # Parameters
    ///
    /// * `encoder` - Encoder that processes one progress step.
    /// * `map_error` - Maps encoder errors to I/O errors.
    /// * `input` - Source values.
    /// * `input_index` - Start index inside `input`.
    /// * `count` - Number of values available from `input_index`.
    ///
    /// # Returns
    ///
    /// Progress for the single encoder call, including consumed input,
    /// committed output, and whether more output capacity is needed.
    ///
    /// # Errors
    ///
    /// Returns invalid input ranges, capacity, encoder, or output errors. An
    /// invalid encoder progress report is returned as `ErrorKind::InvalidData`.
    /// An error does not provide general transactional rollback or complete
    /// progress. The encoder may already have advanced, but units written by
    /// its failing call are not committed to this adapter's pending output.
    /// Do not replay the original input range without an encoder-specific
    /// recovery protocol. Earlier pending units retain the wrapped output's
    /// recovery contract; reset starts a new logical stream and does not undo
    /// output already delivered.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// # use core::convert::Infallible;
    /// # use core::num::NonZeroUsize;
    /// # use qubit_codec::Codec;
    /// # use qubit_codec::CodecTranscodeEncoder;
    /// # use qubit_codec::DecodeFailure;
    /// use qubit_codec::TranscodeEncodeOutput;
    /// use qubit_codec::TranscodeEncoder;
    /// use qubit_codec::TranscodeStatus;
    ///
    /// # struct ByteCodec;
    /// # impl Codec for ByteCodec {
    /// #     type Value = u8;
    /// #     type Unit = u8;
    /// #     type DecodeError = Infallible;
    /// #     type EncodeError = Infallible;
    /// #     const MIN_UNITS_PER_VALUE: usize = 1;
    /// #     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    /// #     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
    /// #     unsafe fn decode(
    /// #         &mut self,
    /// #         input: &[u8],
    /// #         index: usize,
    /// #     ) -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
    /// #         Ok((input[index], NonZeroUsize::MIN))
    /// #     }
    /// #     unsafe fn encode(
    /// #         &mut self,
    /// #         value: &u8,
    /// #         output: &mut [u8],
    /// #         index: usize,
    /// #     ) -> Result<usize, Infallible> {
    /// #         output[index] = *value;
    /// #         Ok(1)
    /// #     }
    /// # }
    /// fn encode<E>(encoder: &mut E, values: &[E::Input]) -> std::io::Result<Vec<u8>>
    /// where
    ///     E: TranscodeEncoder<Output = u8>,
    /// {
    ///     let mut output = TranscodeEncodeOutput::with_capacity(Cursor::new(Vec::new()), 8);
    ///     let mut map_error = |_| std::io::Error::other("encode error");
    ///     output.reset(encoder, &mut map_error)?;
    ///     let mut offset = 0;
    ///     while offset < values.len() {
    ///         let progress = output.transcode_step(
    ///             encoder,
    ///             &mut map_error,
    ///             values,
    ///             offset,
    ///             values.len() - offset,
    ///         )?;
    ///         offset += progress.read();
    ///         if let TranscodeStatus::NeedOutput { required, .. } = progress.status() {
    ///             output.ensure_spare_capacity(required.get())?;
    ///         }
    ///     }
    ///     output.finish(encoder, &mut map_error)?;
    ///     let (cursor, _) = output.into_parts();
    ///     Ok(cursor.into_inner())
    /// }
    ///
    /// let mut encoder = CodecTranscodeEncoder::new(ByteCodec);
    /// assert_eq!(encode(&mut encoder, &[1, 2, 3]).unwrap(), vec![1, 2, 3]);
    /// let mut empty_encoder = CodecTranscodeEncoder::new(ByteCodec);
    /// assert_eq!(encode(&mut empty_encoder, &[]).unwrap(), Vec::<u8>::new());
    /// ```
    pub fn transcode_step<E, M, Value>(
        &mut self,
        encoder: &mut E,
        map_error: &mut M,
        input: &[Value],
        input_index: usize,
        count: usize,
    ) -> Result<TranscodeProgress>
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
            return Ok(TranscodeProgress::complete(0, 0));
        }
        let required = encoder
            .max_transcode_output_len(1)
            .map_err(capacity_error_to_invalid_data)?
            .max(1);
        self.ensure_spare_capacity(required)?;
        let (units, output_index, available_output) = self.output.spare_raw_parts_mut();
        let progress = encoder
            .transcode(&input[..input_end], input_index, units, output_index)
            .map_err(&mut *map_error)?;
        let progress = validate_encode_progress(progress, input_index, count, output_index, available_output)?;
        // SAFETY: Validated progress fits the spare output window.
        unsafe {
            self.output.advance(progress.written());
        }
        Ok(progress)
    }

    /// Encodes values from an indexed input range using a streaming
    /// [`Transcoder`].
    ///
    /// This streaming path reserves the encoder's conservative output bound
    /// for the whole batch before calling it. Preparing that space may deliver
    /// previously buffered units to the wrapped output and can return an I/O
    /// error before consuming source values. Split large batches into smaller
    /// calls when you need to limit the memory required for one operation.
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
    /// Returns invalid input ranges, capacity, transcode, or output errors. An
    /// output-bound contract violation is reported as `ErrorKind::InvalidData`.
    /// An error does not provide general transactional rollback or complete
    /// progress. The encoder may already have advanced, while units written by
    /// its failing call are not committed to this adapter's pending output.
    /// Do not replay the original input range unless the concrete encoder
    /// provides an explicit recovery protocol. Previously committed pending
    /// output remains subject to the wrapped output's recovery contract, and
    /// reset starts a new logical stream without undoing output already
    /// delivered.
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
        let required = encoder
            .max_transcode_output_len(count)
            .map_err(capacity_error_to_invalid_data)?
            .max(1);
        self.ensure_spare_capacity(required)?;
        let input = &input[..input_end];
        let mut read_total = 0;
        while read_total < count {
            let (units, output_index, available_output) = self.output.spare_raw_parts_mut();
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
            if let TranscodeStatus::NeedOutput { required, .. } = progress.status()
                && (read_total == count || available_output - written < required.get() || (read == 0 && written == 0))
            {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "encoder output bound was insufficient",
                ));
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
    /// errors. It does not perform output I/O; use [`Self::finish`] to deliver
    /// the buffered final units and flush the wrapped output.
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
        let pending = self.output.capacity() - self.output.spare_capacity();
        let capacity = pending
            .checked_add(required)
            .ok_or_else(|| capacity_error_to_invalid_data(CapacityError::OutputLengthOverflow))?;
        self.output.try_reserve_capacity(capacity).map_err(allocation_error)?;
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
}

impl<O> TranscodeEncodeOutput<O>
where
    O: SeekableByteOutput,
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
    O: ByteOutput,
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
    #[inline]
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
            let count = match Output::write(&mut self.output, &input[written..]) {
                Ok(count) => count,
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            };
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
    #[inline]
    fn flush(&mut self) -> Result<()> {
        TranscodeEncodeOutput::flush(self)
    }
}

impl<O> Seek for TranscodeEncodeOutput<O>
where
    O: SeekableByteOutput,
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
