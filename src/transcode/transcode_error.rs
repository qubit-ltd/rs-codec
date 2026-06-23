// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors reported by transcode engines and transcoder adapters.

use thiserror::Error;

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

    /// Borrows the wrapped domain error.
    ///
    /// # Returns
    ///
    /// Returns `Some(error)` for [`TranscodeError::Domain`] and `None` for
    /// buffer contract errors.
    #[inline(always)]
    #[must_use]
    pub const fn domain_ref(&self) -> Option<&E> {
        match self {
            Self::Domain(error) => Some(error),
            Self::InvalidInputIndex { .. }
            | Self::InvalidOutputIndex { .. }
            | Self::InsufficientOutput { .. }
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
            Self::Domain(error) => TranscodeError::Domain(f(error)),
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
}
