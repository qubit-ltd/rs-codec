// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal decode-policy action after reject handling.

use core::num::NonZeroUsize;

/// Decode-policy action ready to be applied to buffered state.
pub(in crate::transcode) enum AppliedDecodeInvalidAction<Value> {
    /// Skips invalid source units without emitting a value.
    Skip {
        /// Non-zero invalid source units to consume.
        consumed: NonZeroUsize,
    },
    /// Emits a replacement value while consuming invalid source units.
    Emit {
        /// Replacement value emitted by policy.
        value: Value,
        /// Non-zero invalid source units to consume.
        consumed: NonZeroUsize,
    },
}
