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
///
/// Borrows the input and exclusively borrows output for `'a`; creating this
/// state does not encode a value or validate the supplied indices.
///
/// # Type Parameters
///
/// - `Value`: Logical input value type.
/// - `Unit`: Encoded output unit type.
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
    ///
    /// # Parameters
    ///
    /// - `value`: Input borrowed for the attempt lifetime `'a`.
    /// - `input_index`: Absolute position of this input value.
    /// - `output`: Complete output slice exclusively borrowed for `'a`.
    /// - `output_index`: First writable offset in `output`, retained unchecked.
    ///
    /// # Returns
    ///
    /// A borrowed state holder without encoding or allocating output.
    #[must_use]
    #[inline]
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
    ///
    /// # Returns
    ///
    /// The original input borrowed through this attempt, without cloning it.
    #[must_use]
    #[inline]
    pub(in crate::transcode) fn value(&self) -> &Value {
        self.value
    }

    /// Returns the absolute input index.
    ///
    /// # Returns
    ///
    /// The input position supplied to the constructor, unchanged by this
    /// holder.
    #[must_use]
    #[inline]
    pub(in crate::transcode) const fn input_index(&self) -> usize {
        self.input_index
    }

    /// Returns writable output capacity.
    ///
    /// # Returns
    ///
    /// The units remaining after the output offset, or zero if it exceeds the
    /// output length. This query does not modify the output slice.
    #[must_use]
    #[inline]
    pub(in crate::transcode) fn available_output(&self) -> usize {
        self.output.len().saturating_sub(self.output_index)
    }

    /// Returns the read-only policy view for this attempt.
    ///
    /// # Returns
    ///
    /// A context borrowing the input while this attempt is borrowed, carrying
    /// both absolute indices and the current saturating output capacity.
    #[must_use]
    #[inline]
    pub(in crate::transcode) fn context(&self) -> EncodeContext<'_, Value> {
        EncodeContext::new(self.value, self.input_index, self.output_index, self.available_output())
    }

    /// Returns all mutable engine state for the encode operation.
    ///
    /// # Returns
    ///
    /// The original input reference, input index, exclusive output slice and
    /// output index, in that order. Consumes this holder and transfers its
    /// original `'a` borrows without encoding or allocating.
    #[must_use]
    #[inline]
    pub(in crate::transcode) fn into_parts(self) -> (&'a Value, usize, &'a mut [Unit], usize) {
        (self.value, self.input_index, self.output, self.output_index)
    }
}
