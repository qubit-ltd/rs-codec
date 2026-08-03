// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Incomplete-decode actions returned by buffered decoder policy hooks.

use crate::Codec;

/// Incomplete-decode action for a codec-backed decode hook.
pub type DecodeIncompleteActionOf<C> =
    DecodeIncompleteAction<<C as Codec>::Value>;

/// Action selected after end-of-input leaves a codec value incomplete.
///
/// The decode engine invokes this action only after the caller has established
/// EOF. A non-rejecting action consumes every source unit still available from
/// the current decode cursor.
///
/// # Type Parameters
///
/// - `Value`: Decoded output value type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DecodeIncompleteAction<Value> {
    /// Reject the incomplete source tail.
    ///
    /// The decode engine reports the codec-domain error when available, or an
    /// [`crate::TranscodeFailure::IncompleteInput`] framework error otherwise.
    Reject,

    /// Consume the entire incomplete source tail without output.
    Skip,

    /// Consume the entire incomplete source tail and emit one replacement.
    Emit {
        /// Replacement value to write to the output buffer.
        value: Value,
    },
}
