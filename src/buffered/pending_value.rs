// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Pending decoded value retained by buffered converters.

/// Decoded value retained after source input has been consumed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct PendingValue<Value> {
    /// Decoded logical value.
    value: Value,
    /// Source input index that produced this value.
    input_index: usize,
}

impl<Value> PendingValue<Value> {
    /// Creates a retained decoded value.
    ///
    /// # Parameters
    ///
    /// - `value`: Decoded logical value.
    /// - `input_index`: Source input index that produced the value.
    ///
    /// # Returns
    ///
    /// Returns pending-value state owned by the converter engine.
    #[inline(always)]
    pub(super) const fn new(value: Value, input_index: usize) -> Self {
        Self { value, input_index }
    }

    /// Returns the source input index that produced this value.
    ///
    /// # Returns
    ///
    /// Returns the absolute source input index used for downstream encode
    /// errors.
    #[must_use]
    #[inline(always)]
    pub(super) const fn input_index(&self) -> usize {
        self.input_index
    }

    /// Returns the decoded value.
    ///
    /// # Returns
    ///
    /// Returns the retained decoded value by shared reference.
    #[must_use]
    #[inline(always)]
    pub(super) const fn value(&self) -> &Value {
        &self.value
    }
}
