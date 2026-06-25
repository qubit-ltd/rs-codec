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
/// This type adds adapter-level domain failures that cannot be represented by
/// the wrapped codec itself, such as closed-input incomplete values and
/// trailing units in exact-value decodes. Buffer index and capacity failures
/// are represented by [`crate::TranscodeError`].
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

    /// The wrapped codec reported an error while resetting decode state.
    #[error("codec decode reset error: {source}")]
    DecodeReset {
        /// Error returned by [`crate::Codec::decode_reset`].
        #[source]
        source: E,
    },

    /// The wrapped codec reported an error while flushing decode state.
    #[error("codec decode flush error: {source}")]
    DecodeFlush {
        /// Error returned by [`crate::Codec::decode_flush`].
        #[source]
        source: E,
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
    #[error(
        "trailing input after decoded value: consumed {consumed} units, remaining {remaining}"
    )]
    TrailingInput {
        /// Units consumed by the decoded value.
        consumed: usize,
        /// Extra units left after the decoded value.
        remaining: usize,
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
    #[inline(always)]
    #[must_use]
    pub const fn decode(source: E, input_index: usize) -> Self {
        Self::Decode {
            source,
            input_index,
        }
    }

    /// Creates an error wrapping a codec-specific decode-reset error.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by [`crate::Codec::decode_reset`].
    ///
    /// # Returns
    ///
    /// Returns a codec decode-reset error wrapper.
    #[inline(always)]
    #[must_use]
    pub const fn decode_reset(source: E) -> Self {
        Self::DecodeReset { source }
    }

    /// Creates an error wrapping a codec-specific decode-flush error.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by [`crate::Codec::decode_flush`].
    ///
    /// # Returns
    ///
    /// Returns a codec decode-flush error wrapper.
    #[inline(always)]
    #[must_use]
    pub const fn decode_flush(source: E) -> Self {
        Self::DecodeFlush { source }
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
    #[inline(always)]
    #[must_use]
    pub const fn incomplete(
        input_index: usize,
        required_total: usize,
        available: usize,
    ) -> Self {
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
    #[inline(always)]
    #[must_use]
    pub const fn trailing_input(consumed: usize, remaining: usize) -> Self {
        Self::TrailingInput {
            consumed,
            remaining,
        }
    }

    /// Extracts the wrapped codec source error, when this variant has one.
    ///
    /// # Returns
    ///
    /// Returns `Some(source)` for codec decode, reset, and flush failures.
    /// Returns `None` for adapter-only failures.
    #[inline(always)]
    #[must_use]
    pub fn into_source(self) -> Option<E> {
        match self {
            Self::Decode { source, .. }
            | Self::DecodeReset { source }
            | Self::DecodeFlush { source } => Some(source),
            Self::Incomplete { .. } | Self::TrailingInput { .. } => None,
        }
    }

    /// Returns whether this error indicates an incomplete input prefix.
    ///
    /// # Returns
    ///
    /// Returns `true` only for the [`Incomplete`](Self::Incomplete) variant.
    #[inline(always)]
    #[must_use]
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
    #[inline]
    #[must_use]
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
}
