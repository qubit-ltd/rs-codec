// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Outcome of one buffered decode attempt.

use core::num::NonZeroUsize;

/// Outcome produced by a decode-side engine for one source attempt.
///
/// This type describes what happened to source input after the engine has
/// decoded, applied invalid-input hooks, and optionally delivered a logical
/// value to its caller-provided consumer. It does not describe target output
/// capacity for conversion pipelines; downstream encode backpressure is
/// reported separately by the encode side.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DecodeOutcome {
    /// A logical value was emitted to the decode consumer.
    Emitted {
        /// Source units consumed for the emitted value.
        read: NonZeroUsize,
        /// Logical values emitted by this decode attempt.
        emitted: NonZeroUsize,
    },

    /// Source input was consumed without emitting a value.
    Skipped {
        /// Source units consumed by the skip.
        read: NonZeroUsize,
    },

    /// More source input is required before decoding can continue.
    NeedInput {
        /// Total source units required from the current input position.
        required: NonZeroUsize,
    },
}

impl DecodeOutcome {
    /// Creates an emitted-value outcome.
    #[inline(always)]
    #[must_use]
    pub const fn emitted(read: NonZeroUsize, emitted: NonZeroUsize) -> Self {
        Self::Emitted { read, emitted }
    }

    /// Creates a skipped-input outcome.
    #[inline(always)]
    #[must_use]
    pub const fn skipped(read: NonZeroUsize) -> Self {
        Self::Skipped { read }
    }

    /// Creates a missing-input outcome.
    #[inline(always)]
    #[must_use]
    pub const fn need_input(required: NonZeroUsize) -> Self {
        Self::NeedInput { required }
    }
}
