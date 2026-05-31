/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Internal decode-step result used by buffered converters.

use core::num::NonZeroUsize;

/// Result of one decode attempt in the converter loop.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum DecodeAttempt<Value> {
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
        /// Additional source units required to continue.
        additional: NonZeroUsize,
        /// Source units available at the incomplete boundary.
        available: usize,
    },
}
