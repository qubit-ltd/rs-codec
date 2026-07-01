// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors reported by transcode engines and transcoder adapters.

use thiserror::Error;

use super::{
    capacity_error::CapacityError, codec_phase::CodecPhase,
    transcode_domain_error::TranscodeDomainError, transcode_failure::TranscodeFailure,
};
use crate::{Codec, DecodeFailure};

/// Intermediate error used by codec-backed encoders.
pub type TranscodeEncodeError<C> = TranscodeError<<C as Codec>::EncodeError, <C as Codec>::Value>;

/// Intermediate error used by codec-backed decoders.
pub type TranscodeDecodeError<C> = TranscodeError<<C as Codec>::DecodeError>;

/// Error reported by a transcode operation.
///
/// Transcode errors separate framework failures from codec-, charset-, or
/// policy-specific domain errors. Implementation contract violations are not
/// represented here; callers that need to validate progress should use
/// [`crate::TranscodeProgress::validate`] and handle
/// [`crate::TranscodeContractError`] separately.
///
/// # Type Parameters
///
/// - `E`: Domain error reported by the concrete transcoder.
/// - `Value`: Value context carried by framework failures when available.
#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
pub enum TranscodeError<E, Value = ()> {
    /// Framework-level transcode failure.
    #[error(transparent)]
    Failure(#[from] TranscodeFailure<Value>),

    /// Domain-specific codec, charset, or policy error.
    #[error(transparent)]
    Domain(#[from] TranscodeDomainError<E>),
}

impl<E, Value> TranscodeError<E, Value> {
    /// Creates a domain-specific transcode error.
    ///
    /// # Parameters
    ///
    /// - `source`: Domain error reported by the codec or policy.
    /// - `phase`: Codec lifecycle phase where the error occurred.
    /// - `input_index`: Absolute input index when the error is tied to an input
    ///   value.
    ///
    /// # Returns
    ///
    /// Returns a transcode error wrapping `error`.
    #[inline(always)]
    pub const fn domain(source: E, phase: CodecPhase, input_index: Option<usize>) -> Self {
        Self::Domain(TranscodeDomainError {
            source,
            phase,
            input_index,
            input_consumed: None,
        })
    }

    /// Creates a domain-specific transcode error with decode consumption
    /// context.
    #[inline(always)]
    pub const fn domain_with_consumed(
        source: E,
        phase: CodecPhase,
        input_index: Option<usize>,
        input_consumed: Option<core::num::NonZeroUsize>,
    ) -> Self {
        Self::Domain(TranscodeDomainError {
            source,
            phase,
            input_index,
            input_consumed,
        })
    }

    /// Creates an invalid-input-index error.
    #[inline(always)]
    #[must_use]
    pub const fn invalid_input_index(index: usize, len: usize) -> Self {
        Self::Failure(TranscodeFailure::InvalidInputIndex {
            index,
            input_len: len,
        })
    }

    /// Creates an invalid-output-index error.
    #[inline(always)]
    #[must_use]
    pub const fn invalid_output_index(index: usize, len: usize) -> Self {
        Self::Failure(TranscodeFailure::InvalidOutputIndex {
            index,
            output_len: len,
        })
    }

    /// Creates an insufficient-output error.
    #[inline(always)]
    #[must_use]
    pub const fn insufficient_output(
        output_index: usize,
        required: usize,
        available: usize,
    ) -> Self {
        Self::Failure(TranscodeFailure::InsufficientOutput {
            output_index,
            required,
            available,
        })
    }

    /// Creates an output-length-overflow error.
    #[inline(always)]
    #[must_use]
    pub const fn output_length_overflow() -> Self {
        Self::Failure(TranscodeFailure::OutputLengthOverflow)
    }

    /// Creates an incomplete-input error.
    #[inline(always)]
    #[must_use]
    pub const fn incomplete_input(input_index: usize, required: usize, available: usize) -> Self {
        Self::Failure(TranscodeFailure::IncompleteInput {
            input_index,
            required,
            available,
        })
    }

    /// Creates a trailing-input error.
    #[inline(always)]
    #[must_use]
    pub const fn trailing_input(consumed: usize, remaining: usize) -> Self {
        Self::Failure(TranscodeFailure::TrailingInput {
            consumed,
            remaining,
        })
    }

    /// Creates an unencodable-value error with value context.
    #[inline(always)]
    #[must_use]
    pub fn unencodable_value(input_index: usize, value: Value) -> Self {
        Self::Failure(TranscodeFailure::UnencodableValue {
            input_index,
            value: Some(value),
        })
    }

    /// Creates an unencodable-value error without value context.
    #[inline(always)]
    #[must_use]
    pub const fn unencodable_value_without_context(input_index: usize) -> Self {
        Self::Failure(TranscodeFailure::UnencodableValue {
            input_index,
            value: None,
        })
    }

    /// Converts a low-level decode failure into a transcode error.
    ///
    /// This helper is intended for one-shot value decode paths that reject
    /// decode failures directly instead of routing them through streaming
    /// decode hooks.
    ///
    /// # Parameters
    ///
    /// - `failure`: Failure reported by [`Codec::decode`].
    /// - `input_index`: Absolute input index where decoding started.
    /// - `available`: Input units available from `input_index`.
    ///
    /// # Returns
    ///
    /// Returns [`TranscodeFailure::IncompleteInput`] for
    /// [`DecodeFailure::Incomplete`] and [`TranscodeError::Domain`] for
    /// [`DecodeFailure::Invalid`].
    #[inline]
    #[must_use]
    pub fn from_decode_failure(
        failure: DecodeFailure<E>,
        input_index: usize,
        available: usize,
    ) -> Self {
        match failure {
            DecodeFailure::Incomplete { required_total } => {
                Self::incomplete_input(input_index, required_total.get(), available)
            }
            DecodeFailure::Invalid { source, consumed } => Self::domain_with_consumed(
                source,
                CodecPhase::Main,
                Some(input_index),
                Some(consumed),
            ),
            DecodeFailure::InvalidUnknown { source } => {
                Self::domain(source, CodecPhase::Main, Some(input_index))
            }
        }
    }

    /// Returns whether this error wraps a domain error.
    ///
    /// # Returns
    ///
    /// Returns `true` for [`TranscodeError::Domain`].
    #[inline(always)]
    #[must_use]
    pub const fn is_domain(&self) -> bool {
        matches!(self, Self::Domain(_))
    }

    /// Returns the framework failure carried by this error.
    ///
    /// # Returns
    ///
    /// Returns `Some(failure)` for [`TranscodeError::Failure`] and `None` for
    /// domain errors.
    #[inline(always)]
    #[must_use]
    pub const fn failure_ref(&self) -> Option<&TranscodeFailure<Value>> {
        match self {
            Self::Failure(failure) => Some(failure),
            Self::Domain(_) => None,
        }
    }

    /// Returns the framework failure carried by this error.
    ///
    /// # Returns
    ///
    /// Returns `Some(failure)` for [`TranscodeError::Failure`] and `None` for
    /// domain errors.
    #[inline(always)]
    #[must_use]
    pub fn failure(&self) -> Option<TranscodeFailure<Value>>
    where
        Value: Clone,
    {
        self.failure_ref().cloned()
    }

    /// Borrows the wrapped domain error and transcode context.
    ///
    /// # Returns
    ///
    /// Returns `Some(error)` for [`TranscodeError::Domain`] and `None` for
    /// framework failures.
    #[inline(always)]
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
    /// Returns `Some(error)` for [`TranscodeError::Domain`] and `None` for
    /// buffer contract errors.
    #[must_use]
    pub const fn domain_ref(&self) -> Option<&E> {
        match self {
            Self::Domain(error) => Some(&error.source),
            Self::Failure(_) => None,
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
    pub fn map_domain<F, T>(self, f: F) -> TranscodeError<T, Value>
    where
        F: FnOnce(E) -> T,
    {
        match self {
            Self::Failure(failure) => TranscodeError::Failure(failure),
            Self::Domain(error) => TranscodeError::Domain(TranscodeDomainError {
                source: f(error.source),
                phase: error.phase,
                input_index: error.input_index,
                input_consumed: error.input_consumed,
            }),
        }
    }

    /// Maps value context carried by framework failures while preserving the
    /// domain error type.
    #[inline]
    pub fn map_failure_value<T, F>(self, f: F) -> TranscodeError<E, T>
    where
        F: FnOnce(Value) -> T,
    {
        match self {
            Self::Failure(failure) => TranscodeError::Failure(failure.map_value(f)),
            Self::Domain(error) => TranscodeError::Domain(error),
        }
    }

    /// Validates that `input_index` is within an input slice.
    #[inline]
    pub fn ensure_input_index(input_len: usize, input_index: usize) -> Result<(), Self> {
        if input_index > input_len {
            return Err(Self::invalid_input_index(input_index, input_len));
        }
        Ok(())
    }

    /// Validates that enough input units are available from `input_index`.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Length of the input slice.
    /// - `input_index`: Start index in the input slice.
    /// - `min_required`: Minimum input units required from `input_index`.
    ///
    /// # Errors
    ///
    /// Returns [`TranscodeFailure::InvalidInputIndex`] when `input_index` is
    /// out of range. Returns [`TranscodeFailure::IncompleteInput`] when fewer
    /// than `min_required` units are available.
    #[inline]
    pub fn ensure_min_input(
        input_len: usize,
        input_index: usize,
        min_required: usize,
    ) -> Result<(), Self> {
        Self::ensure_input_index(input_len, input_index)?;
        let available = input_len - input_index;
        if available < min_required {
            return Err(Self::incomplete_input(input_index, min_required, available));
        }
        Ok(())
    }

    /// Validates that no input units remain after a decoded value.
    ///
    /// # Parameters
    ///
    /// - `consumed`: Units consumed by the decoded value.
    /// - `total`: Total input units in the slice.
    ///
    /// # Errors
    ///
    /// Returns [`TranscodeFailure::TrailingInput`] when `consumed < total`.
    #[inline]
    pub fn ensure_no_trailing_input(consumed: usize, total: usize) -> Result<(), Self> {
        let remaining = total.saturating_sub(consumed);
        if remaining != 0 {
            return Err(Self::trailing_input(consumed, remaining));
        }
        Ok(())
    }

    /// Validates that `output_index` is within an output slice.
    #[inline]
    pub fn ensure_output_index(output_len: usize, output_index: usize) -> Result<(), Self> {
        if output_index > output_len {
            return Err(Self::invalid_output_index(output_index, output_len));
        }
        Ok(())
    }

    /// Validates input and output start indices for a transcode call.
    #[inline]
    pub fn ensure_transcode_indices(
        input_len: usize,
        input_index: usize,
        output_len: usize,
        output_index: usize,
    ) -> Result<(), Self> {
        Self::ensure_input_index(input_len, input_index)?;
        Self::ensure_output_index(output_len, output_index)
    }

    /// Validates that an output slice can hold one-shot finalization output.
    #[inline]
    pub fn ensure_output_capacity(
        output_len: usize,
        output_index: usize,
        required: usize,
    ) -> Result<(), Self> {
        Self::ensure_output_index(output_len, output_index)?;
        let available = output_len - output_index;
        if available < required {
            return Err(Self::insufficient_output(output_index, required, available));
        }
        Ok(())
    }

    /// Validates an indexed output range and its minimum writable capacity.
    #[inline]
    pub fn ensure_output_range(
        output_len: usize,
        output_index: usize,
        range_len: usize,
        required: usize,
    ) -> Result<(), Self> {
        Self::ensure_output_index(output_len, output_index)?;
        if !qubit_io::UncheckedSlice::range_fits(output_len, output_index, range_len) {
            return Err(Self::invalid_output_index(output_index, output_len));
        }
        if range_len < required {
            return Err(Self::insufficient_output(output_index, required, range_len));
        }
        Ok(())
    }

    /// Maps this error into the I/O surface used by one-value encode adapters.
    ///
    /// Domain errors are forwarded through `map_domain`. Framework errors
    /// become `InvalidData`, except
    /// [`TranscodeFailure::UnencodableValue`] which maps to `InvalidInput`
    /// with the stable message expected by encode I/O helpers.
    pub fn into_encode_io_error<M>(self, map_domain: &mut M) -> std::io::Error
    where
        M: FnMut(E) -> std::io::Error,
    {
        use std::io::{Error, ErrorKind};

        match self {
            Self::Domain(error) => map_domain(error.source),
            Self::Failure(TranscodeFailure::InvalidInputIndex { index, input_len }) => Error::new(
                ErrorKind::InvalidData,
                format!("invalid input index {index} for input length {input_len}"),
            ),
            Self::Failure(TranscodeFailure::InvalidOutputIndex { index, output_len }) => {
                Error::new(
                    ErrorKind::InvalidData,
                    format!("invalid output index {index} for output length {output_len}"),
                )
            }
            Self::Failure(TranscodeFailure::InsufficientOutput {
                output_index,
                required,
                available,
            }) => Error::new(
                ErrorKind::InvalidData,
                format!(
                    "insufficient output at index {output_index}: required {required} units, available {available}"
                ),
            ),
            Self::Failure(TranscodeFailure::OutputLengthOverflow) => {
                Error::new(ErrorKind::InvalidData, "output length arithmetic overflow")
            }
            Self::Failure(TranscodeFailure::UnencodableValue { .. }) => {
                Error::new(ErrorKind::InvalidInput, "codec cannot encode value")
            }
            Self::Failure(TranscodeFailure::IncompleteInput {
                input_index,
                required,
                available,
            }) => Error::new(
                ErrorKind::InvalidData,
                format!(
                    "incomplete input at index {input_index}: required {required} units, available {available}"
                ),
            ),
            Self::Failure(TranscodeFailure::TrailingInput {
                consumed,
                remaining,
            }) => Error::new(
                ErrorKind::InvalidData,
                format!("trailing input: consumed {consumed} units, remaining {remaining}"),
            ),
        }
    }

    /// Maps this error into the I/O surface used by decode adapters.
    ///
    /// Domain errors are forwarded through `map_domain`. Framework errors
    /// become `InvalidData` because they describe malformed input, invalid
    /// caller ranges, or impossible output planning at the decode boundary.
    pub fn into_decode_io_error<M>(self, map_domain: &mut M) -> std::io::Error
    where
        M: FnMut(E) -> std::io::Error,
    {
        use std::io::{Error, ErrorKind};

        match self {
            Self::Domain(error) => map_domain(error.source),
            Self::Failure(TranscodeFailure::InvalidInputIndex { index, input_len }) => Error::new(
                ErrorKind::InvalidData,
                format!("invalid input index {index} for input length {input_len}"),
            ),
            Self::Failure(TranscodeFailure::InvalidOutputIndex { index, output_len }) => {
                Error::new(
                    ErrorKind::InvalidData,
                    format!("invalid output index {index} for output length {output_len}"),
                )
            }
            Self::Failure(TranscodeFailure::InsufficientOutput {
                output_index,
                required,
                available,
            }) => Error::new(
                ErrorKind::InvalidData,
                format!(
                    "insufficient output at index {output_index}: required {required} units, available {available}"
                ),
            ),
            Self::Failure(TranscodeFailure::OutputLengthOverflow) => {
                Error::new(ErrorKind::InvalidData, "output length arithmetic overflow")
            }
            Self::Failure(TranscodeFailure::UnencodableValue { .. }) => {
                Error::new(ErrorKind::InvalidData, "codec cannot encode value")
            }
            Self::Failure(TranscodeFailure::IncompleteInput {
                input_index,
                required,
                available,
            }) => Error::new(
                ErrorKind::InvalidData,
                format!(
                    "incomplete input at index {input_index}: required {required} units, available {available}"
                ),
            ),
            Self::Failure(TranscodeFailure::TrailingInput {
                consumed,
                remaining,
            }) => Error::new(
                ErrorKind::InvalidData,
                format!("trailing input: consumed {consumed} units, remaining {remaining}"),
            ),
        }
    }
}

impl<E, Value> From<CapacityError> for TranscodeError<E, Value> {
    /// Converts capacity planning errors into transcode framework errors.
    #[inline(always)]
    fn from(error: CapacityError) -> Self {
        match error {
            CapacityError::OutputLengthOverflow => Self::output_length_overflow(),
        }
    }
}
