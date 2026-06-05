// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
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
    ///
    /// When returned by a decode hook, `required_total` must be greater than
    /// the hook context's available input count. Returning a value that is
    /// already satisfied is a hook contract violation and panics in the engine.
    NeedInput {
        /// Total units required from the current value start.
        required_total: usize,
    },

    /// Consume invalid input without producing output.
    ///
    /// When returned by a decode hook, `consumed` must not exceed the hook
    /// context's available input count. Over-consuming is a hook contract
    /// violation and panics in the engine.
    Skip {
        /// Source units to consume.
        consumed: NonZeroUsize,
    },

    /// Produce one value and consume source units.
    ///
    /// When returned by a decode hook, `consumed` must not exceed the hook
    /// context's available input count. Over-consuming is a hook contract
    /// violation and panics in the engine.
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
    ///
    /// # Panics
    ///
    /// Panics when a hook returns `NeedInput` with `required_total <=
    /// available` or a consuming action whose consumed count exceeds
    /// `available`.
    #[must_use]
    #[inline]
    pub(super) fn into_step(
        self,
        input_index: usize,
        available: usize,
    ) -> DecodeStep<Value> {
        match self {
            Self::NeedInput { required_total } => DecodeStep::need_input(
                Self::missing_input(required_total, available),
                available,
            ),
            Self::Skip { consumed } => {
                DecodeStep::skipped(Self::bound_consumed(consumed, available))
            }
            Self::Emit { value, consumed } => DecodeStep::decoded(
                value,
                Self::bound_consumed(consumed, available),
                input_index,
            ),
        }
    }

    /// Returns the additional source units required by a need-input action.
    ///
    /// # Parameters
    ///
    /// - `required_total`: Total source units required from the current value
    ///   start.
    /// - `available`: Source units already visible at the current value start.
    ///
    /// # Returns
    ///
    /// Returns a non-zero additional source-unit count.
    ///
    /// # Panics
    ///
    /// Panics when `required_total <= available`.
    #[must_use]
    #[inline(always)]
    fn missing_input(required_total: usize, available: usize) -> NonZeroUsize {
        assert!(
            required_total > available,
            "DecodeAction::NeedInput required_total must exceed available input",
        );
        let additional = required_total - available;
        // SAFETY: The assertion above guarantees a positive difference.
        unsafe { NonZeroUsize::new_unchecked(additional) }
    }

    /// Validates a policy-reported consumed source-unit count against available
    /// input.
    ///
    /// # Parameters
    ///
    /// - `consumed`: Source units requested by the concrete policy.
    /// - `available`: Source units visible from the current decode cursor.
    ///
    /// # Returns
    ///
    /// Returns the validated non-zero consumed count.
    ///
    /// # Panics
    ///
    /// Panics when `available == 0` or when `consumed > available`.
    #[must_use]
    #[inline(always)]
    fn bound_consumed(
        consumed: NonZeroUsize,
        available: usize,
    ) -> NonZeroUsize {
        assert!(available > 0, "DecodeAction cannot consume empty input");
        assert!(
            consumed.get() <= available,
            "DecodeAction consumed units must not exceed available input",
        );
        consumed
    }
}
