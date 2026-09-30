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

use super::capacity_error::CapacityError;
use super::transcode_domain_error::TranscodeDomainError;
use super::transcode_failure::TranscodeFailure;
use crate::Codec;
use crate::DecodeFailure;

/// Decode transcode error for a codec-backed decoder.
///
/// # Type Parameters
///
/// - `C`: Codec whose decode error becomes the domain failure.
pub type TranscodeDecodeErrorOf<C> = TranscodeDecodeError<<C as Codec>::DecodeError>;

/// Error reported by a decode-oriented transcode operation.
///
/// # Type Parameters
///
/// - `E`: Owned codec or policy failure preserved with transcode context.
///
/// # Examples
///
/// ```
/// use qubit_codec::TranscodeDecodeError;
///
/// let error = TranscodeDecodeError::domain_main("invalid byte", 3);
/// assert!(error.is_domain());
/// assert_eq!(error.domain_ref(), Some(&"invalid byte"));
/// assert!(error.failure_ref().is_none());
/// ```
#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
#[must_use]
pub enum TranscodeDecodeError<E> {
    /// Framework-level transcode failure.
    #[error(transparent)]
    Failure(
        /// Framework contract, capacity, or incomplete-input failure.
        #[from]
        TranscodeFailure,
    ),

    /// Domain-specific codec, charset, or policy error.
    #[error(transparent)]
    Domain(
        /// Codec failure retaining its phase and source position.
        #[from]
        TranscodeDomainError<E>,
    ),
}

impl<E> TranscodeDecodeError<E> {
    /// Creates a reset-phase domain-specific transcode error.
    ///
    /// # Parameters
    ///
    /// - `source`: Owned codec or policy failure.
    ///
    /// # Returns
    ///
    /// The domain failure tagged as reset.
    #[inline]
    pub const fn domain_reset(source: E) -> Self {
        Self::Domain(TranscodeDomainError::reset(source))
    }

    /// Creates a main-phase domain-specific transcode error.
    ///
    /// # Parameters
    ///
    /// - `source`: Owned codec or policy failure.
    /// - `input_index`: Absolute position of the failing input.
    ///
    /// # Returns
    ///
    /// The domain failure tagged with its main-phase input position.
    #[inline]
    pub const fn domain_main(source: E, input_index: usize) -> Self {
        Self::Domain(TranscodeDomainError::main(source, input_index))
    }

    /// Creates a main-phase domain-specific transcode error with decode
    /// consumption context.
    ///
    /// # Parameters
    ///
    /// - `source`: Owned codec or policy failure.
    /// - `input_index`: Absolute position of the failing input.
    /// - `input_consumed`: Some(non-zero span) when known, or None for an
    ///   unknown span.
    ///
    /// # Returns
    ///
    /// The main-phase failure retaining the known or unknown consumed span.
    #[inline]
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
    ///
    /// # Parameters
    ///
    /// - `source`: Owned codec or policy failure.
    ///
    /// # Returns
    ///
    /// The domain failure tagged as finish.
    #[inline]
    pub const fn domain_finish(source: E) -> Self {
        Self::Domain(TranscodeDomainError::finish(source))
    }

    /// Converts a low-level decode failure into a decode transcode error.
    ///
    /// # Parameters
    ///
    /// - `failure`: Owned incomplete or invalid low-level failure.
    /// - `input_index`: Absolute failing input position.
    /// - `available`: Number of readable units at that position.
    ///
    /// # Returns
    ///
    /// A domain error when a codec source exists; otherwise an incomplete-input
    /// framework failure.
    #[inline]
    pub fn from_decode_failure(failure: DecodeFailure<E>, input_index: usize, available: usize) -> Self {
        match failure {
            DecodeFailure::Incomplete {
                source: Some(source), ..
            } => Self::domain_main(source, input_index),
            DecodeFailure::Incomplete { required_total, .. } => {
                TranscodeFailure::incomplete_input(input_index, required_total.get(), available).into()
            }
            DecodeFailure::Invalid { source, consumed } => {
                Self::domain_main_with_consumed(source, input_index, consumed)
            }
        }
    }

    /// Returns whether this error wraps a domain error.
    ///
    /// # Returns
    ///
    /// True for Domain and false for a framework Failure.
    #[inline]
    #[must_use]
    pub const fn is_domain(&self) -> bool {
        matches!(self, Self::Domain(_))
    }

    /// Returns the framework failure carried by this error.
    ///
    /// # Returns
    ///
    /// Some(borrowed failure) for Failure, or None for Domain.
    #[inline]
    #[must_use]
    pub const fn failure_ref(&self) -> Option<&TranscodeFailure> {
        match self {
            Self::Failure(failure) => Some(failure),
            Self::Domain(_) => None,
        }
    }

    /// Borrows the wrapped domain error and transcode context.
    ///
    /// # Returns
    ///
    /// Some(borrowed context) for Domain, or None for Failure.
    #[inline]
    #[must_use]
    pub const fn domain_error_ref(&self) -> Option<&TranscodeDomainError<E>> {
        match self {
            Self::Domain(error) => Some(error),
            Self::Failure(_) => None,
        }
    }

    /// Borrows the wrapped domain error.
    ///
    /// # Returns
    ///
    /// Some(borrowed source) for Domain, or None for Failure.
    #[inline]
    #[must_use]
    pub const fn domain_ref(&self) -> Option<&E> {
        match self {
            Self::Domain(error) => Some(error.source()),
            Self::Failure(_) => None,
        }
    }

    /// Maps the wrapped domain error while preserving framework failures.
    ///
    /// # Type Parameters
    ///
    /// - `F`: One-shot mapping closure, invoked only for a domain failure.
    /// - `T`: Replacement domain error type.
    ///
    /// # Parameters
    ///
    /// - `f`: Consumes the domain source, if present; may perform
    ///   caller-defined side effects. Framework failures do not invoke it.
    ///
    /// # Returns
    ///
    /// The mapped error retaining phase and input context, or the original
    /// framework failure unchanged.
    #[inline]
    pub fn map_domain<F, T>(self, f: F) -> TranscodeDecodeError<T>
    where
        F: FnOnce(E) -> T,
    {
        match self {
            Self::Failure(failure) => TranscodeDecodeError::Failure(failure),
            Self::Domain(error) => TranscodeDecodeError::Domain(error.map_source(f)),
        }
    }
}

impl<E> From<CapacityError> for TranscodeDecodeError<E> {
    /// Converts capacity planning errors into transcode framework errors.
    ///
    /// # Parameters
    ///
    /// - `error`: Owned capacity planning failure.
    ///
    /// # Returns
    ///
    /// The corresponding framework failure wrapped in this decode error.
    #[inline]
    fn from(error: CapacityError) -> Self {
        TranscodeFailure::from(error).into()
    }
}
