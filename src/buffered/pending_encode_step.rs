/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Internal pending-value encode step used by buffered converters.

use core::num::NonZeroUsize;

use super::pending_value::PendingValue;

/// Result of encoding a pending decoded value in the converter loop.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum PendingEncodeStep<Value> {
    /// The pending value was fully written.
    Written {
        /// Number of target units written.
        written: usize,
    },
    /// The pending value could not be written because output is too small.
    NeedOutput {
        /// Retained value to write later.
        pending: PendingValue<Value>,
        /// Additional target units required to continue.
        additional: NonZeroUsize,
        /// Target units available at the output boundary.
        available: usize,
    },
}

impl<Value> PendingEncodeStep<Value> {
    /// Creates a successful pending-value encode step.
    ///
    /// # Parameters
    ///
    /// - `written`: Number of target units written.
    ///
    /// # Returns
    ///
    /// Returns a step that made output progress.
    #[inline(always)]
    pub(super) const fn written(written: usize) -> Self {
        Self::Written { written }
    }

    /// Creates a missing-output pending-value encode step.
    ///
    /// # Parameters
    ///
    /// - `pending`: Decoded value that must remain retained.
    /// - `additional`: Additional output capacity required to continue.
    /// - `available`: Output capacity currently available.
    ///
    /// # Returns
    ///
    /// Returns a step containing the retained value and shortage.
    #[inline(always)]
    pub(super) const fn need_output(pending: PendingValue<Value>, additional: NonZeroUsize, available: usize) -> Self {
        Self::NeedOutput {
            pending,
            additional,
            available,
        }
    }
}
