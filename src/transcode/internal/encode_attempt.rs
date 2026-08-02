// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Borrowed state for one value-level encode attempt.

use super::super::engine::EncodeContext;

/// Engine-owned mutable state for one encode attempt.
pub(in crate::transcode) struct EncodeAttempt<'a, Value, Unit> {
    /// Input value being encoded.
    value: &'a Value,
    /// Absolute input index of `value`.
    input_index: usize,
    /// Complete mutable output slice owned by the engine.
    output: &'a mut [Unit],
    /// Absolute output index where writing begins.
    output_index: usize,
}

impl<'a, Value, Unit> EncodeAttempt<'a, Value, Unit> {
    /// Creates an engine-owned encode attempt.
    #[inline(always)]
    pub(in crate::transcode) fn new(
        value: &'a Value,
        input_index: usize,
        output: &'a mut [Unit],
        output_index: usize,
    ) -> Self {
        Self {
            value,
            input_index,
            output,
            output_index,
        }
    }

    /// Returns the input value being encoded.
    #[inline(always)]
    pub(in crate::transcode) fn value(&self) -> &Value {
        self.value
    }

    /// Returns the absolute input index.
    #[inline(always)]
    pub(in crate::transcode) const fn input_index(&self) -> usize {
        self.input_index
    }

    /// Returns writable output capacity.
    #[inline(always)]
    pub(in crate::transcode) fn available_output(&self) -> usize {
        self.output.len().saturating_sub(self.output_index)
    }

    /// Returns the read-only policy view for this attempt.
    #[inline(always)]
    pub(in crate::transcode) fn context(&self) -> EncodeContext<'_, Value> {
        EncodeContext::new(
            self.value,
            self.input_index,
            self.output_index,
            self.available_output(),
        )
    }

    /// Returns all mutable engine state for the encode operation.
    #[inline(always)]
    pub(in crate::transcode) fn into_parts(
        self,
    ) -> (&'a Value, usize, &'a mut [Unit], usize) {
        (self.value, self.input_index, self.output, self.output_index)
    }
}
