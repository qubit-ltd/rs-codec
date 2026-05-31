/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Internal encode-step result used by buffered converters.

use core::num::NonZeroUsize;

use super::pending_value::PendingValue;

/// Result of one encode attempt in the converter loop.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum EncodeAttempt<Value> {
    /// The value was fully written.
    Written {
        /// Number of target units written.
        written: usize,
    },
    /// The value could not be written because target output is too small.
    NeedOutput {
        /// Retained value to write later.
        pending: PendingValue<Value>,
        /// Additional target units required to continue.
        additional: NonZeroUsize,
        /// Target units available at the output boundary.
        available: usize,
    },
}
