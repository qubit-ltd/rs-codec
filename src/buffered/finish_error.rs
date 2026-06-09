// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Error reported while finishing buffered output.

use thiserror::Error;

use super::capacity_error::CapacityError;

/// Error reported by one-shot buffered finalization.
///
/// `finish` methods require enough output capacity to write all final output in
/// one call. This type separates caller capacity mistakes from semantic errors
/// reported by the concrete codec or hook policy.
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum FinishError<E> {
    /// Finish-output bound arithmetic overflowed.
    #[error("finish output capacity planning failed: {source}")]
    Capacity {
        /// Capacity planning error.
        #[source]
        source: CapacityError,
    },

    /// The caller supplied an output index outside the output slice.
    #[error("invalid finish output index {index} for output length {len}")]
    InvalidOutputIndex {
        /// Invalid output index supplied by the caller.
        index: usize,
        /// Length of the output slice.
        len: usize,
    },

    /// The output slice cannot hold all final output.
    #[error(
        "insufficient finish output at index {output_index}: required {required} units, available {available}"
    )]
    InsufficientOutput {
        /// Absolute output index where finalization would start writing.
        output_index: usize,
        /// Output units required to finish in one call.
        required: usize,
        /// Output units available from `output_index`.
        available: usize,
    },

    /// The underlying codec or hook policy rejected finalization.
    #[error("finish failed: {source}")]
    Source {
        /// Error returned by the concrete codec or hook policy.
        #[source]
        source: E,
    },
}

impl<E> FinishError<E> {
    /// Creates a finish error from capacity planning failure.
    ///
    /// # Parameters
    ///
    /// - `source`: Capacity planning error reported while computing the finish
    ///   bound.
    ///
    /// # Returns
    ///
    /// Returns a finish error wrapping `source`.
    #[must_use]
    #[inline(always)]
    pub const fn capacity(source: CapacityError) -> Self {
        Self::Capacity { source }
    }

    /// Creates an invalid-output-index finish error.
    ///
    /// # Parameters
    ///
    /// - `index`: Output index supplied by the caller.
    /// - `len`: Length of the output slice.
    ///
    /// # Returns
    ///
    /// Returns an invalid-output-index finish error.
    #[must_use]
    #[inline(always)]
    pub const fn invalid_output_index(index: usize, len: usize) -> Self {
        Self::InvalidOutputIndex { index, len }
    }

    /// Creates an insufficient-output finish error.
    ///
    /// # Parameters
    ///
    /// - `output_index`: Output index supplied by the caller.
    /// - `required`: Output units required to finish in one call.
    /// - `available`: Output units available from `output_index`.
    ///
    /// # Returns
    ///
    /// Returns an insufficient-output finish error.
    #[must_use]
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

    /// Creates a finish error from an underlying semantic error.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by the concrete codec or hook policy.
    ///
    /// # Returns
    ///
    /// Returns a finish error wrapping `source`.
    #[must_use]
    #[inline(always)]
    pub const fn source(source: E) -> Self {
        Self::Source { source }
    }

    /// Maps the underlying semantic error while preserving capacity failures.
    ///
    /// # Type Parameters
    ///
    /// - `T`: Target semantic error type.
    /// - `F`: Mapping function type.
    ///
    /// # Parameters
    ///
    /// - `map`: Function used to map the underlying semantic error.
    ///
    /// # Returns
    ///
    /// Returns a finish error with the mapped semantic error type.
    #[inline]
    pub fn map_source<T, F>(self, map: F) -> FinishError<T>
    where
        F: FnOnce(E) -> T,
    {
        match self {
            Self::Capacity { source } => FinishError::Capacity { source },
            Self::InvalidOutputIndex { index, len } => {
                FinishError::InvalidOutputIndex { index, len }
            }
            Self::InsufficientOutput {
                output_index,
                required,
                available,
            } => FinishError::InsufficientOutput {
                output_index,
                required,
                available,
            },
            Self::Source { source } => FinishError::Source {
                source: map(source),
            },
        }
    }

    /// Validates that an output slice can hold one-shot final output.
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
    /// Returns [`FinishError::InvalidOutputIndex`] when `output_index` is
    /// beyond the slice, or [`FinishError::InsufficientOutput`] when fewer
    /// than `required` units are writable from `output_index`.
    #[inline]
    pub fn ensure_output_capacity(
        output_len: usize,
        output_index: usize,
        required: usize,
    ) -> Result<(), Self> {
        if output_index > output_len {
            return Err(Self::invalid_output_index(output_index, output_len));
        }
        let available = output_len - output_index;
        if available < required {
            return Err(Self::insufficient_output(output_index, required, available));
        }
        Ok(())
    }
}
