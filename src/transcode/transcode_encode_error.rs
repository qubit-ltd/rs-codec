// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors reported by transcode encoders.

use thiserror::Error;

use super::{
    capacity_error::CapacityError, transcode_domain_error::TranscodeDomainError,
    transcode_failure::TranscodeFailure,
};
use crate::Codec;

/// Encode transcode error for a codec-backed encoder.
pub type TranscodeEncodeErrorOf<C> =
    TranscodeEncodeError<<C as Codec>::EncodeError, <C as Codec>::Value>;

/// Error reported by an encode-oriented transcode operation.
#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
pub enum TranscodeEncodeError<E, V> {
    /// Framework-level transcode failure.
    #[error(transparent)]
    Failure(#[from] TranscodeFailure),

    /// The input value cannot be encoded by the target codec and policy.
    #[error("unencodable value at input index {input_index}")]
    Unencodable {
        /// Absolute input index of the value being encoded.
        input_index: usize,
        /// Value being encoded, when the transcoder can expose it.
        value: Option<V>,
    },

    /// Domain-specific codec, charset, or policy error.
    #[error(transparent)]
    Domain(#[from] TranscodeDomainError<E>),
}

impl<E, V> TranscodeEncodeError<E, V> {
    /// Creates an invalid-input-index framework error.
    #[inline(always)]
    pub const fn invalid_input_index(index: usize, input_len: usize) -> Self {
        Self::Failure(TranscodeFailure::invalid_input_index(index, input_len))
    }

    /// Creates an invalid-output-index framework error.
    #[inline(always)]
    pub const fn invalid_output_index(index: usize, output_len: usize) -> Self {
        Self::Failure(TranscodeFailure::invalid_output_index(index, output_len))
    }

    /// Creates an insufficient-output framework error.
    #[inline(always)]
    pub const fn insufficient_output(
        output_index: usize,
        required: usize,
        available: usize,
    ) -> Self {
        Self::Failure(TranscodeFailure::insufficient_output(
            output_index,
            required,
            available,
        ))
    }

    /// Creates an output-length-overflow framework error.
    #[inline(always)]
    pub const fn output_length_overflow() -> Self {
        Self::Failure(TranscodeFailure::output_length_overflow())
    }

    /// Creates an incomplete-input framework error.
    #[inline(always)]
    pub const fn incomplete_input(input_index: usize, required: usize, available: usize) -> Self {
        Self::Failure(TranscodeFailure::incomplete_input(
            input_index,
            required,
            available,
        ))
    }

    /// Creates a trailing-input framework error.
    #[inline(always)]
    pub const fn trailing_input(consumed: usize, remaining: usize) -> Self {
        Self::Failure(TranscodeFailure::trailing_input(consumed, remaining))
    }

    /// Creates a reset-phase domain-specific transcode error.
    #[inline(always)]
    pub const fn domain_reset(source: E) -> Self {
        Self::Domain(TranscodeDomainError::reset(source))
    }

    /// Creates a main-phase domain-specific transcode error.
    #[inline(always)]
    pub const fn domain_main(source: E, input_index: usize) -> Self {
        Self::Domain(TranscodeDomainError::main(source, input_index))
    }

    /// Creates a finish-phase domain-specific transcode error.
    #[inline(always)]
    pub const fn domain_finish(source: E) -> Self {
        Self::Domain(TranscodeDomainError::finish(source))
    }

    /// Creates an unencodable-value error with value context.
    #[inline(always)]
    #[must_use]
    pub fn unencodable(input_index: usize, value: V) -> Self {
        Self::Unencodable {
            input_index,
            value: Some(value),
        }
    }

    /// Creates an unencodable-value error without value context.
    #[inline(always)]
    #[must_use]
    pub const fn unencodable_without_context(input_index: usize) -> Self {
        Self::Unencodable {
            input_index,
            value: None,
        }
    }

    /// Returns whether this error wraps a domain error.
    #[inline(always)]
    #[must_use]
    pub const fn is_domain(&self) -> bool {
        matches!(self, Self::Domain(_))
    }

    /// Returns the framework failure carried by this error.
    #[inline(always)]
    #[must_use]
    pub const fn failure_ref(&self) -> Option<&TranscodeFailure> {
        match self {
            Self::Failure(failure) => Some(failure),
            Self::Unencodable { .. } | Self::Domain(_) => None,
        }
    }

    /// Borrows the wrapped domain error and transcode context.
    #[inline(always)]
    #[must_use]
    pub const fn domain_error_ref(&self) -> Option<&TranscodeDomainError<E>> {
        match self {
            Self::Domain(error) => Some(error),
            Self::Failure(_) | Self::Unencodable { .. } => None,
        }
    }

    /// Borrows the wrapped domain error.
    #[must_use]
    pub const fn domain_ref(&self) -> Option<&E> {
        match self {
            Self::Domain(error) => Some(error.source()),
            Self::Failure(_) | Self::Unencodable { .. } => None,
        }
    }

    /// Borrows the unencodable value context carried by this error.
    #[inline(always)]
    #[must_use]
    pub const fn unencodable_ref(&self) -> Option<(usize, Option<&V>)> {
        match self {
            Self::Unencodable { input_index, value } => Some((*input_index, value.as_ref())),
            Self::Failure(_) | Self::Domain(_) => None,
        }
    }

    /// Maps the wrapped domain error while preserving other errors.
    #[inline]
    pub fn map_domain<F, T>(self, f: F) -> TranscodeEncodeError<T, V>
    where
        F: FnOnce(E) -> T,
    {
        match self {
            Self::Failure(failure) => TranscodeEncodeError::Failure(failure),
            Self::Unencodable { input_index, value } => {
                TranscodeEncodeError::Unencodable { input_index, value }
            }
            Self::Domain(error) => TranscodeEncodeError::Domain(error.map_source(f)),
        }
    }

    /// Maps value context carried by unencodable-value errors.
    #[inline]
    pub fn map_value<F, W>(self, f: F) -> TranscodeEncodeError<E, W>
    where
        F: FnOnce(V) -> W,
    {
        match self {
            Self::Failure(failure) => TranscodeEncodeError::Failure(failure),
            Self::Unencodable { input_index, value } => TranscodeEncodeError::Unencodable {
                input_index,
                value: value.map(f),
            },
            Self::Domain(error) => TranscodeEncodeError::Domain(error),
        }
    }

    /// Ensures the output index is valid.
    #[inline]
    pub fn ensure_output_index(output_len: usize, output_index: usize) -> Result<(), Self> {
        TranscodeFailure::ensure_output_index(output_len, output_index).map_err(Self::from)
    }

    /// Ensures input and output indices are valid.
    #[inline]
    pub fn ensure_transcode_indices(
        input_len: usize,
        input_index: usize,
        output_len: usize,
        output_index: usize,
    ) -> Result<(), Self> {
        TranscodeFailure::ensure_transcode_indices(input_len, input_index, output_len, output_index)
            .map_err(Self::from)
    }

    /// Ensures output capacity is sufficient.
    #[inline]
    pub fn ensure_output_capacity(
        output_len: usize,
        output_index: usize,
        required: usize,
    ) -> Result<(), Self> {
        TranscodeFailure::ensure_output_capacity(output_len, output_index, required)
            .map_err(Self::from)
    }
}

impl<E, V> From<CapacityError> for TranscodeEncodeError<E, V> {
    /// Converts capacity planning errors into transcode framework errors.
    #[inline(always)]
    fn from(error: CapacityError) -> Self {
        TranscodeFailure::from(error).into()
    }
}
