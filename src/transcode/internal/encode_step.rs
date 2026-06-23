// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal encode-step result used by buffered encoders.

use core::num::NonZeroUsize;

/// Result of one prepared value encode attempt.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::transcode) enum EncodeStep {
    /// The value was fully written.
    Written {
        /// Output units written by the encode hook.
        written: usize,
    },
    /// The value could not be written because output capacity is insufficient.
    NeedOutput {
        /// Total output units required from the current output position.
        required: NonZeroUsize,
        /// Output units available at the stop boundary.
        available: usize,
    },
}

impl EncodeStep {
    /// Creates a successful encode step.
    ///
    /// # Parameters
    ///
    /// - `written`: Output units written by the encode hook.
    ///
    /// # Returns
    ///
    /// Returns a step that consumed one logical input value.
    #[inline(always)]
    pub(in crate::transcode) const fn written(written: usize) -> Self {
        Self::Written { written }
    }

    /// Creates an output-starved encode step.
    ///
    /// # Parameters
    ///
    /// - `required`: Output units required by the prepared encode plan.
    /// - `available`: Output units currently writable at the output cursor.
    ///
    /// # Returns
    ///
    /// Returns a step describing the required output capacity.
    #[inline(always)]
    pub(in crate::transcode) const fn need_output(
        required: NonZeroUsize,
        available: usize,
    ) -> Self {
        Self::NeedOutput {
            required,
            available,
        }
    }
}
