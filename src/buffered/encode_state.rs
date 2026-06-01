/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Mutable state for one buffered encode call.

use core::num::NonZeroUsize;

use super::transcode_progress::TranscodeProgress;

/// Mutable state for one buffered encode call.
pub(super) struct EncodeState<'a, Value, Unit> {
    /// Complete input value slice visible to the encoder.
    input: &'a [Value],
    /// Absolute input value index where this call starts.
    input_start: usize,
    /// Complete output unit slice visible to the encoder.
    output: &'a mut [Unit],
    /// Absolute output unit index where this call starts.
    output_start: usize,
    /// Absolute input value index for the next encode attempt.
    input_cursor: usize,
    /// Absolute output unit index for the next write.
    output_cursor: usize,
}

impl<'a, Value, Unit> EncodeState<'a, Value, Unit> {
    /// Creates mutable encode state.
    ///
    /// # Parameters
    ///
    /// - `input`: Complete input value slice visible to the encoder.
    /// - `input_index`: Absolute input value index where encoding starts.
    /// - `output`: Complete output unit slice visible to the encoder.
    /// - `output_index`: Absolute output unit index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns initialized encode state with cursors at the requested start
    /// positions.
    pub(super) fn new(input: &'a [Value], input_index: usize, output: &'a mut [Unit], output_index: usize) -> Self {
        debug_assert!(input_index <= input.len(), "input index must be within the input slice");

        Self {
            input,
            input_start: input_index,
            output,
            output_start: output_index,
            input_cursor: input_index,
            output_cursor: output_index,
        }
    }

    /// Returns whether there is still input to encode.
    #[inline(always)]
    pub(super) fn has_input(&self) -> bool {
        self.input_cursor < self.input.len()
    }

    /// Returns the current input value and its absolute index.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `self.has_input()` returned `true`.
    #[inline(always)]
    pub(super) unsafe fn current_input_unchecked(&self) -> (&Value, usize) {
        // SAFETY: Guaranteed by the caller.
        let value = unsafe { self.input.get_unchecked(self.input_cursor) };
        (value, self.input_cursor)
    }

    /// Returns whether the output cursor is within the visible output slice.
    #[inline(always)]
    pub(super) fn output_cursor_in_bounds(&self) -> bool {
        self.output_cursor <= self.output.len()
    }

    /// Returns the number of writable output units from the current cursor.
    #[inline(always)]
    fn available_output(&self) -> usize {
        self.output.len().saturating_sub(self.output_cursor)
    }

    /// Returns whether the current output has the requested capacity.
    #[inline(always)]
    pub(super) fn has_output_for(&self, required: usize) -> bool {
        self.available_output() >= required
    }

    /// Returns write arguments for the current input value and output cursor.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `self.has_input()` returned `true`.
    #[inline(always)]
    pub(super) unsafe fn write_parts_unchecked(&mut self) -> (&Value, usize, &mut [Unit], usize) {
        // SAFETY: Guaranteed by the caller.
        let value = unsafe { self.input.get_unchecked(self.input_cursor) };
        (value, self.input_cursor, &mut *self.output, self.output_cursor)
    }

    /// Accepts a completed one-value write and advances both cursors.
    #[inline(always)]
    pub(super) fn accept_written_value(&mut self, written: usize) {
        debug_assert!(
            written <= self.available_output(),
            "encode step wrote beyond available output",
        );
        self.input_cursor += 1;
        self.output_cursor += written;
    }

    /// Returns completed progress for the current cursors.
    pub(super) fn complete_progress(&self) -> TranscodeProgress {
        TranscodeProgress::complete(
            self.input_cursor - self.input_start,
            self.output_cursor - self.output_start,
        )
    }

    /// Returns progress for a missing output capacity bound.
    pub(super) fn need_output_progress(&self, required: usize) -> TranscodeProgress {
        let available = self.available_output();
        debug_assert!(required > available, "need-output progress requires missing capacity");
        let additional = NonZeroUsize::new(required - available).expect("missing output is non-zero");
        TranscodeProgress::need_output(
            self.output_cursor,
            additional,
            available,
            self.input_cursor - self.input_start,
            self.output_cursor - self.output_start,
        )
    }
}
