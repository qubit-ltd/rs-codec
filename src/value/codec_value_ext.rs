// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Value-level convenience methods for low-level codecs.

use core::num::NonZeroUsize;

use crate::{
    CapacityError,
    Codec,
    TranscodeError,
};

/// Extension trait for checked one-value codec operations.
///
/// `CodecValueExt` keeps convenience operations out of the low-level
/// [`Codec`] contract while still making them available to all codec
/// implementations. The methods compose primitive reset, encode, decode, and
/// finish hooks with capacity checks and adapter-level error wrapping.
pub trait CodecValueExt: Codec {
    /// Returns the maximum unit count emitted by one complete value encode.
    ///
    /// This is the checked sum of
    /// [`Codec::MAX_ENCODE_RESET_UNITS`],
    /// [`Codec::MAX_UNITS_PER_VALUE`], and
    /// [`Codec::MAX_ENCODE_FINISH_UNITS`]. It is useful for callers that want
    /// to reuse scratch storage for repeated one-value encodes.
    ///
    /// # Returns
    ///
    /// Returns the maximum reset-value-finish output length.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityError::OutputLengthOverflow`] when the sum cannot be
    /// represented as `usize`.
    #[inline(always)]
    #[must_use = "capacity planning can fail on overflow"]
    fn max_encode_value_units(&self) -> Result<usize, CapacityError> {
        Self::MAX_ENCODE_RESET_UNITS
            .checked_add(Self::MAX_UNITS_PER_VALUE)
            .and_then(|units| units.checked_add(Self::MAX_ENCODE_FINISH_UNITS))
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    /// Encodes one value through the complete encode lifecycle.
    ///
    /// The method validates the output index and the combined reset, value,
    /// and finish capacity before calling the unchecked codec hooks. It is a
    /// convenience wrapper for code paths that need one complete value and
    /// want to reuse caller-owned storage.
    ///
    /// # Parameters
    ///
    /// - `value`: Value to encode.
    /// - `output`: Destination unit buffer.
    /// - `output_index`: Start index in `output`.
    ///
    /// # Returns
    ///
    /// Returns the total number of reset, value, and finish units written.
    ///
    /// # Errors
    ///
    /// Returns [`TranscodeError`] when output bounds are invalid, when output
    /// capacity is insufficient, when output length arithmetic overflows, or
    /// when the codec cannot encode `value`.
    ///
    /// # Panics
    ///
    /// Panics when the codec writes or reports more units than its declared
    /// reset, value, or finish bound.
    fn encode_value_with_reset(
        &mut self,
        value: &Self::Value,
        output: &mut [Self::Unit],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::EncodeError, Self::Value>>
    where
        Self::Value: Clone,
    {
        if !self.can_encode_value(value) {
            return Err(TranscodeError::unencodable_value(0, value.clone()));
        }
        let reset_units = Self::MAX_ENCODE_RESET_UNITS;
        let value_units = self.encode_len(value);
        let required = reset_units
            .checked_add(value_units)
            .and_then(|units| units.checked_add(Self::MAX_ENCODE_FINISH_UNITS))
            .ok_or_else(TranscodeError::output_length_overflow)?;
        TranscodeError::ensure_output_capacity(
            output.len(),
            output_index,
            required,
        )?;

        let reset_written = unsafe {
            // SAFETY: The capacity check above reserves the combined
            // reset, value, and finish output bound at `output_index`.
            self.encode_reset(output, output_index)
        }
        .map_err(TranscodeError::domain_reset)?;
        assert!(
            reset_written <= reset_units,
            "Codec::encode_reset wrote beyond its reset bound",
        );

        let value_written = unsafe {
            // SAFETY: `reset_written <= reset_units` and the earlier combined
            // capacity check leave the exact value width writable.
            self.encode(value, output, output_index + reset_written)
        }
        .map_err(|error| TranscodeError::domain_main(error, 0))?;
        assert!(
            value_written == value_units,
            "Codec::encode wrote a different length than Codec::encode_len",
        );

        let finish_index = output_index + reset_written + value_written;
        let finish_written = unsafe {
            // SAFETY: The combined capacity check reserves the codec-declared
            // finish bound after the reset and exact value output.
            self.encode_finish(output, finish_index)
        }
        .map_err(TranscodeError::domain_finish)?;
        assert!(
            finish_written <= Self::MAX_ENCODE_FINISH_UNITS,
            "Codec::encode_finish wrote beyond its finish bound",
        );
        Ok(reset_written + value_written + finish_written)
    }

    /// Decodes one value and finishes decode-side state into caller storage.
    ///
    /// The method validates input and finish-output bounds before entering the
    /// unchecked codec hooks. It returns the decoded value, the consumed input
    /// count, and the number of finished values written to `finish_output`.
    ///
    /// # Parameters
    ///
    /// - `input`: Source unit buffer.
    /// - `input_index`: Start index in `input`.
    /// - `finish_output`: Destination value buffer for decode-finish output.
    /// - `finish_output_index`: Start index in `finish_output`.
    ///
    /// # Returns
    ///
    /// Returns `(value, consumed, finished)`.
    ///
    /// # Errors
    ///
    /// Returns [`TranscodeError`] when input or output bounds are invalid, when
    /// finish output capacity is insufficient, or when decoding or finishing
    /// fails. In this one-shot API, [`crate::DecodeFailure::Incomplete`] is a
    /// terminal [`crate::TranscodeFailure::IncompleteInput`] error rather than
    /// a resumable streaming status.
    ///
    /// # Panics
    ///
    /// Panics when the codec consumes beyond available input or finishes more
    /// values than its declared bound.
    fn decode_value_with_finish(
        &mut self,
        input: &[Self::Unit],
        input_index: usize,
        finish_output: &mut [Self::Value],
        finish_output_index: usize,
    ) -> Result<
        (Self::Value, NonZeroUsize, usize),
        TranscodeError<Self::DecodeError>,
    > {
        TranscodeError::ensure_min_input(
            input.len(),
            input_index,
            Self::MIN_UNITS_PER_VALUE,
        )?;

        let finish_cap = Self::MAX_DECODE_FINISH_VALUES;
        TranscodeError::ensure_output_capacity(
            finish_output.len(),
            finish_output_index,
            finish_cap,
        )?;

        let (value, consumed) = unsafe {
            // SAFETY: The input checks above guarantee the minimum readable
            // units required by `Codec::decode`.
            self.decode(input, input_index)
        }
        .map_err(|failure| {
            TranscodeError::from_decode_failure(
                failure,
                input_index,
                input.len() - input_index,
            )
        })?;
        let available = input.len() - input_index;
        assert!(
            consumed.get() <= available,
            "Codec::decode consumed beyond available input",
        );

        let finished = unsafe {
            // SAFETY: The finish-output checks above reserve the declared
            // finish output bound at `finish_output_index`.
            self.decode_finish(finish_output, finish_output_index)
        }
        .map_err(TranscodeError::domain_finish)?;
        assert!(
            finished <= finish_cap,
            "Codec::decode_finish wrote beyond its finish bound",
        );
        Ok((value, consumed, finished))
    }

    /// Decodes exactly one value and then finishes decode-side state.
    ///
    /// Unlike [`decode_value_with_finish`](Self::decode_value_with_finish),
    /// this helper requires the supplied input slice to contain exactly one
    /// encoded value. It validates trailing input before calling
    /// [`Codec::decode_finish`], preserving whole-value decoder semantics while
    /// still centralizing finish scratch-buffer handling.
    ///
    /// # Parameters
    ///
    /// - `input`: Source units for exactly one encoded value.
    /// - `finish_output`: Destination value buffer for decode-finish output.
    /// - `finish_output_index`: Start index in `finish_output`.
    ///
    /// # Returns
    ///
    /// Returns `(value, finished)`.
    ///
    /// # Errors
    ///
    /// Returns [`TranscodeError`] when finish output bounds are invalid or
    /// insufficient, when decoding fails, or when exact-value decode semantics
    /// are not satisfied. In this one-shot API,
    /// [`crate::DecodeFailure::Incomplete`] is a terminal
    /// [`crate::TranscodeFailure::IncompleteInput`] error rather than a
    /// resumable streaming status.
    ///
    /// # Panics
    ///
    /// Panics when the codec consumes beyond available input or finishes more
    /// values than its declared bound.
    fn decode_exact_value_with_finish(
        &mut self,
        input: &[Self::Unit],
        finish_output: &mut [Self::Value],
        finish_output_index: usize,
    ) -> Result<(Self::Value, usize), TranscodeError<Self::DecodeError>> {
        TranscodeError::ensure_min_input(
            input.len(),
            0,
            Self::MIN_UNITS_PER_VALUE,
        )?;

        let finish_cap = Self::MAX_DECODE_FINISH_VALUES;
        TranscodeError::ensure_output_capacity(
            finish_output.len(),
            finish_output_index,
            finish_cap,
        )?;

        let (value, consumed) = unsafe {
            // SAFETY: The input check above guarantees the minimum readable
            // units required by `Codec::decode` at index 0.
            self.decode(input, 0)
        }
        .map_err(|failure| {
            TranscodeError::from_decode_failure(failure, 0, input.len())
        })?;
        assert!(
            consumed.get() <= input.len(),
            "Codec::decode consumed beyond available input",
        );
        TranscodeError::ensure_no_trailing_input(consumed.get(), input.len())?;

        let finished = unsafe {
            // SAFETY: The finish-output checks above reserve the declared
            // finish output bound at `finish_output_index`.
            self.decode_finish(finish_output, finish_output_index)
        }
        .map_err(TranscodeError::domain_finish)?;
        assert!(
            finished <= finish_cap,
            "Codec::decode_finish wrote beyond its finish bound",
        );
        Ok((value, finished))
    }
}

impl<C> CodecValueExt for C where C: Codec + ?Sized {}
