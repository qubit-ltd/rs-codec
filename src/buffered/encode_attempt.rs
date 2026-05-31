/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Internal encode-step result used by buffered converters.

use core::num::NonZeroUsize;

use super::pending_value::PendingValue;

/// Result of one encode attempt in the converter loop.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum EncodeAttempt<Value> {
    /// The value was fully written.
    Written {
        /// Number of target units written.
        written: usize,
    },
    /// The value could not be written because target output is too small.
    NeedOutput {
        /// Retained value to write later.
        pending: PendingValue<Value>,
        /// Additional target units required to continue.
        additional: NonZeroUsize,
        /// Target units available at the output boundary.
        available: usize,
    },
}

impl<Value> EncodeAttempt<Value> {
    /// Creates a successful encode attempt.
    ///
    /// # Parameters
    ///
    /// - `written`: Number of target units written.
    ///
    /// # Returns
    ///
    /// Returns an encode attempt that made output progress.
    #[inline(always)]
    pub(super) const fn written(written: usize) -> Self {
        Self::Written { written }
    }

    /// Creates a missing-output encode attempt.
    ///
    /// # Parameters
    ///
    /// - `pending`: Decoded value that must remain retained.
    /// - `required`: Required output capacity.
    /// - `available`: Output capacity currently available.
    ///
    /// # Returns
    ///
    /// Returns an encode attempt containing the retained value and shortage.
    #[inline]
    pub(super) fn need_output(pending: PendingValue<Value>, required: usize, available: usize) -> Self {
        debug_assert!(required > available, "need-output attempt requires missing capacity");
        Self::NeedOutput {
            pending,
            additional: NonZeroUsize::new(required - available).expect("missing output is non-zero"),
            available,
        }
    }
}
