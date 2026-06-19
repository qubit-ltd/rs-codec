// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors reported by transcode engines and transcoder adapters.

use core::fmt;

/// Error reported by a transcode operation.
///
/// The enum keeps caller-contract failures in the framework layer and stores
/// codec-, charset-, or policy-specific failures in [`TranscodeError::Domain`].
///
/// # Type Parameters
///
/// - `E`: Domain error reported by the concrete transcoder.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TranscodeError<E> {
    /// The caller supplied an input index beyond the input slice length.
    InvalidInputIndex {
        /// Invalid input index supplied by the caller.
        index: usize,
        /// Length of the input slice.
        len: usize,
    },
    /// The caller supplied an output index beyond the output slice length.
    InvalidOutputIndex {
        /// Invalid output index supplied by the caller.
        index: usize,
        /// Length of the output slice.
        len: usize,
    },
    /// The output slice cannot hold required one-shot reset or finish output.
    InsufficientOutput {
        /// Output index supplied by the caller.
        output_index: usize,
        /// Required writable output units.
        required: usize,
        /// Available writable output units.
        available: usize,
    },
    /// Output length arithmetic overflowed.
    OutputLengthOverflow,
    /// Domain-specific codec, charset, or policy error.
    Domain(E),
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
    ///
    /// # Parameters
    ///
    /// - `index`: Invalid input index supplied by the caller.
    /// - `len`: Length of the input slice.
    ///
    /// # Returns
    ///
    /// Returns the invalid-input-index error.
    #[inline(always)]
    pub const fn invalid_input_index(index: usize, len: usize) -> Self {
        Self::InvalidInputIndex { index, len }
    }

    /// Creates an invalid-output-index error.
    ///
    /// # Parameters
    ///
    /// - `index`: Invalid output index supplied by the caller.
    /// - `len`: Length of the output slice.
    ///
    /// # Returns
    ///
    /// Returns the invalid-output-index error.
    #[inline(always)]
    pub const fn invalid_output_index(index: usize, len: usize) -> Self {
        Self::InvalidOutputIndex { index, len }
    }

    /// Creates an insufficient-output error.
    ///
    /// # Parameters
    ///
    /// - `output_index`: Output index supplied by the caller.
    /// - `required`: Output units required to finish in one call.
    /// - `available`: Output units available from `output_index`.
    ///
    /// # Returns
    ///
    /// Returns the insufficient-output error.
    #[inline(always)]
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
    /// framework contract errors.
    #[must_use]
    #[inline(always)]
    pub const fn domain_ref(&self) -> Option<&E> {
        match self {
            Self::Domain(error) => Some(error),
            _ => None,
        }
    }

    /// Maps the wrapped domain error while preserving framework errors.
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
    ///
    /// # Parameters
    ///
    /// - `input_len`: Length of the input slice.
    /// - `input_index`: Input index supplied by the caller.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when `input_index <= input_len`.
    ///
    /// # Errors
    ///
    /// Returns an invalid-input-index error when `input_index` is beyond the
    /// slice.
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

    /// Validates that `output_index` is within an output slice.
    ///
    /// # Parameters
    ///
    /// - `output_len`: Length of the output slice.
    /// - `output_index`: Output index supplied by the caller.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when `output_index <= output_len`.
    ///
    /// # Errors
    ///
    /// Returns an invalid-output-index error when `output_index` is beyond the
    /// slice.
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
    ///
    /// # Parameters
    ///
    /// - `input_len`: Length of the input slice.
    /// - `input_index`: Input index supplied by the caller.
    /// - `output_len`: Length of the output slice.
    /// - `output_index`: Output index supplied by the caller.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when both indices are within their slices.
    ///
    /// # Errors
    ///
    /// Returns an invalid-input-index or invalid-output-index error when either
    /// index is beyond its slice.
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
    ///
    /// # Parameters
    ///
    /// - `output_len`: Length of the output slice.
    /// - `output_index`: Output index supplied by the caller.
    /// - `required`: Output units required to finish in one call.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when output capacity is sufficient.
    ///
    /// # Errors
    ///
    /// Returns an invalid-output-index error when `output_index` is beyond the
    /// slice, or an insufficient-output error when fewer than `required` units
    /// are writable from `output_index`.
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
    ///
    /// # Parameters
    ///
    /// - `output_len`: Length of the output slice.
    /// - `output_index`: Output index supplied by the caller.
    /// - `range_len`: Length of the writable range starting at `output_index`.
    /// - `required`: Minimum output units required inside the range.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the range fits inside the slice and can hold
    /// `required` units.
    ///
    /// # Errors
    ///
    /// Returns an invalid-output-index error when the range overflows or
    /// extends beyond the slice, or an insufficient-output error when
    /// `range_len` is smaller than `required`.
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
}

impl<E> From<E> for TranscodeError<E> {
    /// Wraps a domain error in [`TranscodeError::Domain`].
    ///
    /// # Parameters
    ///
    /// - `error`: Domain error to wrap.
    ///
    /// # Returns
    ///
    /// Returns the wrapped transcode error.
    #[inline(always)]
    fn from(error: E) -> Self {
        Self::Domain(error)
    }
}

impl<E> fmt::Display for TranscodeError<E>
where
    E: fmt::Display,
{
    /// Formats the transcode error.
    ///
    /// # Parameters
    ///
    /// - `f`: Destination formatter.
    ///
    /// # Returns
    ///
    /// Returns the formatter result.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInputIndex { index, len } => {
                write!(f, "invalid input index {index}; input length is {len}")
            }
            Self::InvalidOutputIndex { index, len } => {
                write!(
                    f,
                    "invalid output index {index}; output length is {len}"
                )
            }
            Self::InsufficientOutput {
                output_index,
                required,
                available,
            } => write!(
                f,
                "insufficient output at index {output_index}; required {required}, available {available}"
            ),
            Self::OutputLengthOverflow => {
                f.write_str("output length arithmetic overflow")
            }
            Self::Domain(error) => error.fmt(f),
        }
    }
}

impl<E> std::error::Error for TranscodeError<E>
where
    E: std::error::Error + 'static,
{
    /// Returns the source domain error, if any.
    ///
    /// # Returns
    ///
    /// Returns `Some(error)` for [`TranscodeError::Domain`] and `None` for
    /// framework contract errors.
    #[inline(always)]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Domain(error) => Some(error),
            _ => None,
        }
    }
}
