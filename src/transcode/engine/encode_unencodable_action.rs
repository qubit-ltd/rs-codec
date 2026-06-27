// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Unencodable-value actions returned by buffered encoder policy hooks.

/// Action selected after a codec reports an unencodable input value.
///
/// Normal encodable values are handled by the encode engine itself. Hook
/// implementations return this action only for values outside the codec's
/// encodable domain.
///
/// # Type Parameters
///
/// - `Value`: Logical input value type accepted by the codec.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EncodeUnencodableAction<Value> {
    /// Consume the current input value without producing output.
    Skip,

    /// Encode a replacement value and consume the current input value.
    ///
    /// The replacement must be encodable by the same codec. Returning an
    /// unencodable replacement is a hook contract violation and panics in the
    /// encode engine.
    Replace {
        /// Replacement value to encode.
        value: Value,
    },
}

impl<Value> EncodeUnencodableAction<Value> {
    /// Creates a replacement action.
    ///
    /// # Parameters
    ///
    /// - `value`: Replacement value to encode.
    ///
    /// # Returns
    ///
    /// Returns [`Self::Replace`] carrying `value`.
    #[inline(always)]
    #[must_use]
    pub const fn replace(value: Value) -> Self {
        Self::Replace { value }
    }
}
