// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Failures returned by low-level codec decode operations.

use core::num::NonZeroUsize;

/// Failure reported by [`crate::Codec::decode`].
///
/// This type separates stream-control failures from codec-domain failures.
/// [`Incomplete`](Self::Incomplete) tells buffered adapters to preserve the
/// current input tail and request more units. [`Invalid`](Self::Invalid)
/// carries the codec-specific malformed, non-canonical, or otherwise invalid
/// input error.
///
/// # Type Parameters
///
/// - `E`: Codec-specific invalid-input error type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DecodeFailure<E> {
    /// The visible input is a valid prefix but not enough to decode a value.
    Incomplete {
        /// Non-zero total units required from the current value start.
        required_total: NonZeroUsize,
    },

    /// The input is invalid for the codec.
    Invalid {
        /// Codec-specific invalid-input error.
        source: E,
        /// Invalid units that may be consumed by a non-strict policy.
        consumed: Option<NonZeroUsize>,
    },
}

impl<E> DecodeFailure<E> {
    /// Creates an incomplete-input decode failure.
    ///
    /// # Parameters
    ///
    /// - `required_total`: Non-zero total units required from the current value
    ///   start.
    ///
    /// # Returns
    ///
    /// Returns an incomplete decode failure.
    #[inline(always)]
    #[must_use]
    pub const fn incomplete(required_total: NonZeroUsize) -> Self {
        Self::Incomplete { required_total }
    }

    /// Creates an invalid-input decode failure with a consumption hint.
    ///
    /// # Parameters
    ///
    /// - `source`: Codec-specific invalid-input error.
    /// - `consumed`: Invalid input units that may be consumed.
    ///
    /// # Returns
    ///
    /// Returns an invalid decode failure.
    #[inline(always)]
    #[must_use]
    pub const fn invalid(source: E, consumed: NonZeroUsize) -> Self {
        Self::Invalid {
            source,
            consumed: Some(consumed),
        }
    }

    /// Creates an invalid-input decode failure without a consumption hint.
    ///
    /// # Parameters
    ///
    /// - `source`: Codec-specific invalid-input error.
    ///
    /// # Returns
    ///
    /// Returns an invalid decode failure.
    #[inline(always)]
    #[must_use]
    pub const fn invalid_without_consumed(source: E) -> Self {
        Self::Invalid {
            source,
            consumed: None,
        }
    }

    /// Returns the total input units required for an incomplete prefix.
    ///
    /// # Returns
    ///
    /// Returns `Some(required_total)` for incomplete failures, or `None` for
    /// invalid-input failures.
    #[inline(always)]
    #[must_use]
    pub const fn required_total(&self) -> Option<NonZeroUsize> {
        match self {
            Self::Incomplete { required_total } => Some(*required_total),
            Self::Invalid { .. } => None,
        }
    }

    /// Borrows the codec-specific invalid-input error.
    ///
    /// # Returns
    ///
    /// Returns `Some(source)` for invalid-input failures, or `None` for
    /// incomplete failures.
    #[inline(always)]
    #[must_use]
    pub const fn invalid_source(&self) -> Option<&E> {
        match self {
            Self::Invalid { source, .. } => Some(source),
            Self::Incomplete { .. } => None,
        }
    }

    /// Returns invalid units that may be consumed by a non-strict policy.
    ///
    /// # Returns
    ///
    /// Returns `Some(consumed)` when the invalid failure carries a consumption
    /// hint, or `None` otherwise.
    #[inline(always)]
    #[must_use]
    pub const fn consumed_units(&self) -> Option<NonZeroUsize> {
        match self {
            Self::Invalid { consumed, .. } => *consumed,
            Self::Incomplete { .. } => None,
        }
    }
}
