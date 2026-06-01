/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Generic encode error used by codec-backed encoder adapters.

use thiserror::Error;

/// Error reported by codec-backed buffered encoder adapters.
///
/// The wrapped codec remains responsible for domain-specific encode failures.
/// This type adds adapter-level failures that cannot be represented by the
/// wrapped codec itself, such as a buffered encoder receiving an invalid input
/// start index.
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum CodecEncodeError<E> {
    /// The wrapped codec reported an encode error.
    #[error("codec encode error at input index {input_index}: {source}")]
    Encode {
        /// Error returned by the wrapped codec.
        #[source]
        source: E,
        /// Absolute input index of the value being encoded.
        input_index: usize,
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

impl<E> CodecEncodeError<E> {
    /// Creates an error wrapping a codec-specific encode error.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by the wrapped codec.
    /// - `input_index`: Absolute input index of the value being encoded.
    ///
    /// # Returns
    ///
    /// Returns a codec encode error wrapper.
    #[must_use]
    pub const fn encode(source: E, input_index: usize) -> Self {
        Self::Encode { source, input_index }
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
