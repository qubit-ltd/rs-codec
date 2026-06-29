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
    capacity_error::CapacityError,
    codec_phase::CodecPhase,
};
use crate::{
    Codec,
    DecodeFailure,
};

/// Intermediate error used by codec-backed encoders.
pub type TranscodeEncodeError<C> = TranscodeError<<C as Codec>::EncodeError>;

/// Intermediate error used by codec-backed decoders.
pub type TranscodeDecodeError<C> = TranscodeError<<C as Codec>::DecodeError>;

/// Error reported by a transcode operation.
///
/// Buffer contract failures are framework errors owned by the transcode layer
/// and are represented directly by this enum. Codec-, charset-, or
/// policy-specific failures are carried as domain errors.
///
/// # Type Parameters
///
/// - `E`: Domain error reported by the concrete transcoder.
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
#[non_exhaustive]
pub enum TranscodeError<E> {
    /// The caller supplied an input index outside the input slice.
    #[error("invalid input index {index} for input length {len}")]
    InvalidInputIndex {
        /// Invalid input index supplied by the caller.
        index: usize,
        /// Length of the input slice.
        len: usize,
    },

    /// The caller supplied an output index outside the output slice.
    #[error("invalid output index {index} for output length {len}")]
    InvalidOutputIndex {
        /// Invalid output index supplied by the caller.
        index: usize,
        /// Length of the output slice.
        len: usize,
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

    /// Domain-specific codec, charset, or policy error.
    #[error("codec {phase:?} error at input index {input_index:?}: {source}")]
    Domain {
        /// Domain error returned by the codec or policy facade.
        #[source]
        source: E,
        /// Codec lifecycle phase where the error occurred.
        phase: CodecPhase,
        /// Absolute input index when the phase is associated with an input
        /// value.
        input_index: Option<usize>,
    },
}

impl<E> TranscodeError<E> {
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
    pub const fn domain(
        source: E,
        phase: CodecPhase,
        input_index: Option<usize>,
    ) -> Self {
        Self::Domain {
            source,
            phase,
            input_index,
        }
    }

    /// Creates an invalid-input-index error.
    #[inline(always)]
    #[must_use]
    pub const fn invalid_input_index(index: usize, len: usize) -> Self {
        Self::InvalidInputIndex { index, len }
    }

    /// Creates an invalid-output-index error.
    #[inline(always)]
    #[must_use]
    pub const fn invalid_output_index(index: usize, len: usize) -> Self {
        Self::InvalidOutputIndex { index, len }
    }

    /// Creates an insufficient-output error.
    #[inline(always)]
    #[must_use]
    pub const fn insufficient_output(
        output_index: usize,
        required: usize,
        available: usize,
    ) -> Self {
        Self::InsufficientOutput {
            output_index,
            required,
            available,
        }
    }

    /// Creates an output-length-overflow error.
    #[inline(always)]
    #[must_use]
    pub const fn output_length_overflow() -> Self {
        Self::OutputLengthOverflow
    }

    /// Creates an incomplete-input error.
    #[inline(always)]
    #[must_use]
    pub const fn incomplete_input(
        input_index: usize,
        required: usize,
        available: usize,
    ) -> Self {
        Self::IncompleteInput {
            input_index,
            required,
            available,
        }
    }

    /// Creates a trailing-input error.
    #[inline(always)]
    #[must_use]
    pub const fn trailing_input(consumed: usize, remaining: usize) -> Self {
        Self::TrailingInput {
            consumed,
            remaining,
        }
    }

    /// Creates an unencodable-value error.
    #[inline(always)]
    #[must_use]
    pub const fn unencodable_value(input_index: usize) -> Self {
        Self::UnencodableValue { input_index }
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
    /// Returns [`TranscodeError::IncompleteInput`] for
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
                Self::incomplete_input(
                    input_index,
                    required_total.get(),
                    available,
                )
            }
            DecodeFailure::Invalid { source, .. } => {
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
        matches!(self, Self::Domain { .. })
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
            Self::Domain { source, .. } => Some(source),
            Self::InvalidInputIndex { .. }
            | Self::InvalidOutputIndex { .. }
            | Self::InsufficientOutput { .. }
            | Self::IncompleteInput { .. }
            | Self::TrailingInput { .. }
            | Self::UnencodableValue { .. }
            | Self::OutputLengthOverflow => None,
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
            Self::InvalidInputIndex { index, len } => {
                TranscodeError::InvalidInputIndex { index, len }
            }
            Self::InvalidOutputIndex { index, len } => {
                TranscodeError::InvalidOutputIndex { index, len }
            }
            Self::InsufficientOutput {
                output_index,
                required,
                available,
            } => TranscodeError::InsufficientOutput {
                output_index,
                required,
                available,
            },
            Self::OutputLengthOverflow => TranscodeError::OutputLengthOverflow,
            Self::IncompleteInput {
                input_index,
                required,
                available,
            } => TranscodeError::IncompleteInput {
                input_index,
                required,
                available,
            },
            Self::TrailingInput {
                consumed,
                remaining,
            } => TranscodeError::TrailingInput {
                consumed,
                remaining,
            },
            Self::UnencodableValue { input_index } => {
                TranscodeError::UnencodableValue { input_index }
            }
            Self::Domain {
                source,
                phase,
                input_index,
            } => TranscodeError::Domain {
                source: f(source),
                phase,
                input_index,
            },
        }
    }

    /// Validates that `input_index` is within an input slice.
    #[inline]
    pub fn ensure_input_index(
        input_len: usize,
        input_index: usize,
    ) -> Result<(), Self> {
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
    /// Returns [`TranscodeError::InvalidInputIndex`] when `input_index` is out
    /// of range. Returns [`TranscodeError::IncompleteInput`] when fewer than
    /// `min_required` units are available.
    #[inline]
    pub fn ensure_min_input(
        input_len: usize,
        input_index: usize,
        min_required: usize,
    ) -> Result<(), Self> {
        Self::ensure_input_index(input_len, input_index)?;
        let available = input_len - input_index;
        if available < min_required {
            return Err(Self::incomplete_input(
                input_index,
                min_required,
                available,
            ));
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
    /// Returns [`TranscodeError::TrailingInput`] when `consumed < total`.
    #[inline]
    pub fn ensure_no_trailing_input(
        consumed: usize,
        total: usize,
    ) -> Result<(), Self> {
        let remaining = total.saturating_sub(consumed);
        if remaining != 0 {
            return Err(Self::trailing_input(consumed, remaining));
        }
        Ok(())
    }

    /// Validates that `output_index` is within an output slice.
    #[inline]
    pub fn ensure_output_index(
        output_len: usize,
        output_index: usize,
    ) -> Result<(), Self> {
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
            return Err(Self::insufficient_output(
                output_index,
                required,
                available,
            ));
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
        if !qubit_io::UncheckedSlice::range_fits(
            output_len,
            output_index,
            range_len,
        ) {
            return Err(Self::invalid_output_index(output_index, output_len));
        }
        if range_len < required {
            return Err(Self::insufficient_output(
                output_index,
                required,
                range_len,
            ));
        }
        Ok(())
    }

    /// Maps this error into the I/O surface used by one-value encode adapters.
    ///
    /// Domain errors are forwarded through `map_domain`. Framework errors
    /// become `InvalidData`, except [`Self::UnencodableValue`] which maps
    /// to `InvalidInput` with the stable message expected by encode I/O
    /// helpers.
    pub fn into_encode_io_error<M>(self, map_domain: &mut M) -> std::io::Error
    where
        M: FnMut(E) -> std::io::Error,
    {
        use std::io::{
            Error,
            ErrorKind,
        };

        match self {
            Self::Domain { source, .. } => map_domain(source),
            Self::InvalidInputIndex { index, len } => Error::new(
                ErrorKind::InvalidData,
                format!("invalid input index {index} for input length {len}"),
            ),
            Self::InvalidOutputIndex { index, len } => Error::new(
                ErrorKind::InvalidData,
                format!("invalid output index {index} for output length {len}"),
            ),
            Self::InsufficientOutput {
                output_index,
                required,
                available,
            } => Error::new(
                ErrorKind::InvalidData,
                format!(
                    "insufficient output at index {output_index}: required {required} units, available {available}"
                ),
            ),
            Self::OutputLengthOverflow => Error::new(
                ErrorKind::InvalidData,
                "output length arithmetic overflow",
            ),
            Self::UnencodableValue { .. } => {
                Error::new(ErrorKind::InvalidInput, "codec cannot encode value")
            }
            Self::IncompleteInput {
                input_index,
                required,
                available,
            } => Error::new(
                ErrorKind::InvalidData,
                format!(
                    "incomplete input at index {input_index}: required {required} units, available {available}"
                ),
            ),
            Self::TrailingInput {
                consumed,
                remaining,
            } => Error::new(
                ErrorKind::InvalidData,
                format!(
                    "trailing input: consumed {consumed} units, remaining {remaining}"
                ),
            ),
        }
    }
}

impl<E> From<CapacityError> for TranscodeError<E> {
    /// Converts capacity planning errors into transcode framework errors.
    #[inline(always)]
    fn from(error: CapacityError) -> Self {
        match error {
            CapacityError::OutputLengthOverflow => Self::OutputLengthOverflow,
        }
    }
}
