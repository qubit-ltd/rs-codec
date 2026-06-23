// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Encode context for one buffered encode attempt.

/// Context for one encode attempt inside a buffered encoder engine.
///
/// The context carries the current input value and output cursor. The hook
/// decides whether the value can be consumed with the visible output capacity
/// and reports that decision through [`crate::EncodeOutcome`].
///
/// # Type Parameters
///
/// - `Value`: Logical input value type.
/// - `Unit`: Encoded output unit type.
#[derive(Debug)]
pub struct EncodeContext<'a, Value, Unit> {
    /// Input value being encoded.
    pub input_value: &'a Value,

    /// Absolute input index of [`input_value`](Self::input_value).
    pub input_index: usize,

    /// Complete output unit slice visible to the encoder.
    pub output: &'a mut [Unit],

    /// Start position in [`output`](Self::output) where writing begins.
    pub output_index: usize,
}

impl<Value, Unit> EncodeContext<'_, Value, Unit> {
    /// Returns writable output units from the current output index.
    ///
    /// # Returns
    ///
    /// Returns output capacity visible to this encode attempt.
    #[must_use]
    #[inline(always)]
    pub fn available_output(&self) -> usize {
        self.output.len().saturating_sub(self.output_index)
    }
}
