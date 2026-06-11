// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Generic encode error used by codec-backed encoder adapters.

use crate::transcode::TranscodeError;
use thiserror::Error;

/// Error reported by codec-backed buffered encoder adapters.
///
/// The wrapped codec remains responsible for domain-specific encode failures.
/// This type adds adapter-level failures that cannot be represented by the
/// wrapped codec itself, such as a buffered encoder receiving an invalid input
/// or output start index.
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

    /// The caller supplied an output index outside the output slice.
    #[error("invalid output index {index} for output length {len}")]
    InvalidOutputIndex {
        /// Invalid output index supplied by the caller.
        index: usize,
        /// Length of the output slice.
        len: usize,
    },

    /// The output slice cannot hold all finish or reset output in one call.
    #[error(
        "insufficient finish output at index {output_index}: required {required} units, available {available}"
    )]
    InsufficientOutput {
        /// Absolute output index where finalization would start writing.
        output_index: usize,
        /// Output units required to finish in one call.
        required: usize,
        /// Output units available from `output_index`.
        available: usize,
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
    #[inline(always)]
    pub const fn encode(source: E, input_index: usize) -> Self {
        Self::Encode {
            source,
            input_index,
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
    ///
    /// # Parameters
    ///
    /// - `output_index`: Output index supplied by the caller.
    /// - `required`: Output units required to finish in one call.
    /// - `available`: Output units available from `output_index`.
    ///
    /// # Returns
    ///
    /// Returns an insufficient-output error.
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
}

impl<E> TranscodeError for CodecEncodeError<E> {
    #[inline(always)]
    fn invalid_input_index(_context: (), index: usize, len: usize) -> Self {
        Self::invalid_input_index(index, len)
    }

    #[inline(always)]
    fn invalid_output_index(_context: (), index: usize, len: usize) -> Self {
        Self::invalid_output_index(index, len)
    }

    #[inline(always)]
    fn insufficient_output(
        _context: (),
        output_index: usize,
        required: usize,
        available: usize,
    ) -> Self {
        Self::insufficient_output(output_index, required, available)
    }
}
