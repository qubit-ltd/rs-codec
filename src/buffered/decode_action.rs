/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Decode actions returned by buffered decoder policy hooks.

use core::num::NonZeroUsize;

use super::decode_step::DecodeStep;

/// Action selected after a codec decode attempt fails during `transcode`.
///
/// # Type Parameters
///
/// - `Value`: Decoded output value type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DecodeAction<Value> {
    /// More source units are needed before a value can be produced.
    NeedInput {
        /// Total units required from the current value start.
        required_total: usize,
    },

    /// Consume invalid input without producing output.
    Skip {
        /// Source units to consume.
        consumed: NonZeroUsize,
    },

    /// Produce one value and consume source units.
    Emit {
        /// Value to write to the output buffer.
        value: Value,
        /// Source units to consume.
        consumed: NonZeroUsize,
    },
}

impl<Value> DecodeAction<Value> {
    /// Converts this policy action into the normalized internal decode attempt.
    ///
    /// # Parameters
    ///
    /// - `input_index`: Absolute source index where the failed decode started.
    /// - `available`: Source units visible from `input_index`.
    ///
    /// # Returns
    ///
    /// Returns the internal decode attempt consumed by buffered decode loops.
    #[must_use]
    pub(super) fn into_step(self, input_index: usize, available: usize) -> DecodeStep<Value> {
        match self {
            Self::NeedInput { required_total } => {
                DecodeStep::need_input(Self::missing_input(required_total, available), available)
            }
            Self::Skip { consumed } => DecodeStep::skipped(Self::bound_consumed(consumed, available)),
            Self::Emit { value, consumed } => {
                DecodeStep::decoded(value, Self::bound_consumed(consumed, available), input_index)
            }
        }
    }

    /// Returns the additional source units required by a need-input action.
    ///
    /// # Parameters
    ///
    /// - `required_total`: Total source units required from the current value start.
    /// - `available`: Source units already visible at the current value start.
    ///
    /// # Returns
    ///
    /// Returns a non-zero additional source-unit count.
    #[must_use]
    #[inline(always)]
    fn missing_input(required_total: usize, available: usize) -> NonZeroUsize {
        let additional = required_total.saturating_sub(available).max(1);
        // SAFETY: The count is clamped to at least one.
        unsafe { NonZeroUsize::new_unchecked(additional) }
    }

    /// Bounds a policy-reported consumed source-unit count to available input.
    ///
    /// # Parameters
    ///
    /// - `consumed`: Source units requested by the concrete policy.
    /// - `available`: Source units visible from the current decode cursor.
    ///
    /// # Returns
    ///
    /// Returns a non-zero count clamped to the currently available input.
    #[must_use]
    #[inline(always)]
    fn bound_consumed(consumed: NonZeroUsize, available: usize) -> NonZeroUsize {
        debug_assert!(available > 0, "decode action cannot consume empty input");
        let consumed = consumed.get().min(available).max(1);
        // SAFETY: The normalized count is clamped to at least one.
        unsafe { NonZeroUsize::new_unchecked(consumed) }
    }
}
