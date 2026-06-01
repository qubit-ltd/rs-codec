/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Generic decode error used by codec adapters.

use thiserror::Error;

/// Error reported by codec-backed value and buffered decoder adapters.
///
/// The wrapped codec remains responsible for domain-specific decode failures.
/// This type adds adapter-level failures that cannot be represented by the
/// wrapped codec itself, such as a value decoder receiving too few units before
/// it can safely call [`crate::Codec::decode_unchecked`].
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
    #[error("incomplete input at index {input_index}: required {required_total} units, available {available}")]
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
    pub const fn decode(source: E, input_index: usize) -> Self {
        Self::Decode { source, input_index }
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
    pub const fn trailing_input(consumed: usize, remaining: usize) -> Self {
        Self::TrailingInput { consumed, remaining }
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
    pub const fn invalid_input_index(index: usize, len: usize) -> Self {
        Self::InvalidInputIndex { index, len }
    }
}
