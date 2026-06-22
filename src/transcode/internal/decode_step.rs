// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal decode-step result used by buffered converters.

use core::num::NonZeroUsize;

/// Result of one decode step in the converter loop.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::transcode) enum DecodeStep<Value> {
    /// A source value was decoded or emitted by policy.
    Decoded {
        /// Decoded logical value.
        value: Value,
        /// Number of consumed source units.
        consumed: NonZeroUsize,
        /// Source input index used for downstream encode context.
        input_index: usize,
    },
    /// Source input was consumed without producing a value.
    Skipped {
        /// Number of consumed source units.
        consumed: NonZeroUsize,
    },
    /// More source input is required before decoding can continue.
    NeedInput {
        /// Total source units required from the current input position.
        required: NonZeroUsize,
        /// Source units available at the incomplete boundary.
        available: usize,
    },
}

impl<Value> DecodeStep<Value> {
    /// Creates a decoded-value step.
    ///
    /// # Parameters
    ///
    /// - `value`: Decoded logical value.
    /// - `consumed`: Number of consumed source units.
    /// - `input_index`: Source index used for downstream encode context.
    ///
    /// # Returns
    ///
    /// Returns a decoded step.
    #[inline(always)]
    pub(in crate::transcode) const fn decoded(
        value: Value,
        consumed: NonZeroUsize,
        input_index: usize,
    ) -> Self {
        Self::Decoded {
            value,
            consumed,
            input_index,
        }
    }

    /// Creates a skipped-input step.
    ///
    /// # Parameters
    ///
    /// - `consumed`: Number of consumed source units.
    ///
    /// # Returns
    ///
    /// Returns a skipped step.
    #[inline(always)]
    pub(in crate::transcode) const fn skipped(consumed: NonZeroUsize) -> Self {
        Self::Skipped { consumed }
    }

    /// Creates a missing-input step.
    ///
    /// # Parameters
    ///
    /// - `required`: Total source units required from the current input
    ///   position.
    /// - `available`: Source units currently available.
    ///
    /// # Returns
    ///
    /// Returns a need-input step.
    #[inline(always)]
    pub(in crate::transcode) const fn need_input(required: NonZeroUsize, available: usize) -> Self {
        Self::NeedInput {
            required,
            available,
        }
    }
}
