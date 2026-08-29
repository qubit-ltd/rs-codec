// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Domain-specific transcode errors with codec phase context.

use core::num::NonZeroUsize;

use thiserror::Error;

/// Domain-specific codec, charset, or policy error with transcode context.
///
/// Reset and finish failures are lifecycle failures and therefore cannot carry
/// an input index. Main-phase failures always carry the absolute input index
/// where the domain error occurred.
///
/// # Type Parameters
///
/// - `E`: Domain error reported by the concrete codec, hook, or facade.
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum TranscodeDomainError<E> {
    /// Domain error reported while resetting codec or hook state.
    #[error("codec reset error: {source}")]
    Reset {
        /// Domain error returned by the codec or policy facade.
        #[source]
        source: E,
    },

    /// Domain error reported while processing input.
    #[error("codec main error at input index {input_index}: {source}")]
    Main {
        /// Domain error returned by the codec or policy facade.
        #[source]
        source: E,
        /// Absolute input index where the error occurred.
        input_index: usize,
        /// Invalid input units consumed by the codec-domain error, when known.
        input_consumed: Option<NonZeroUsize>,
    },

    /// Domain error reported while finishing codec or hook state.
    #[error("codec finish error: {source}")]
    Finish {
        /// Domain error returned by the codec or policy facade.
        #[source]
        source: E,
    },
}

impl<E> TranscodeDomainError<E> {
    /// Creates a reset-phase domain error.
    ///
    /// # Parameters
    ///
    /// - `source`: Domain error returned by reset handling.
    ///
    /// # Returns
    ///
    /// Returns a reset-phase domain error.
    #[inline(always)]
    #[must_use]
    pub const fn reset(source: E) -> Self {
        Self::Reset { source }
    }

    /// Creates a main-phase domain error.
    ///
    /// # Parameters
    ///
    /// - `source`: Domain error returned by value processing.
    /// - `input_index`: Absolute input index where the error occurred.
    ///
    /// # Returns
    ///
    /// Returns a main-phase domain error.
    #[inline(always)]
    #[must_use]
    pub const fn main(source: E, input_index: usize) -> Self {
        Self::Main {
            source,
            input_index,
            input_consumed: None,
        }
    }

    /// Creates a main-phase domain error with decode consumption context.
    ///
    /// # Parameters
    ///
    /// - `source`: Domain error returned by value processing.
    /// - `input_index`: Absolute input index where the error occurred.
    /// - `input_consumed`: Invalid input units consumed, when known.
    ///
    /// # Returns
    ///
    /// Returns a main-phase domain error with consumption context.
    #[inline(always)]
    #[must_use]
    pub const fn main_with_consumed(source: E, input_index: usize, input_consumed: Option<NonZeroUsize>) -> Self {
        Self::Main {
            source,
            input_index,
            input_consumed,
        }
    }

    /// Creates a finish-phase domain error.
    ///
    /// # Parameters
    ///
    /// - `source`: Domain error returned by finish handling.
    ///
    /// # Returns
    ///
    /// Returns a finish-phase domain error.
    #[inline(always)]
    #[must_use]
    pub const fn finish(source: E) -> Self {
        Self::Finish { source }
    }

    /// Returns the wrapped source error.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the domain error.
    #[inline(always)]
    #[must_use]
    pub const fn source(&self) -> &E {
        match self {
            Self::Reset { source } | Self::Main { source, .. } | Self::Finish { source } => source,
        }
    }

    /// Consumes this error and returns the wrapped source error.
    ///
    /// # Returns
    ///
    /// Returns the domain error.
    #[inline(always)]
    #[must_use]
    pub fn into_source(self) -> E {
        match self {
            Self::Reset { source } | Self::Main { source, .. } | Self::Finish { source } => source,
        }
    }

    /// Returns the absolute input index associated with the error.
    ///
    /// # Returns
    ///
    /// Returns `Some(index)` for main-phase errors and `None` for reset or
    /// finish errors.
    #[inline(always)]
    #[must_use]
    pub const fn input_index(&self) -> Option<usize> {
        match self {
            Self::Main { input_index, .. } => Some(*input_index),
            Self::Reset { .. } | Self::Finish { .. } => None,
        }
    }

    /// Returns invalid input consumption associated with the error.
    ///
    /// # Returns
    ///
    /// Returns `Some(consumed)` when a main-phase decode error reports the
    /// invalid input width. Returns `None` otherwise.
    #[inline(always)]
    #[must_use]
    pub const fn input_consumed(&self) -> Option<NonZeroUsize> {
        match self {
            Self::Main { input_consumed, .. } => *input_consumed,
            Self::Reset { .. } | Self::Finish { .. } => None,
        }
    }

    /// Maps the wrapped source error while preserving phase context.
    ///
    /// # Type Parameters
    ///
    /// - `T`: Target source error type.
    /// - `F`: Mapping function type.
    ///
    /// # Parameters
    ///
    /// - `f`: Function applied to the wrapped source error.
    ///
    /// # Returns
    ///
    /// Returns the mapped domain error.
    #[inline]
    pub fn map_source<T, F>(self, f: F) -> TranscodeDomainError<T>
    where
        F: FnOnce(E) -> T,
    {
        match self {
            Self::Reset { source } => TranscodeDomainError::Reset { source: f(source) },
            Self::Main {
                source,
                input_index,
                input_consumed,
            } => TranscodeDomainError::Main {
                source: f(source),
                input_index,
                input_consumed,
            },
            Self::Finish { source } => TranscodeDomainError::Finish { source: f(source) },
        }
    }
}
