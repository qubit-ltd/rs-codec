// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors reported by transcode decoders.

use core::num::NonZeroUsize;

use thiserror::Error;

use super::{
    capacity_error::CapacityError,
    transcode_domain_error::TranscodeDomainError,
    transcode_failure::TranscodeFailure,
};
use crate::{
    Codec,
    DecodeFailure,
};

/// Decode transcode error for a codec-backed decoder.
pub type TranscodeDecodeErrorOf<C> =
    TranscodeDecodeError<<C as Codec>::DecodeError>;

/// Error reported by a decode-oriented transcode operation.
#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
pub enum TranscodeDecodeError<E> {
    /// Framework-level transcode failure.
    #[error(transparent)]
    Failure(#[from] TranscodeFailure),

    /// Domain-specific codec, charset, or policy error.
    #[error(transparent)]
    Domain(#[from] TranscodeDomainError<E>),
}

impl<E> TranscodeDecodeError<E> {
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
    pub const fn incomplete_input(
        input_index: usize,
        required: usize,
        available: usize,
    ) -> Self {
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

    /// Creates a main-phase domain-specific transcode error with decode
    /// consumption context.
    #[inline(always)]
    pub const fn domain_main_with_consumed(
        source: E,
        input_index: usize,
        input_consumed: Option<NonZeroUsize>,
    ) -> Self {
        Self::Domain(TranscodeDomainError::main_with_consumed(
            source,
            input_index,
            input_consumed,
        ))
    }

    /// Creates a finish-phase domain-specific transcode error.
    #[inline(always)]
    pub const fn domain_finish(source: E) -> Self {
        Self::Domain(TranscodeDomainError::finish(source))
    }

    /// Converts a low-level decode failure into a decode transcode error.
    #[inline]
    #[must_use]
    pub fn from_decode_failure(
        failure: DecodeFailure<E>,
        input_index: usize,
        available: usize,
    ) -> Self {
        match failure {
            DecodeFailure::Incomplete { required_total } => {
                TranscodeFailure::incomplete_input(
                    input_index,
                    required_total.get(),
                    available,
                )
                .into()
            }
            DecodeFailure::Invalid { source, consumed } => {
                Self::domain_main_with_consumed(source, input_index, consumed)
            }
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
            Self::Domain(_) => None,
        }
    }

    /// Borrows the wrapped domain error and transcode context.
    #[inline(always)]
    #[must_use]
    pub const fn domain_error_ref(&self) -> Option<&TranscodeDomainError<E>> {
        match self {
            Self::Domain(error) => Some(error),
            Self::Failure(_) => None,
        }
    }

    /// Borrows the wrapped domain error.
    #[must_use]
    pub const fn domain_ref(&self) -> Option<&E> {
        match self {
            Self::Domain(error) => Some(error.source()),
            Self::Failure(_) => None,
        }
    }

    /// Maps the wrapped domain error while preserving framework failures.
    #[inline]
    pub fn map_domain<F, T>(self, f: F) -> TranscodeDecodeError<T>
    where
        F: FnOnce(E) -> T,
    {
        match self {
            Self::Failure(failure) => TranscodeDecodeError::Failure(failure),
            Self::Domain(error) => {
                TranscodeDecodeError::Domain(error.map_source(f))
            }
        }
    }

    /// Ensures the input index is valid.
    #[inline]
    pub fn ensure_input_index(
        input_len: usize,
        input_index: usize,
    ) -> Result<(), Self> {
        TranscodeFailure::ensure_input_index(input_len, input_index)
            .map_err(Self::from)
    }

    /// Ensures at least `required` input units are readable.
    #[inline]
    pub fn ensure_min_input(
        input_len: usize,
        input_index: usize,
        required: usize,
    ) -> Result<(), Self> {
        TranscodeFailure::ensure_min_input(input_len, input_index, required)
            .map_err(Self::from)
    }

    /// Ensures there is no trailing input.
    #[inline]
    pub fn ensure_no_trailing_input(
        consumed: usize,
        input_len: usize,
    ) -> Result<(), Self> {
        TranscodeFailure::ensure_no_trailing_input(consumed, input_len)
            .map_err(Self::from)
    }

    /// Ensures the output index is valid.
    #[inline]
    pub fn ensure_output_index(
        output_len: usize,
        output_index: usize,
    ) -> Result<(), Self> {
        TranscodeFailure::ensure_output_index(output_len, output_index)
            .map_err(Self::from)
    }

    /// Ensures input and output indices are valid.
    #[inline]
    pub fn ensure_transcode_indices(
        input_len: usize,
        input_index: usize,
        output_len: usize,
        output_index: usize,
    ) -> Result<(), Self> {
        TranscodeFailure::ensure_transcode_indices(
            input_len,
            input_index,
            output_len,
            output_index,
        )
        .map_err(Self::from)
    }

    /// Ensures output capacity is sufficient.
    #[inline]
    pub fn ensure_output_capacity(
        output_len: usize,
        output_index: usize,
        required: usize,
    ) -> Result<(), Self> {
        TranscodeFailure::ensure_output_capacity(
            output_len,
            output_index,
            required,
        )
        .map_err(Self::from)
    }

    /// Ensures output range capacity is sufficient.
    #[inline]
    pub fn ensure_output_range(
        output_len: usize,
        output_index: usize,
        available: usize,
        required: usize,
    ) -> Result<(), Self> {
        TranscodeFailure::ensure_output_range(
            output_len,
            output_index,
            available,
            required,
        )
        .map_err(Self::from)
    }
}

impl<E> From<CapacityError> for TranscodeDecodeError<E> {
    /// Converts capacity planning errors into transcode framework errors.
    #[inline(always)]
    fn from(error: CapacityError) -> Self {
        TranscodeFailure::from(error).into()
    }
}
