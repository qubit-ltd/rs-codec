// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Outcome of one buffered encode value attempt.

use core::num::NonZeroUsize;

/// Outcome returned by encode-side per-value hooks.
///
/// This is deliberately smaller than [`crate::TranscodeProgress`]. It only
/// describes what happened to the current input value; the encode engine owns
/// input/output cursor updates and progress construction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EncodeOutcome {
    /// The current input value was consumed.
    Consumed {
        /// Output units written for this value.
        written: usize,
    },

    /// The current input value was not consumed because output is too small.
    NeedOutput {
        /// Total output units required from the current output cursor.
        required: NonZeroUsize,
    },
}

impl EncodeOutcome {
    /// Creates an outcome for a consumed input value.
    #[must_use]
    #[inline(always)]
    pub const fn consumed(written: usize) -> Self {
        Self::Consumed { written }
    }

    /// Creates an outcome for insufficient output capacity.
    #[must_use]
    #[inline(always)]
    pub const fn need_output(required: NonZeroUsize) -> Self {
        Self::NeedOutput { required }
    }
}
