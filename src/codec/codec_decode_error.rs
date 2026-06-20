// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Generic decode error used by codec adapters.

use thiserror::Error;

/// Error reported by codec-backed value and buffered decoder adapters.
///
/// The wrapped codec remains responsible for domain-specific decode failures.
/// This type adds adapter-level failures that cannot be represented by the
/// wrapped codec itself, such as a value decoder receiving too few units before
/// it can safely call [`crate::Codec::decode`] or a buffered decoder
/// receiving an invalid output start index.
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum CodecDecodeError<E> {
    /// The wrapped codec reported a decode error.
    #[error("codec decode error at input index {input_index}: {source}")]
    Decode {
        /// Error returned by the wrapped codec.
        #[source]
        source: E,
        /// Absolute input index at which the adapter called the wrapped codec.
        input_index: usize,
    },

    /// The adapter could not safely call the wrapped codec because input ended.
    #[error(
        "incomplete input at index {input_index}: required {required_total} units, available {available}"
    )]
    Incomplete {
        /// Absolute input index where the incomplete value starts.
        input_index: usize,
        /// Total units required from `input_index`.
        required_total: usize,
        /// Units available from `input_index`.
        available: usize,
    },

    /// A whole-value decode succeeded but left trailing input units.
    #[error("trailing input after decoded value: consumed {consumed} units, remaining {remaining}")]
    TrailingInput {
        /// Units consumed by the decoded value.
        consumed: usize,
        /// Extra units left after the decoded value.
        remaining: usize,
    },

    /// The caller supplied an input index outside the input slice.
    #[error("invalid input index {index} for input length {len}")]
    InvalidInputIndex {
        /// Invalid input index supplied by the caller.
        index: usize,
        /// Length of the input slice.
        len: usize,
    },

    /// The caller supplied an output index outside the output slice.
    #[error("invalid output index {index} for output length {len}")]
    InvalidOutputIndex {
        /// Invalid output index supplied by the caller.
        index: usize,
        /// Length of the output slice.
        len: usize,
    },

    /// The output slice cannot hold all output required by the adapter call.
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
}

impl<E> CodecDecodeError<E> {
    /// Creates an error wrapping a codec-specific decode error.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by the wrapped codec.
    /// - `input_index`: Absolute input index used for the codec call.
    ///
    /// # Returns
    ///
    /// Returns a codec decode error wrapper.
    #[must_use]
    #[inline(always)]
    pub const fn decode(source: E, input_index: usize) -> Self {
        Self::Decode {
            source,
            input_index,
        }
    }

    /// Creates an adapter-level incomplete-input error.
    ///
    /// # Parameters
    ///
    /// - `input_index`: Absolute input index where the incomplete value starts.
    /// - `required_total`: Total units required from `input_index`.
    /// - `available`: Units available from `input_index`.
    ///
    /// # Returns
    ///
    /// Returns an incomplete-input error.
    #[must_use]
    #[inline(always)]
    pub const fn incomplete(input_index: usize, required_total: usize, available: usize) -> Self {
        Self::Incomplete {
            input_index,
            required_total,
            available,
        }
    }

    /// Creates a trailing-input error for whole-value decoding.
    ///
    /// # Parameters
    ///
    /// - `consumed`: Units consumed by the decoded value.
    /// - `remaining`: Extra units left after the decoded value.
    ///
    /// # Returns
    ///
    /// Returns a trailing-input error.
    #[must_use]
    #[inline(always)]
    pub const fn trailing_input(consumed: usize, remaining: usize) -> Self {
        Self::TrailingInput {
            consumed,
            remaining,
        }
    }

    /// Creates an invalid-input-index error.
    ///
    /// # Parameters
    ///
    /// - `index`: Invalid input index supplied by the caller.
    /// - `len`: Length of the input slice.
    ///
    /// # Returns
    ///
    /// Returns an invalid-input-index error.
    #[must_use]
    #[inline(always)]
    pub const fn invalid_input_index(index: usize, len: usize) -> Self {
        Self::InvalidInputIndex { index, len }
    }

    /// Creates an invalid-output-index error.
    ///
    /// # Parameters
    ///
    /// - `index`: Invalid output index supplied by the caller.
    /// - `len`: Length of the output slice.
    ///
    /// # Returns
    ///
    /// Returns an invalid-output-index error.
    #[must_use]
    #[inline(always)]
    pub const fn invalid_output_index(index: usize, len: usize) -> Self {
        Self::InvalidOutputIndex { index, len }
    }

    /// Creates an insufficient-output error.
    #[must_use]
    #[inline(always)]
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

    /// Returns whether this error indicates an incomplete input prefix.
    ///
    /// # Returns
    ///
    /// Returns `true` only for the [`Incomplete`](Self::Incomplete) variant.
    #[must_use]
    #[inline(always)]
    pub const fn is_incomplete(&self) -> bool {
        matches!(self, Self::Incomplete { .. })
    }

    /// Returns the additional input units needed to make progress.
    ///
    /// This is a convenience accessor over the [`Incomplete`](Self::Incomplete)
    /// variant's fields. Streaming callers can use it to determine how many
    /// more units they must buffer before retrying a decode.
    ///
    /// # Returns
    ///
    /// Returns `Some(needed)` for [`Incomplete`](Self::Incomplete) errors,
    /// where `needed` is the strictly positive difference between the minimum
    /// units required and those already available. Returns `None` for all
    /// other variants.
    #[must_use]
    #[inline]
    pub fn needed_additional(&self) -> Option<core::num::NonZeroUsize> {
        match *self {
            Self::Incomplete {
                required_total,
                available,
                ..
            } => {
                let needed = required_total.saturating_sub(available);
                core::num::NonZeroUsize::new(needed)
            }
            _ => None,
        }
    }

    /// Validates that `input_index` is within an input slice.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Length of the input slice.
    /// - `input_index`: Input index supplied by the caller.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when `input_index <= input_len`.
    ///
    /// # Errors
    ///
    /// Returns an invalid-input-index error when `input_index` is beyond the
    /// slice.
    #[inline]
    pub fn ensure_input_index(input_len: usize, input_index: usize) -> Result<(), Self> {
        if input_index > input_len {
            return Err(Self::invalid_input_index(input_index, input_len));
        }
        Ok(())
    }

    /// Validates that `output_index` is within an output slice.
    ///
    /// # Parameters
    ///
    /// - `output_len`: Length of the output slice.
    /// - `output_index`: Output index supplied by the caller.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when `output_index <= output_len`.
    ///
    /// # Errors
    ///
    /// Returns an invalid-output-index error when `output_index` is beyond the
    /// slice.
    #[inline]
    pub fn ensure_output_index(output_len: usize, output_index: usize) -> Result<(), Self> {
        if output_index > output_len {
            return Err(Self::invalid_output_index(output_index, output_len));
        }
        Ok(())
    }

    /// Validates that an output slice can hold required adapter output.
    ///
    /// # Parameters
    ///
    /// - `output_len`: Length of the output slice.
    /// - `output_index`: Output index supplied by the caller.
    /// - `required`: Output units required from `output_index`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when output capacity is sufficient.
    ///
    /// # Errors
    ///
    /// Returns an invalid-output-index error when `output_index` is beyond the
    /// slice, or an insufficient-output error when fewer than `required` units
    /// are writable from `output_index`.
    #[inline]
    pub fn ensure_output_capacity(
        output_len: usize,
        output_index: usize,
        required: usize,
    ) -> Result<(), Self> {
        Self::ensure_output_index(output_len, output_index)?;
        let available = output_len - output_index;
        if available < required {
            return Err(Self::insufficient_output(output_index, required, available));
        }
        Ok(())
    }

    /// Validates that enough input units are available from `input_index`.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Length of the input slice.
    /// - `input_index`: Absolute input index where reading starts.
    /// - `min_required`: Minimum units required from `input_index`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when at least `min_required` units are available.
    ///
    /// # Errors
    ///
    /// Returns an incomplete-input error when fewer than `min_required` units
    /// are available from `input_index`.
    #[inline]
    pub fn ensure_min_input(
        input_len: usize,
        input_index: usize,
        min_required: usize,
    ) -> Result<(), Self> {
        let available = input_len.saturating_sub(input_index);
        if available < min_required {
            return Err(Self::incomplete(input_index, min_required, available));
        }
        Ok(())
    }

    /// Validates that decoding consumed the entire input slice.
    ///
    /// # Parameters
    ///
    /// - `consumed`: Units consumed by the decoded value.
    /// - `total`: Total units in the input slice.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when `consumed == total`.
    ///
    /// # Errors
    ///
    /// Returns a trailing-input error when extra units remain after the
    /// decoded value.
    #[inline]
    pub fn ensure_no_trailing_input(consumed: usize, total: usize) -> Result<(), Self> {
        let remaining = total.saturating_sub(consumed);
        if remaining != 0 {
            return Err(Self::trailing_input(consumed, remaining));
        }
        Ok(())
    }
}
