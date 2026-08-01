// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Read-only policy context for one buffered encode attempt.

/// Read-only context supplied to an encode policy hook.
///
/// The engine retains ownership of the mutable output slice. Hooks can inspect
/// the current value and cursor/capacity metadata, then return an
/// [`crate::engine::EncodeUnencodableAction`] without being able to mutate
/// output behind the engine's progress accounting.
///
/// # Type Parameters
///
/// - `Value`: Logical input value type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EncodeContext<'a, Value> {
    input_value: &'a Value,
    input_index: usize,
    output_index: usize,
    available_output: usize,
}

impl<'a, Value> EncodeContext<'a, Value> {
    /// Creates an encode policy context.
    ///
    /// # Parameters
    ///
    /// - `input_value`: Borrowed input value being encoded.
    /// - `input_index`: Absolute input index of `input_value`.
    /// - `output_index`: Absolute output index where writing begins.
    /// - `available_output`: Output units writable from `output_index`.
    ///
    /// # Returns
    ///
    /// Returns a read-only encode context.
    #[inline(always)]
    #[must_use]
    pub const fn new(
        input_value: &'a Value,
        input_index: usize,
        output_index: usize,
        available_output: usize,
    ) -> Self {
        Self {
            input_value,
            input_index,
            output_index,
            available_output,
        }
    }

    /// Returns the input value being encoded.
    #[inline(always)]
    #[must_use]
    pub const fn input_value(&self) -> &Value {
        self.input_value
    }

    /// Returns the absolute input index of the current value.
    #[inline(always)]
    #[must_use]
    pub const fn input_index(&self) -> usize {
        self.input_index
    }

    /// Returns the absolute output index where writing begins.
    #[inline(always)]
    #[must_use]
    pub const fn output_index(&self) -> usize {
        self.output_index
    }

    /// Returns output capacity visible to this encode attempt.
    #[inline(always)]
    #[must_use]
    pub const fn available_output(&self) -> usize {
        self.available_output
    }
}
