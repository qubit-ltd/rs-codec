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
    input_value: &'a Value,
    input_index: usize,
    output: &'a mut [Unit],
    output_index: usize,
}

impl<'a, Value, Unit> EncodeContext<'a, Value, Unit> {
    /// Creates an encode context.
    ///
    /// # Parameters
    ///
    /// - `input_value`: Borrowed input value being encoded.
    /// - `input_index`: Absolute input index of `input_value`.
    /// - `output`: Complete output unit slice visible to the encoder.
    /// - `output_index`: Start position in `output` where writing begins.
    ///
    /// # Returns
    ///
    /// Returns an encode context.
    #[inline(always)]
    #[must_use]
    pub fn new(
        input_value: &'a Value,
        input_index: usize,
        output: &'a mut [Unit],
        output_index: usize,
    ) -> Self {
        Self {
            input_value,
            input_index,
            output,
            output_index,
        }
    }

    /// Returns the input value being encoded.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the current input value.
    #[inline(always)]
    #[must_use]
    pub fn input_value(&self) -> &Value {
        self.input_value
    }

    /// Returns the absolute input index of the current value.
    ///
    /// # Returns
    ///
    /// Returns the absolute input index.
    #[inline(always)]
    #[must_use]
    pub fn input_index(&self) -> usize {
        self.input_index
    }

    /// Returns the complete output unit slice visible to the encoder.
    ///
    /// # Returns
    ///
    /// Returns the output slice.
    #[inline(always)]
    #[must_use]
    pub fn output(&mut self) -> &mut [Unit] {
        self.output
    }

    /// Returns the start position in the output slice where writing begins.
    ///
    /// # Returns
    ///
    /// Returns the absolute output index.
    #[inline(always)]
    #[must_use]
    pub fn output_index(&self) -> usize {
        self.output_index
    }

    /// Returns writable output units from the current output index.
    ///
    /// # Returns
    ///
    /// Returns output capacity visible to this encode attempt.
    #[inline(always)]
    #[must_use]
    pub fn available_output(&self) -> usize {
        self.output.len().saturating_sub(self.output_index)
    }

    /// Consumes the context and returns all parts.
    ///
    /// Use this when you need simultaneous access to the input value reference
    /// and the mutable output slice, since Rust's borrow checker disallows
    /// taking `&self` and `&mut self` in the same expression.
    ///
    /// # Returns
    ///
    /// Returns `(input_value, input_index, output, output_index)`.
    #[inline(always)]
    #[must_use]
    pub fn into_parts(self) -> (&'a Value, usize, &'a mut [Unit], usize) {
        (
            self.input_value,
            self.input_index,
            self.output,
            self.output_index,
        )
    }
}
