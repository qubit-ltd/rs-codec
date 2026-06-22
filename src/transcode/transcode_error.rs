// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors reported by transcode engines and transcoder adapters.

use thiserror::Error;

use super::buffer_contract_error::BufferContractError;

/// Error reported by a transcode operation.
///
/// Buffer contract failures are framework errors owned by the transcode layer.
/// Codec-, charset-, or policy-specific failures are carried as domain errors.
///
/// # Type Parameters
///
/// - `E`: Domain error reported by the concrete transcoder.
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum TranscodeError<E> {
    /// The caller-provided input or output buffer contract was violated.
    #[error(transparent)]
    Buffer(#[from] BufferContractError),

    /// Domain-specific codec, charset, or policy error.
    #[error("{0}")]
    Domain(#[source] E),
}

impl<E> TranscodeError<E> {
    /// Creates a domain-specific transcode error.
    ///
    /// # Parameters
    ///
    /// - `error`: Domain error reported by the concrete transcoder.
    ///
    /// # Returns
    ///
    /// Returns a transcode error wrapping `error`.
    #[inline(always)]
    pub const fn domain(error: E) -> Self {
        Self::Domain(error)
    }

    /// Creates an invalid-input-index error.
    #[must_use]
    #[inline(always)]
    pub const fn invalid_input_index(index: usize, len: usize) -> Self {
        Self::Buffer(BufferContractError::invalid_input_index(index, len))
    }

    /// Creates an invalid-output-index error.
    #[must_use]
    #[inline(always)]
    pub const fn invalid_output_index(index: usize, len: usize) -> Self {
        Self::Buffer(BufferContractError::invalid_output_index(index, len))
    }

    /// Creates an insufficient-output error.
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
    #[must_use]
    #[inline(always)]
    pub const fn output_length_overflow() -> Self {
        Self::Buffer(BufferContractError::output_length_overflow())
    }

    /// Returns whether this error wraps a domain error.
    ///
    /// # Returns
    ///
    /// Returns `true` for [`TranscodeError::Domain`].
    #[must_use]
    #[inline(always)]
    pub const fn is_domain(&self) -> bool {
        matches!(self, Self::Domain(_))
    }

    /// Borrows the wrapped domain error.
    ///
    /// # Returns
    ///
    /// Returns `Some(error)` for [`TranscodeError::Domain`] and `None` for
    /// buffer contract errors.
    #[must_use]
    #[inline(always)]
    pub const fn domain_ref(&self) -> Option<&E> {
        match self {
            Self::Domain(error) => Some(error),
            Self::Buffer(_) => None,
        }
    }

    /// Maps the wrapped domain error while preserving buffer contract errors.
    ///
    /// # Type Parameters
    ///
    /// - `F`: Mapping function type.
    /// - `T`: Target domain error type.
    ///
    /// # Parameters
    ///
    /// - `f`: Function applied to the wrapped domain error.
    ///
    /// # Returns
    ///
    /// Returns the mapped transcode error.
    #[inline]
    pub fn map_domain<F, T>(self, f: F) -> TranscodeError<T>
    where
        F: FnOnce(E) -> T,
    {
        match self {
            Self::Buffer(error) => TranscodeError::Buffer(error),
            Self::Domain(error) => TranscodeError::Domain(f(error)),
        }
    }

    /// Validates that `input_index` is within an input slice.
    #[inline]
    pub fn ensure_input_index(
        input_len: usize,
        input_index: usize,
    ) -> Result<(), Self> {
        BufferContractError::ensure_input_index(input_len, input_index)
            .map_err(Self::Buffer)
    }

    /// Validates that `output_index` is within an output slice.
    #[inline]
    pub fn ensure_output_index(
        output_len: usize,
        output_index: usize,
    ) -> Result<(), Self> {
        BufferContractError::ensure_output_index(output_len, output_index)
            .map_err(Self::Buffer)
    }

    /// Validates input and output start indices for a transcode call.
    #[inline]
    pub fn ensure_transcode_indices(
        input_len: usize,
        input_index: usize,
        output_len: usize,
        output_index: usize,
    ) -> Result<(), Self> {
        BufferContractError::ensure_transcode_indices(
            input_len,
            input_index,
            output_len,
            output_index,
        )
        .map_err(Self::Buffer)
    }

    /// Validates that an output slice can hold one-shot finalization output.
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

    /// Validates an indexed output range and its minimum writable capacity.
    #[inline]
    pub fn ensure_output_range(
        output_len: usize,
        output_index: usize,
        range_len: usize,
        required: usize,
    ) -> Result<(), Self> {
        BufferContractError::ensure_output_range(
            output_len,
            output_index,
            range_len,
            required,
        )
        .map_err(Self::Buffer)
    }
}
