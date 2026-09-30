// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors reported by transcode encoders.

use thiserror::Error;

use super::capacity_error::CapacityError;
use super::transcode_domain_error::TranscodeDomainError;
use super::transcode_failure::TranscodeFailure;
use crate::Codec;

/// Encode transcode error for a codec-backed encoder.
///
/// # Type Parameters
///
/// - `C`: Codec supplying the domain error and input-value types.
pub type TranscodeEncodeErrorOf<C> = TranscodeEncodeError<<C as Codec>::EncodeError, <C as Codec>::Value>;

/// Error reported by an encode-oriented transcode operation.
///
/// # Type Parameters
///
/// - `E`: Domain error preserved from the codec or policy.
/// - `V`: Optional owned context for an unencodable input value.
///
/// # Examples
///
/// ```
/// use qubit_codec::TranscodeEncodeError;
///
/// let error = TranscodeEncodeError::<(), u8>::unencodable(3, 255);
/// assert_eq!(error.unencodable_ref(), Some((3, Some(&255))));
/// ```
#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
#[must_use]
pub enum TranscodeEncodeError<E, V> {
    /// Framework-level transcode failure.
    #[error(transparent)]
    Failure(
        /// Framework contract failure preserved as the error source.
        #[from]
        TranscodeFailure,
    ),

    /// The input value cannot be encoded by the target codec and policy.
    #[error("unencodable value at input index {input_index}")]
    Unencodable {
        /// Absolute input index of the value being encoded.
        input_index: usize,
        /// `Some` owns the rejected value; `None` means value context was
        /// unavailable.
        value: Option<V>,
    },

    /// Domain-specific codec, charset, or policy error.
    #[error(transparent)]
    Domain(
        /// Owned domain source and its reset, main, or finish context.
        #[from]
        TranscodeDomainError<E>,
    ),
}

impl<E, V> TranscodeEncodeError<E, V> {
    /// Creates a reset-phase domain-specific transcode error.
    ///
    /// # Parameters
    ///
    /// - `source`: Owned domain error from reset.
    ///
    /// # Returns
    ///
    /// Returns a domain error carrying reset-phase context.
    #[inline]
    pub const fn domain_reset(source: E) -> Self {
        Self::Domain(TranscodeDomainError::reset(source))
    }

    /// Creates a main-phase domain-specific transcode error.
    ///
    /// # Parameters
    ///
    /// - `source`: Owned domain error during encoding.
    /// - `input_index`: Absolute index of the failed input value.
    ///
    /// # Returns
    ///
    /// Returns a domain error carrying the input index and main-phase context.
    #[inline]
    pub const fn domain_main(source: E, input_index: usize) -> Self {
        Self::Domain(TranscodeDomainError::main(source, input_index))
    }

    /// Creates a finish-phase domain-specific transcode error.
    ///
    /// # Parameters
    ///
    /// - `source`: Owned domain error from finalization.
    ///
    /// # Returns
    ///
    /// Returns a domain error carrying finish-phase context.
    #[inline]
    pub const fn domain_finish(source: E) -> Self {
        Self::Domain(TranscodeDomainError::finish(source))
    }

    /// Creates an unencodable-value error with value context.
    ///
    /// # Parameters
    ///
    /// - `input_index`: Absolute index of the rejected value.
    /// - `value`: Rejected value transferred into the error.
    ///
    /// # Returns
    ///
    /// Returns an unencodable error with owned value context.
    #[inline]
    pub fn unencodable(input_index: usize, value: V) -> Self {
        Self::Unencodable {
            input_index,
            value: Some(value),
        }
    }

    /// Creates an unencodable-value error without value context.
    ///
    /// # Parameters
    ///
    /// - `input_index`: Absolute index of the rejected input.
    ///
    /// # Returns
    ///
    /// Returns an unencodable error with no retained value.
    #[inline]
    pub const fn unencodable_without_context(input_index: usize) -> Self {
        Self::Unencodable {
            input_index,
            value: None,
        }
    }

    /// Returns whether this error wraps a domain error.
    ///
    /// # Returns
    ///
    /// Returns `true` only for the Domain variant.
    #[inline]
    #[must_use]
    pub const fn is_domain(&self) -> bool {
        matches!(self, Self::Domain(_))
    }

    /// Returns the framework failure carried by this error.
    ///
    /// # Returns
    ///
    /// Returns `Some` borrowed framework failure for Failure, otherwise `None`.
    #[inline]
    #[must_use]
    pub const fn failure_ref(&self) -> Option<&TranscodeFailure> {
        match self {
            Self::Failure(failure) => Some(failure),
            Self::Unencodable { .. } | Self::Domain(_) => None,
        }
    }

    /// Borrows the wrapped domain error and transcode context.
    ///
    /// # Returns
    ///
    /// Returns `Some` borrowed domain error and phase context for Domain,
    /// or `None` for framework and unencodable errors.
    #[inline]
    #[must_use]
    pub const fn domain_error_ref(&self) -> Option<&TranscodeDomainError<E>> {
        match self {
            Self::Domain(error) => Some(error),
            Self::Failure(_) | Self::Unencodable { .. } => None,
        }
    }

    /// Borrows the wrapped domain error.
    ///
    /// # Returns
    ///
    /// Returns `Some` borrowed domain source for Domain, otherwise `None`.
    #[inline]
    #[must_use]
    pub const fn domain_ref(&self) -> Option<&E> {
        match self {
            Self::Domain(error) => Some(error.source()),
            Self::Failure(_) | Self::Unencodable { .. } => None,
        }
    }

    /// Borrows the unencodable value context carried by this error.
    ///
    /// # Returns
    ///
    /// Returns `Some((index, value))` for Unencodable, otherwise `None`.
    /// The nested option is `Some` when a retained value can be borrowed,
    /// and `None` when the error has only an input index.
    #[inline]
    #[must_use]
    pub const fn unencodable_ref(&self) -> Option<(usize, Option<&V>)> {
        match self {
            Self::Unencodable { input_index, value } => Some((*input_index, value.as_ref())),
            Self::Failure(_) | Self::Domain(_) => None,
        }
    }

    /// Maps the wrapped domain error while preserving other errors.
    ///
    /// # Type Parameters
    ///
    /// - `F`: Callback consuming the domain source.
    /// - `T`: Replacement domain error type.
    ///
    /// # Parameters
    ///
    /// - `f`: Called exactly once for Domain, never for other variants.
    ///
    /// # Returns
    ///
    /// Returns an error with the transformed domain source while preserving
    /// phase, input position, framework failures, and unencodable value
    /// context.
    #[inline]
    pub fn map_domain<F, T>(self, f: F) -> TranscodeEncodeError<T, V>
    where
        F: FnOnce(E) -> T,
    {
        match self {
            Self::Failure(failure) => TranscodeEncodeError::Failure(failure),
            Self::Unencodable { input_index, value } => TranscodeEncodeError::Unencodable { input_index, value },
            Self::Domain(error) => TranscodeEncodeError::Domain(error.map_source(f)),
        }
    }

    /// Maps value context carried by unencodable-value errors.
    ///
    /// # Type Parameters
    ///
    /// - `F`: Callback consuming a retained value.
    /// - `W`: Replacement value-context type.
    ///
    /// # Parameters
    ///
    /// - `f`: Called once for a retained unencodable value, otherwise not
    ///   called.
    ///
    /// # Returns
    ///
    /// Returns an error with transformed value context; absent context and
    /// other variants are preserved.
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
}

impl<E, V> From<CapacityError> for TranscodeEncodeError<E, V> {
    /// Converts capacity planning errors into transcode framework errors.
    ///
    /// # Parameters
    ///
    /// - `error`: Capacity failure from output planning.
    ///
    /// # Returns
    ///
    /// Returns the corresponding framework failure variant.
    #[inline]
    fn from(error: CapacityError) -> Self {
        TranscodeFailure::from(error).into()
    }
}
