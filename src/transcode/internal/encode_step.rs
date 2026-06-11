// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal encode-step result used by buffered encoders.

use core::num::NonZeroUsize;

use super::super::transcode_progress::TranscodeProgress;
use super::encode_state::EncodeState;

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
        /// Additional output units required to continue.
        additional: NonZeroUsize,
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
    /// Returns a step describing the missing output capacity.
    #[inline(always)]
    pub(in crate::transcode) fn need_output(
        required: usize,
        available: usize,
    ) -> Self {
        let additional = NonZeroUsize::new(required - available)
            .expect("missing output is non-zero");
        Self::NeedOutput {
            additional,
            available,
        }
    }

    /// Applies this step to the current encode state.
    ///
    /// # Parameters
    ///
    /// - `state`: Active encode call state.
    ///
    /// # Returns
    ///
    /// Returns stop progress when more output is required, otherwise `None`.
    #[inline]
    pub(in crate::transcode) fn apply_to_state<Value, Unit>(
        self,
        state: &mut EncodeState<'_, Value, Unit>,
    ) -> Option<TranscodeProgress> {
        match self {
            Self::Written { written } => {
                state.accept_written_value(written);
                None
            }
            Self::NeedOutput {
                additional,
                available,
            } => Some(state.need_output_progress_with(additional, available)),
        }
    }
}
