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
/// In a codec-backed converter, values emitted by source decode reset or
/// finish have no source-unit position and use placeholder index 0. Ordinary
/// decoded values use their source index relative to the current input slice.
///
/// # Type Parameters
///
/// - `Value`: Logical input value type.
///
/// # Examples
///
/// ```
/// use qubit_codec::engine::EncodeContext;
///
/// let value = 7_u8;
/// let context = EncodeContext::new(&value, 0, 0, 1);
/// assert_eq!(context.input_value(), &7);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EncodeContext<'a, Value> {
    /// Value borrowed for the lifetime of this policy context.
    input_value: &'a Value,
    /// Input position of the borrowed value, or placeholder 0 for a converter
    /// lifecycle-generated value.
    input_index: usize,
    /// Absolute output cursor at the start of the encode attempt.
    output_index: usize,
    /// Number of writable output units remaining at the cursor.
    available_output: usize,
}

impl<'a, Value> EncodeContext<'a, Value> {
    /// Creates an encode policy context.
    ///
    /// # Parameters
    ///
    /// - `input_value`: Borrowed input value being encoded.
    /// - `input_index`: Input index of `input_value`; converter lifecycle-
    ///   generated values use placeholder 0 because they have no source-unit
    ///   position.
    /// - `output_index`: Absolute output index where writing begins.
    /// - `available_output`: Output units writable from `output_index`.
    ///
    /// # Returns
    ///
    /// Returns a read-only encode context.
    #[must_use]
    #[inline]
    pub const fn new(input_value: &'a Value, input_index: usize, output_index: usize, available_output: usize) -> Self {
        Self {
            input_value,
            input_index,
            output_index,
            available_output,
        }
    }

    /// Returns the input value being encoded.
    ///
    /// # Returns
    ///
    /// The input value borrowed through this context without allocation.
    #[must_use = "the borrowed input value should be inspected"]
    #[inline]
    pub const fn input_value(&self) -> &Value {
        self.input_value
    }

    /// Returns the input index of the current value.
    ///
    /// In a codec-backed converter, lifecycle-generated values use placeholder
    /// index 0 because they have no source-unit position. Ordinary decoded
    /// values use an index relative to the current input slice.
    ///
    /// # Returns
    ///
    /// The value's input index, including placeholder 0 for converter
    /// lifecycle-generated values.
    #[must_use]
    #[inline]
    pub const fn input_index(&self) -> usize {
        self.input_index
    }

    /// Returns the absolute output index where writing begins.
    ///
    /// # Returns
    ///
    /// The starting output cursor for this encode attempt.
    #[must_use]
    #[inline]
    pub const fn output_index(&self) -> usize {
        self.output_index
    }

    /// Returns output capacity visible to this encode attempt.
    ///
    /// # Returns
    ///
    /// The number of output units available from the starting cursor.
    #[must_use]
    #[inline]
    pub const fn available_output(&self) -> usize {
        self.available_output
    }
}
