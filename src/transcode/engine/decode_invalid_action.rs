// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Invalid-decode actions returned by buffered decoder policy hooks.

use core::num::NonZeroUsize;

/// Action selected after a codec reports invalid encoded input.
///
/// Incomplete input is not a policy action. Codecs report it with
/// [`crate::DecodeFailure::Incomplete`], and the decode engine converts it
/// directly into [`crate::TranscodeStatus::NeedInput`].
///
/// # Type Parameters
///
/// - `Value`: Decoded output value type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DecodeInvalidAction<Value> {
    /// Reject the current invalid input.
    ///
    /// The decode engine reports the original codec decode error as
    /// [`crate::TranscodeError::Domain`] at the current input index.
    Reject,

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
    #[inline(always)]
    #[must_use]
    pub(super) fn bound_consumed(consumed: NonZeroUsize, available: usize) -> NonZeroUsize {
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
