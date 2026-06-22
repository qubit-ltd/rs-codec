// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Generic encode error used by codec-backed encoder adapters.

use thiserror::Error;

use crate::BufferContractError;

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

    /// The wrapped codec cannot represent the input value.
    #[error("unencodable value at input index {input_index}")]
    UnencodableValue {
        /// Absolute input index of the value being encoded.
        input_index: usize,
    },

    /// The caller-provided input or output buffer contract was violated.
    #[error(transparent)]
    Buffer(#[from] BufferContractError),
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

    /// Creates an unencodable-value error.
    ///
    /// # Parameters
    ///
    /// - `input_index`: Absolute input index of the value being encoded.
    ///
    /// # Returns
    ///
    /// Returns an unencodable-value error.
    #[must_use]
    #[inline(always)]
    pub const fn unencodable_value(input_index: usize) -> Self {
        Self::UnencodableValue { input_index }
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
        Self::Buffer(BufferContractError::invalid_input_index(index, len))
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
        Self::Buffer(BufferContractError::invalid_output_index(index, len))
    }

    /// Creates an insufficient-output error.
    ///
    /// # Parameters
    ///
    /// - `output_index`: Output index supplied by the caller.
    /// - `required`: Output units required from `output_index`.
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
        Self::Buffer(BufferContractError::insufficient_output(
            output_index,
            required,
            available,
        ))
    }

    /// Creates an output-length-overflow error.
    ///
    /// # Returns
    ///
    /// Returns an output-length-overflow error.
    #[must_use]
    #[inline(always)]
    pub const fn output_length_overflow() -> Self {
        Self::Buffer(BufferContractError::output_length_overflow())
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
    pub fn ensure_input_index(
        input_len: usize,
        input_index: usize,
    ) -> Result<(), Self> {
        BufferContractError::ensure_input_index(input_len, input_index)
            .map_err(Self::Buffer)
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
    pub fn ensure_output_index(
        output_len: usize,
        output_index: usize,
    ) -> Result<(), Self> {
        BufferContractError::ensure_output_index(output_len, output_index)
            .map_err(Self::Buffer)
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
        BufferContractError::ensure_output_capacity(
            output_len,
            output_index,
            required,
        )
        .map_err(Self::Buffer)
    }
}
