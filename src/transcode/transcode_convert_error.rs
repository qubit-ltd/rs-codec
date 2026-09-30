// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors reported by transcode converters.

use thiserror::Error;

use super::capacity_error::CapacityError;
use super::transcode_decode_error::TranscodeDecodeError;
use super::transcode_domain_error::TranscodeDomainError;
use super::transcode_encode_error::TranscodeEncodeError;
use super::transcode_failure::TranscodeFailure;
use crate::Codec;

/// Convert transcode error for a codec-backed converter.
///
/// # Type Parameters
///
/// - `D`: Source codec providing decode errors and the intermediate value type.
/// - `E`: Target codec providing encode errors for the same value type.
pub type TranscodeConvertErrorOf<D, E> =
    TranscodeConvertError<<D as Codec>::DecodeError, <E as Codec>::EncodeError, <D as Codec>::Value>;

/// Error reported by a unit-to-unit transcode conversion.
///
/// # Type Parameters
///
/// - `DE`: Source decoder domain error.
/// - `EE`: Target encoder domain error.
/// - `V`: Owned intermediate value retained when encoding is rejected.
///
/// # Examples
///
/// ```
/// use qubit_codec::TranscodeConvertError;
///
/// let error = TranscodeConvertError::<&str, &str, u8>::unencodable(3, 42);
/// assert_eq!(error.unencodable_ref(), Some((3, Some(&42))));
/// assert_eq!(error.failure_ref(), None);
/// ```
#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
#[must_use]
pub enum TranscodeConvertError<DE, EE, V> {
    /// Framework-level transcode failure.
    #[error(transparent)]
    Failure(
        /// Framework contract, capacity, or lifecycle failure.
        #[from]
        TranscodeFailure,
    ),

    /// Source-side domain error.
    #[error("decode side failed: {0}")]
    DecodeDomain(
        /// Owned source error with its lifecycle phase and input context.
        #[source]
        TranscodeDomainError<DE>,
    ),

    /// Target-side domain error.
    #[error("encode side failed: {0}")]
    EncodeDomain(
        /// Owned target error with its lifecycle phase and input context.
        #[source]
        TranscodeDomainError<EE>,
    ),

    /// The decoded intermediate value cannot be encoded by the target codec and
    /// policy.
    #[error("unencodable value at input index {input_index}")]
    Unencodable {
        /// Absolute source input index of the value being encoded.
        input_index: usize,
        /// `Some` retains the decoded intermediate value; `None` means value
        /// context was unavailable or intentionally omitted.
        value: Option<V>,
    },
}

impl<DE, EE, V> TranscodeConvertError<DE, EE, V> {
    /// Creates a source reset-phase domain-specific converter error.
    ///
    /// # Parameters
    ///
    /// - `source`: Owned source error raised during reset.
    ///
    /// # Returns
    ///
    /// Returns a source-domain reset error.
    #[inline]
    pub const fn decode_domain_reset(source: DE) -> Self {
        Self::DecodeDomain(TranscodeDomainError::reset(source))
    }

    /// Creates a source main-phase domain-specific converter error.
    ///
    /// # Parameters
    ///
    /// - `source`: Owned source error raised during conversion.
    /// - `input_index`: Absolute source position of the failed operation.
    ///
    /// # Returns
    ///
    /// Returns a source-domain main-phase error at the supplied input index.
    #[inline]
    pub const fn decode_domain_main(source: DE, input_index: usize) -> Self {
        Self::DecodeDomain(TranscodeDomainError::main(source, input_index))
    }

    /// Creates a source main-phase converter error with decode consumption
    /// context.
    ///
    /// # Parameters
    ///
    /// - `source`: Owned source error raised during conversion.
    /// - `input_index`: Absolute source position of the failed operation.
    /// - `input_consumed`: `Some` records invalid units consumed; `None` means
    ///   unknown.
    ///
    /// # Returns
    ///
    /// Returns a source-domain main-phase error retaining consumption context.
    #[inline]
    pub const fn decode_domain_main_with_consumed(
        source: DE,
        input_index: usize,
        input_consumed: Option<core::num::NonZeroUsize>,
    ) -> Self {
        Self::DecodeDomain(TranscodeDomainError::main_with_consumed(
            source,
            input_index,
            input_consumed,
        ))
    }

    /// Creates a source finish-phase domain-specific converter error.
    ///
    /// # Parameters
    ///
    /// - `source`: Owned source error raised during finalization.
    ///
    /// # Returns
    ///
    /// Returns a source-domain finish error.
    #[inline]
    pub const fn decode_domain_finish(source: DE) -> Self {
        Self::DecodeDomain(TranscodeDomainError::finish(source))
    }

    /// Creates a target reset-phase domain-specific converter error.
    ///
    /// # Parameters
    ///
    /// - `source`: Owned target error raised during reset.
    ///
    /// # Returns
    ///
    /// Returns a target-domain reset error.
    #[inline]
    pub const fn encode_domain_reset(source: EE) -> Self {
        Self::EncodeDomain(TranscodeDomainError::reset(source))
    }

    /// Creates a target main-phase domain-specific converter error.
    ///
    /// # Parameters
    ///
    /// - `source`: Owned target error raised during conversion.
    /// - `input_index`: Absolute source position of the failed operation.
    ///
    /// # Returns
    ///
    /// Returns a target-domain main-phase error at the supplied input index.
    #[inline]
    pub const fn encode_domain_main(source: EE, input_index: usize) -> Self {
        Self::EncodeDomain(TranscodeDomainError::main(source, input_index))
    }

    /// Creates a target finish-phase domain-specific converter error.
    ///
    /// # Parameters
    ///
    /// - `source`: Owned target error raised during finalization.
    ///
    /// # Returns
    ///
    /// Returns a target-domain finish error.
    #[inline]
    pub const fn encode_domain_finish(source: EE) -> Self {
        Self::EncodeDomain(TranscodeDomainError::finish(source))
    }

    /// Creates an unencodable intermediate-value error with value context.
    ///
    /// # Parameters
    ///
    /// - `input_index`: Absolute source index of the rejected value.
    /// - `value`: Intermediate value moved into the error for recovery.
    ///
    /// # Returns
    ///
    /// Returns an unencodable error owning `Some(value)`.
    #[inline]
    pub fn unencodable(input_index: usize, value: V) -> Self {
        Self::Unencodable {
            input_index,
            value: Some(value),
        }
    }

    /// Creates an unencodable intermediate-value error without value context.
    ///
    /// # Parameters
    ///
    /// - `input_index`: Absolute source index of the rejected value.
    ///
    /// # Returns
    ///
    /// Returns an unencodable error with absent (`None`) value context.
    #[inline]
    pub const fn unencodable_without_context(input_index: usize) -> Self {
        Self::Unencodable {
            input_index,
            value: None,
        }
    }

    /// Converts an encode error into a converter error while adding fallback
    /// value context to unencodable errors that lack it.
    ///
    /// # Parameters
    ///
    /// - `error`: Owned target-side error to convert.
    /// - `fallback_value`: Value moved into absent context, or dropped if not
    ///   needed.
    ///
    /// # Returns
    ///
    /// Returns the converted error, adding the fallback only when unencodable
    /// value context is absent.
    #[inline]
    pub fn from_encode_error_with_value(error: TranscodeEncodeError<EE, V>, fallback_value: V) -> Self {
        match error {
            TranscodeEncodeError::Unencodable {
                input_index,
                value: None,
            } => Self::Unencodable {
                input_index,
                value: Some(fallback_value),
            },
            other => Self::from(other),
        }
    }

    /// Returns the framework failure carried by this error.
    ///
    /// # Returns
    ///
    /// Returns `Some` for a framework failure, or `None` for domain and
    /// unencodable errors.
    #[inline]
    #[must_use]
    pub const fn failure_ref(&self) -> Option<&TranscodeFailure> {
        match self {
            Self::Failure(failure) => Some(failure),
            Self::DecodeDomain(_) | Self::EncodeDomain(_) | Self::Unencodable { .. } => None,
        }
    }

    /// Borrows the unencodable value context carried by this error.
    ///
    /// # Returns
    ///
    /// Returns `Some((index, value))` for an unencodable error and `None`
    /// otherwise. The inner `Some` borrows retained value context; inner
    /// `None` means it is absent.
    #[inline]
    #[must_use]
    pub const fn unencodable_ref(&self) -> Option<(usize, Option<&V>)> {
        match self {
            Self::Unencodable { input_index, value } => Some((*input_index, value.as_ref())),
            Self::Failure(_) | Self::DecodeDomain(_) | Self::EncodeDomain(_) => None,
        }
    }

    /// Maps the source domain error while preserving other errors.
    ///
    /// # Type Parameters
    ///
    /// - `F`: One-shot transformation from `DE` to `T`.
    /// - `T`: Replacement payload type.
    ///
    /// # Parameters
    ///
    /// - `f`: Invoked only for a source-domain error; otherwise dropped without
    ///   being called.
    ///
    /// # Returns
    ///
    /// Returns an owned error preserving all other variants and positional
    /// context.
    ///
    /// # Panics
    ///
    /// Propagates a panic if the invoked transformation panics.
    #[inline]
    pub fn map_decode_domain<F, T>(self, f: F) -> TranscodeConvertError<T, EE, V>
    where
        F: FnOnce(DE) -> T,
    {
        match self {
            Self::Failure(failure) => TranscodeConvertError::Failure(failure),
            Self::DecodeDomain(error) => TranscodeConvertError::DecodeDomain(error.map_source(f)),
            Self::EncodeDomain(error) => TranscodeConvertError::EncodeDomain(error),
            Self::Unencodable { input_index, value } => TranscodeConvertError::Unencodable { input_index, value },
        }
    }

    /// Maps the target domain error while preserving other errors.
    ///
    /// # Type Parameters
    ///
    /// - `F`: One-shot transformation from `EE` to `T`.
    /// - `T`: Replacement payload type.
    ///
    /// # Parameters
    ///
    /// - `f`: Invoked only for a target-domain error; otherwise dropped without
    ///   being called.
    ///
    /// # Returns
    ///
    /// Returns an owned error preserving all other variants and positional
    /// context.
    ///
    /// # Panics
    ///
    /// Propagates a panic if the invoked transformation panics.
    #[inline]
    pub fn map_encode_domain<F, T>(self, f: F) -> TranscodeConvertError<DE, T, V>
    where
        F: FnOnce(EE) -> T,
    {
        match self {
            Self::Failure(failure) => TranscodeConvertError::Failure(failure),
            Self::DecodeDomain(error) => TranscodeConvertError::DecodeDomain(error),
            Self::EncodeDomain(error) => TranscodeConvertError::EncodeDomain(error.map_source(f)),
            Self::Unencodable { input_index, value } => TranscodeConvertError::Unencodable { input_index, value },
        }
    }

    /// Maps value context carried by unencodable-value errors.
    ///
    /// # Type Parameters
    ///
    /// - `F`: One-shot transformation from `V` to `W`.
    /// - `W`: Replacement payload type.
    ///
    /// # Parameters
    ///
    /// - `f`: Invoked only for present unencodable value context; otherwise
    ///   dropped without being called.
    ///
    /// # Returns
    ///
    /// Returns an owned error preserving all other variants and positional
    /// context.
    ///
    /// # Panics
    ///
    /// Propagates a panic if the invoked transformation panics.
    #[inline]
    pub fn map_value<F, W>(self, f: F) -> TranscodeConvertError<DE, EE, W>
    where
        F: FnOnce(V) -> W,
    {
        match self {
            Self::Failure(failure) => TranscodeConvertError::Failure(failure),
            Self::DecodeDomain(error) => TranscodeConvertError::DecodeDomain(error),
            Self::EncodeDomain(error) => TranscodeConvertError::EncodeDomain(error),
            Self::Unencodable { input_index, value } => TranscodeConvertError::Unencodable {
                input_index,
                value: value.map(f),
            },
        }
    }
}

impl<DE, EE, V> From<TranscodeDecodeError<DE>> for TranscodeConvertError<DE, EE, V> {
    /// Converts a source-side decode error into a converter error.
    ///
    /// # Parameters
    ///
    /// - `error`: Owned failure to convert without discarding its context.
    ///
    /// # Returns
    ///
    /// Returns the corresponding converter variant preserving the original
    /// cause.
    #[inline]
    fn from(error: TranscodeDecodeError<DE>) -> Self {
        match error {
            TranscodeDecodeError::Failure(failure) => Self::Failure(failure),
            TranscodeDecodeError::Domain(error) => Self::DecodeDomain(error),
        }
    }
}

impl<DE, EE, V> From<TranscodeEncodeError<EE, V>> for TranscodeConvertError<DE, EE, V> {
    /// Converts a target-side encode error into a converter error.
    ///
    /// # Parameters
    ///
    /// - `error`: Owned failure to convert without discarding its context.
    ///
    /// # Returns
    ///
    /// Returns the corresponding converter variant preserving the original
    /// cause.
    #[inline]
    fn from(error: TranscodeEncodeError<EE, V>) -> Self {
        match error {
            TranscodeEncodeError::Failure(failure) => Self::Failure(failure),
            TranscodeEncodeError::Unencodable { input_index, value } => Self::Unencodable { input_index, value },
            TranscodeEncodeError::Domain(error) => Self::EncodeDomain(error),
        }
    }
}

impl<DE, EE, V> From<CapacityError> for TranscodeConvertError<DE, EE, V> {
    /// Converts capacity planning errors into transcode framework errors.
    ///
    /// # Parameters
    ///
    /// - `error`: Owned failure to convert without discarding its context.
    ///
    /// # Returns
    ///
    /// Returns the corresponding converter variant preserving the original
    /// cause.
    #[inline]
    fn from(error: CapacityError) -> Self {
        TranscodeFailure::from(error).into()
    }
}
