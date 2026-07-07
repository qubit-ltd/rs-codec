// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors reported by transcode converters.

use thiserror::Error;

use super::{
    capacity_error::CapacityError,
    transcode_decode_error::TranscodeDecodeError,
    transcode_domain_error::TranscodeDomainError,
    transcode_encode_error::TranscodeEncodeError,
    transcode_failure::TranscodeFailure,
};
use crate::Codec;

/// Convert transcode error for a codec-backed converter.
pub type TranscodeConvertErrorOf<D, E> = TranscodeConvertError<
    <D as Codec>::DecodeError,
    <E as Codec>::EncodeError,
    <D as Codec>::Value,
>;

/// Error reported by a unit-to-unit transcode conversion.
#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
pub enum TranscodeConvertError<DE, EE, V> {
    /// Framework-level transcode failure.
    #[error(transparent)]
    Failure(#[from] TranscodeFailure),

    /// Source-side domain error.
    #[error("decode side failed: {0}")]
    DecodeDomain(#[source] TranscodeDomainError<DE>),

    /// Target-side domain error.
    #[error("encode side failed: {0}")]
    EncodeDomain(#[source] TranscodeDomainError<EE>),

    /// The decoded intermediate value cannot be encoded by the target codec and
    /// policy.
    #[error("unencodable value at input index {input_index}")]
    Unencodable {
        /// Absolute source input index of the value being encoded.
        input_index: usize,
        /// Decoded intermediate value, when available.
        value: Option<V>,
    },
}

impl<DE, EE, V> TranscodeConvertError<DE, EE, V> {
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

    /// Creates a source reset-phase domain-specific converter error.
    #[inline(always)]
    pub const fn decode_domain_reset(source: DE) -> Self {
        Self::DecodeDomain(TranscodeDomainError::reset(source))
    }

    /// Creates a source main-phase domain-specific converter error.
    #[inline(always)]
    pub const fn decode_domain_main(source: DE, input_index: usize) -> Self {
        Self::DecodeDomain(TranscodeDomainError::main(source, input_index))
    }

    /// Creates a source main-phase converter error with decode consumption
    /// context.
    #[inline(always)]
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
    #[inline(always)]
    pub const fn decode_domain_finish(source: DE) -> Self {
        Self::DecodeDomain(TranscodeDomainError::finish(source))
    }

    /// Creates a target reset-phase domain-specific converter error.
    #[inline(always)]
    pub const fn encode_domain_reset(source: EE) -> Self {
        Self::EncodeDomain(TranscodeDomainError::reset(source))
    }

    /// Creates a target main-phase domain-specific converter error.
    #[inline(always)]
    pub const fn encode_domain_main(source: EE, input_index: usize) -> Self {
        Self::EncodeDomain(TranscodeDomainError::main(source, input_index))
    }

    /// Creates a target finish-phase domain-specific converter error.
    #[inline(always)]
    pub const fn encode_domain_finish(source: EE) -> Self {
        Self::EncodeDomain(TranscodeDomainError::finish(source))
    }

    /// Creates an unencodable intermediate-value error with value context.
    #[inline(always)]
    #[must_use]
    pub fn unencodable(input_index: usize, value: V) -> Self {
        Self::Unencodable {
            input_index,
            value: Some(value),
        }
    }

    /// Creates an unencodable intermediate-value error without value context.
    #[inline(always)]
    #[must_use]
    pub const fn unencodable_without_context(input_index: usize) -> Self {
        Self::Unencodable {
            input_index,
            value: None,
        }
    }

    /// Returns the framework failure carried by this error.
    #[inline(always)]
    #[must_use]
    pub const fn failure_ref(&self) -> Option<&TranscodeFailure> {
        match self {
            Self::Failure(failure) => Some(failure),
            Self::DecodeDomain(_)
            | Self::EncodeDomain(_)
            | Self::Unencodable { .. } => None,
        }
    }

    /// Borrows the unencodable value context carried by this error.
    #[inline(always)]
    #[must_use]
    pub const fn unencodable_ref(&self) -> Option<(usize, Option<&V>)> {
        match self {
            Self::Unencodable { input_index, value } => {
                Some((*input_index, value.as_ref()))
            }
            Self::Failure(_)
            | Self::DecodeDomain(_)
            | Self::EncodeDomain(_) => None,
        }
    }

    /// Maps the source domain error while preserving other errors.
    #[inline]
    pub fn map_decode_domain<F, T>(
        self,
        f: F,
    ) -> TranscodeConvertError<T, EE, V>
    where
        F: FnOnce(DE) -> T,
    {
        match self {
            Self::Failure(failure) => TranscodeConvertError::Failure(failure),
            Self::DecodeDomain(error) => {
                TranscodeConvertError::DecodeDomain(error.map_source(f))
            }
            Self::EncodeDomain(error) => {
                TranscodeConvertError::EncodeDomain(error)
            }
            Self::Unencodable { input_index, value } => {
                TranscodeConvertError::Unencodable { input_index, value }
            }
        }
    }

    /// Maps the target domain error while preserving other errors.
    #[inline]
    pub fn map_encode_domain<F, T>(
        self,
        f: F,
    ) -> TranscodeConvertError<DE, T, V>
    where
        F: FnOnce(EE) -> T,
    {
        match self {
            Self::Failure(failure) => TranscodeConvertError::Failure(failure),
            Self::DecodeDomain(error) => {
                TranscodeConvertError::DecodeDomain(error)
            }
            Self::EncodeDomain(error) => {
                TranscodeConvertError::EncodeDomain(error.map_source(f))
            }
            Self::Unencodable { input_index, value } => {
                TranscodeConvertError::Unencodable { input_index, value }
            }
        }
    }

    /// Maps value context carried by unencodable-value errors.
    #[inline]
    pub fn map_value<F, W>(self, f: F) -> TranscodeConvertError<DE, EE, W>
    where
        F: FnOnce(V) -> W,
    {
        match self {
            Self::Failure(failure) => TranscodeConvertError::Failure(failure),
            Self::DecodeDomain(error) => {
                TranscodeConvertError::DecodeDomain(error)
            }
            Self::EncodeDomain(error) => {
                TranscodeConvertError::EncodeDomain(error)
            }
            Self::Unencodable { input_index, value } => {
                TranscodeConvertError::Unencodable {
                    input_index,
                    value: value.map(f),
                }
            }
        }
    }

    /// Converts an encode error into a converter error while adding fallback
    /// value context to unencodable errors that lack it.
    #[inline]
    pub fn from_encode_error_with_value(
        error: TranscodeEncodeError<EE, V>,
        fallback_value: V,
    ) -> Self {
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
}

impl<DE, EE, V> From<TranscodeDecodeError<DE>>
    for TranscodeConvertError<DE, EE, V>
{
    /// Converts a source-side decode error into a converter error.
    #[inline]
    fn from(error: TranscodeDecodeError<DE>) -> Self {
        match error {
            TranscodeDecodeError::Failure(failure) => Self::Failure(failure),
            TranscodeDecodeError::Domain(error) => Self::DecodeDomain(error),
        }
    }
}

impl<DE, EE, V> From<TranscodeEncodeError<EE, V>>
    for TranscodeConvertError<DE, EE, V>
{
    /// Converts a target-side encode error into a converter error.
    #[inline]
    fn from(error: TranscodeEncodeError<EE, V>) -> Self {
        match error {
            TranscodeEncodeError::Failure(failure) => Self::Failure(failure),
            TranscodeEncodeError::Unencodable { input_index, value } => {
                Self::Unencodable { input_index, value }
            }
            TranscodeEncodeError::Domain(error) => Self::EncodeDomain(error),
        }
    }
}

impl<DE, EE, V> From<CapacityError> for TranscodeConvertError<DE, EE, V> {
    /// Converts capacity planning errors into transcode framework errors.
    #[inline(always)]
    fn from(error: CapacityError) -> Self {
        TranscodeFailure::from(error).into()
    }
}
