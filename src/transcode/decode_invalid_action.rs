// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Invalid-decode actions returned by buffered decoder policy hooks.

use core::num::NonZeroUsize;

use super::internal::decode_step::DecodeStep;

/// Action selected after a codec reports invalid encoded input.
///
/// Incomplete input is not a policy action. Codecs report it with
/// [`crate::CodecDecodeFailure::Incomplete`], and the decode engine converts it
/// directly into [`crate::TranscodeStatus::NeedInput`].
///
/// # Type Parameters
///
/// - `Value`: Decoded output value type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DecodeInvalidAction<Value> {
    /// Consume invalid input without producing output.
    ///
    /// `consumed` must not exceed the hook context's available input count.
    /// Over-consuming is a hook contract violation and panics in the engine.
    Skip {
        /// Source units to consume.
        consumed: NonZeroUsize,
    },

    /// Produce one replacement value and consume source units.
    ///
    /// `consumed` must not exceed the hook context's available input count.
    /// Over-consuming is a hook contract violation and panics in the engine.
    Emit {
        /// Value to write to the output buffer.
        value: Value,
        /// Source units to consume.
        consumed: NonZeroUsize,
    },
}

impl<Value> DecodeInvalidAction<Value> {
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
    /// Panics when a consuming action exceeds `available`.
    #[must_use]
    #[inline]
    pub(super) fn into_step(self, input_index: usize, available: usize) -> DecodeStep<Value> {
        match self {
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
    fn bound_consumed(consumed: NonZeroUsize, available: usize) -> NonZeroUsize {
        assert!(
            available > 0,
            "DecodeInvalidAction cannot consume empty input",
        );
        assert!(
            consumed.get() <= available,
            "DecodeInvalidAction consumed units must not exceed available input",
        );
        consumed
    }
}
