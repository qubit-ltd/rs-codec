// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Framework-level failures reported by safe transcode APIs.

use thiserror::Error;

use super::capacity_error::CapacityError;

/// Framework-level failure reported by a transcode operation.
///
/// These failures are part of the safe public transcode API. They describe
/// caller-supplied buffer ranges, capacity planning, and complete-input shape
/// that the transcode layer can detect without interpreting a concrete codec
/// error.
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum TranscodeFailure {
    /// The caller supplied an input index outside the input slice.
    #[error("invalid input index {index} for input length {input_len}")]
    InvalidInputIndex {
        /// Invalid input index supplied by the caller.
        index: usize,
        /// Length of the input slice.
        input_len: usize,
    },

    /// The caller supplied an output index outside the output slice.
    #[error("invalid output index {index} for output length {output_len}")]
    InvalidOutputIndex {
        /// Invalid output index supplied by the caller.
        index: usize,
        /// Length of the output slice.
        output_len: usize,
    },

    /// The output slice cannot hold all required output.
    #[error(
        "insufficient output at index {output_index}: required {required} units, available {available}"
    )]
    InsufficientOutput {
        /// Absolute output index where writing would start.
        output_index: usize,
        /// Output units required from `output_index`.
        required: usize,
        /// Output units available from `output_index`.
        available: usize,
    },

    /// Output length arithmetic overflowed.
    #[error("output length arithmetic overflow")]
    OutputLengthOverflow,

    /// The complete input ended with an incomplete value.
    #[error(
        "incomplete input at index {input_index}: required {required} units, available {available}"
    )]
    IncompleteInput {
        /// Absolute input index where the incomplete value starts.
        input_index: usize,
        /// Input units required to complete the value.
        required: usize,
        /// Input units available from `input_index`.
        available: usize,
    },

    /// The input contains exactly one decoded value plus trailing units.
    #[error(
        "trailing input after value: consumed {consumed} units, remaining {remaining}"
    )]
    TrailingInput {
        /// Units consumed by the decoded value.
        consumed: usize,
        /// Extra units left after the decoded value.
        remaining: usize,
    },
}

impl TranscodeFailure {
    /// Creates an invalid-input-index error.
    #[inline(always)]
    #[must_use]
    pub const fn invalid_input_index(index: usize, len: usize) -> Self {
        Self::InvalidInputIndex {
            index,
            input_len: len,
        }
    }

    /// Creates an invalid-output-index error.
    #[inline(always)]
    #[must_use]
    pub const fn invalid_output_index(index: usize, len: usize) -> Self {
        Self::InvalidOutputIndex {
            index,
            output_len: len,
        }
    }

    /// Creates an insufficient-output error.
    #[inline(always)]
    #[must_use]
    pub const fn insufficient_output(
        output_index: usize,
        required: usize,
        available: usize,
    ) -> Self {
        Self::InsufficientOutput {
            output_index,
            required,
            available,
        }
    }

    /// Creates an output-length-overflow error.
    #[inline(always)]
    #[must_use]
    pub const fn output_length_overflow() -> Self {
        Self::OutputLengthOverflow
    }

    /// Creates an incomplete-input error.
    #[inline(always)]
    #[must_use]
    pub const fn incomplete_input(
        input_index: usize,
        required: usize,
        available: usize,
    ) -> Self {
        Self::IncompleteInput {
            input_index,
            required,
            available,
        }
    }

    /// Creates a trailing-input error.
    #[inline(always)]
    #[must_use]
    pub const fn trailing_input(consumed: usize, remaining: usize) -> Self {
        Self::TrailingInput {
            consumed,
            remaining,
        }
    }

    /// Validates that `input_index` is within an input slice.
    #[inline]
    pub fn ensure_input_index(
        input_len: usize,
        input_index: usize,
    ) -> Result<(), Self> {
        if input_index > input_len {
            return Err(Self::invalid_input_index(input_index, input_len));
        }
        Ok(())
    }

    /// Validates that enough input units are available from `input_index`.
    #[inline]
    pub fn ensure_min_input(
        input_len: usize,
        input_index: usize,
        min_required: usize,
    ) -> Result<(), Self> {
        Self::ensure_input_index(input_len, input_index)?;
        let available = input_len - input_index;
        if available < min_required {
            return Err(Self::incomplete_input(
                input_index,
                min_required,
                available,
            ));
        }
        Ok(())
    }

    /// Validates that no input units remain after a decoded value.
    #[inline]
    pub fn ensure_no_trailing_input(
        consumed: usize,
        total: usize,
    ) -> Result<(), Self> {
        let remaining = total.saturating_sub(consumed);
        if remaining != 0 {
            return Err(Self::trailing_input(consumed, remaining));
        }
        Ok(())
    }

    /// Validates that `output_index` is within an output slice.
    #[inline]
    pub fn ensure_output_index(
        output_len: usize,
        output_index: usize,
    ) -> Result<(), Self> {
        if output_index > output_len {
            return Err(Self::invalid_output_index(output_index, output_len));
        }
        Ok(())
    }

    /// Validates input and output start indices for a transcode call.
    #[inline]
    pub fn ensure_transcode_indices(
        input_len: usize,
        input_index: usize,
        output_len: usize,
        output_index: usize,
    ) -> Result<(), Self> {
        Self::ensure_input_index(input_len, input_index)?;
        Self::ensure_output_index(output_len, output_index)
    }

    /// Validates that an output slice can hold one-shot finalization output.
    #[inline]
    pub fn ensure_output_capacity(
        output_len: usize,
        output_index: usize,
        required: usize,
    ) -> Result<(), Self> {
        Self::ensure_output_index(output_len, output_index)?;
        let available = output_len - output_index;
        if available < required {
            return Err(Self::insufficient_output(
                output_index,
                required,
                available,
            ));
        }
        Ok(())
    }

    /// Validates an indexed output range and its minimum writable capacity.
    #[inline]
    pub fn ensure_output_range(
        output_len: usize,
        output_index: usize,
        range_len: usize,
        required: usize,
    ) -> Result<(), Self> {
        Self::ensure_output_index(output_len, output_index)?;
        let available = output_len - output_index;
        if range_len > available {
            return Err(Self::invalid_output_index(output_index, output_len));
        }
        if range_len < required {
            return Err(Self::insufficient_output(
                output_index,
                required,
                range_len,
            ));
        }
        Ok(())
    }
}

impl From<CapacityError> for TranscodeFailure {
    /// Converts capacity planning errors into transcode framework failures.
    #[inline(always)]
    fn from(error: CapacityError) -> Self {
        match error {
            CapacityError::OutputLengthOverflow => Self::OutputLengthOverflow,
        }
    }
}
