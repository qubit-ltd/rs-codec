/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Generic decode failure metadata.

/// Control-flow view of a codec-specific decode error.
///
/// This enum does not replace codec-specific error types. It exposes only the
/// small amount of information a generic buffered caller needs to decide whether
/// it should wait for more input or consume invalid input and make progress.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DecodeFailure {
    /// The currently available units are a valid prefix but not a complete value.
    Incomplete {
        /// Total units required from the current decode start.
        required: usize,
        /// Units currently available from the current decode start.
        available: usize,
    },

    /// The current units cannot form a valid value.
    Invalid {
        /// Number of units a buffered caller may consume to make progress.
        consumed: usize,
    },
}

impl DecodeFailure {
    /// Returns incomplete-input details when this failure is incomplete.
    ///
    /// # Returns
    ///
    /// Returns `Some((required, available))` for [`Self::Incomplete`], or
    /// `None` for [`Self::Invalid`].
    #[must_use]
    #[inline]
    pub const fn incomplete(self) -> Option<(usize, usize)> {
        match self {
            Self::Incomplete { required, available } => Some((required, available)),
            Self::Invalid { .. } => None,
        }
    }

    /// Returns invalid-input consumption when this failure is invalid.
    ///
    /// # Returns
    ///
    /// Returns `Some(consumed)` for [`Self::Invalid`], or `None` for
    /// [`Self::Incomplete`].
    #[must_use]
    #[inline]
    pub const fn invalid_consumed(self) -> Option<usize> {
        match self {
            Self::Invalid { consumed } => Some(consumed),
            Self::Incomplete { .. } => None,
        }
    }
}
