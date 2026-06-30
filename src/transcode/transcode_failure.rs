// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Framework-level failures reported by safe transcode APIs.

use thiserror::Error;

/// Framework-level failure reported by a transcode operation.
///
/// These failures are part of the safe public transcode API. They describe
/// caller-supplied buffer ranges, capacity planning, complete-input shape, or
/// unhandled value-domain boundaries that the transcode layer can detect
/// without interpreting a concrete codec error.
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
#[non_exhaustive]
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

    /// The codec could not encode a value and no hook policy handled it.
    #[error("unencodable value at input index {input_index}")]
    UnencodableValue {
        /// Absolute input index of the value being encoded.
        input_index: usize,
    },
}
