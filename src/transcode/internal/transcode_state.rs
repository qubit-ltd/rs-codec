// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared input/output state for one transcode call.

use core::num::NonZeroUsize;

use super::super::transcode_progress::TranscodeProgress;

/// Shared input/output state for one transcode call.
///
/// `TranscodeState` owns the input/output slices visible to a single buffered
/// transcode call and tracks absolute input/output cursors. Concrete
/// encode/decode/convert states wrap it and keep domain-specific operations in
/// their own types.
pub(in crate::transcode) struct TranscodeState<'a, Input, Output> {
    /// Complete input slice visible to the current call.
    input: &'a [Input],
    /// Complete output slice visible to the current call.
    output: &'a mut [Output],
    /// Absolute input index where this call starts.
    input_start: usize,
    /// Absolute output index where this call starts.
    output_start: usize,
    /// Absolute input index for the next operation.
    input_cursor: usize,
    /// Absolute output index for the next write.
    output_cursor: usize,
}

impl<'a, Input, Output> TranscodeState<'a, Input, Output> {
    /// Creates shared transcode state.
    ///
    /// # Parameters
    ///
    /// - `input`: Complete input slice visible to the current call.
    /// - `input_index`: Absolute input index where this call starts.
    /// - `output`: Complete output slice visible to the current call.
    /// - `output_index`: Absolute output index where this call starts.
    ///
    /// # Returns
    ///
    /// Returns initialized state with both cursors at their call starts.
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) fn new(
        input: &'a [Input],
        input_index: usize,
        output: &'a mut [Output],
        output_index: usize,
    ) -> Self {
        debug_assert!(
            input_index <= input.len(),
            "input index must be within the input slice",
        );

        Self {
            input,
            output,
            input_start: input_index,
            output_start: output_index,
            input_cursor: input_index,
            output_cursor: output_index,
        }
    }

    /// Returns the complete input slice.
    ///
    /// # Returns
    ///
    /// Returns the full input slice visible to this call.
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) fn input(&self) -> &[Input] {
        self.input
    }

    /// Returns the complete output slice mutably.
    ///
    /// # Returns
    ///
    /// Returns the full mutable output slice visible to this call.
    #[inline(always)]
    pub(in crate::transcode) fn output_mut(&mut self) -> &mut [Output] {
        self.output
    }

    /// Returns input and output slices together.
    ///
    /// # Returns
    ///
    /// Returns the immutable input slice and mutable output slice. This helper
    /// lets callers borrow the two disjoint fields at the same time.
    #[inline(always)]
    pub(in crate::transcode) fn input_output_mut(
        &mut self,
    ) -> (&[Input], &mut [Output]) {
        (self.input, self.output)
    }

    /// Returns the absolute input start index.
    ///
    /// # Returns
    ///
    /// Returns the input index recorded when this call began.
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) const fn input_start(&self) -> usize {
        self.input_start
    }

    /// Returns the absolute output start index.
    ///
    /// # Returns
    ///
    /// Returns the output index recorded when this call began.
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) const fn output_start(&self) -> usize {
        self.output_start
    }

    /// Returns the current absolute input cursor.
    ///
    /// # Returns
    ///
    /// Returns the absolute input index for the next read or consume operation.
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) const fn input_cursor(&self) -> usize {
        self.input_cursor
    }

    /// Returns the current absolute output cursor.
    ///
    /// # Returns
    ///
    /// Returns the absolute output index for the next write operation.
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) const fn output_cursor(&self) -> usize {
        self.output_cursor
    }

    /// Returns whether input remains.
    ///
    /// # Returns
    ///
    /// Returns `true` when the input cursor has not reached the input end.
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) fn has_input(&self) -> bool {
        self.input_cursor < self.input.len()
    }

    /// Returns whether output has no room for another value.
    ///
    /// # Returns
    ///
    /// Returns `true` when the output cursor is at the output end.
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) fn needs_output(&self) -> bool {
        self.output_cursor == self.output.len()
    }

    /// Returns input units visible from the current input cursor.
    ///
    /// # Returns
    ///
    /// Returns remaining input units visible from `input_cursor`.
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) fn available_input(&self) -> usize {
        self.input.len() - self.input_cursor
    }

    /// Returns output units writable from the current output cursor.
    ///
    /// # Returns
    ///
    /// Returns remaining output capacity from `output_cursor`.
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) fn available_output(&self) -> usize {
        self.output.len().saturating_sub(self.output_cursor)
    }

    /// Returns input units consumed since this call started.
    ///
    /// # Returns
    ///
    /// Returns consumed input units relative to `input_start`.
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) const fn read(&self) -> usize {
        self.input_cursor - self.input_start
    }

    /// Returns output units written since this call started.
    ///
    /// # Returns
    ///
    /// Returns written output units relative to `output_start`.
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) const fn written(&self) -> usize {
        self.output_cursor - self.output_start
    }

    /// Advances the input cursor.
    ///
    /// # Parameters
    ///
    /// - `read`: Number of input units consumed by the last operation.
    #[inline(always)]
    pub(in crate::transcode) fn advance_input(&mut self, read: usize) {
        self.input_cursor += read;
    }

    /// Advances the output cursor.
    ///
    /// # Parameters
    ///
    /// - `written`: Number of output units written by the last operation.
    #[inline(always)]
    pub(in crate::transcode) fn advance_output(&mut self, written: usize) {
        self.output_cursor += written;
    }

    /// Advances both cursors.
    ///
    /// # Parameters
    ///
    /// - `read`: Number of input units consumed by the last operation.
    /// - `written`: Number of output units written by the last operation.
    #[inline(always)]
    pub(in crate::transcode) fn advance(
        &mut self,
        read: usize,
        written: usize,
    ) {
        self.advance_input(read);
        self.advance_output(written);
    }

    /// Returns completed progress for the current cursors.
    ///
    /// # Returns
    ///
    /// Returns completed progress with consumed input and written output
    /// counters.
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) fn complete_progress(&self) -> TranscodeProgress {
        TranscodeProgress::complete(self.read(), self.written())
    }

    /// Returns progress for missing input.
    ///
    /// # Parameters
    ///
    /// - `required`: Total input units required from the current input
    ///   position.
    /// - `available`: Input units currently available at the stop boundary.
    ///
    /// # Returns
    ///
    /// Returns [`TranscodeProgress`] with need-input status.
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) fn need_input_progress(
        &self,
        required: NonZeroUsize,
        available: usize,
    ) -> TranscodeProgress {
        TranscodeProgress::need_input(
            self.input_cursor,
            required,
            available,
            self.read(),
            self.written(),
        )
    }

    /// Returns progress for missing output.
    ///
    /// # Parameters
    ///
    /// - `required`: Total output units required from the current output
    ///   position.
    /// - `available`: Output units currently available at the stop boundary.
    ///
    /// # Returns
    ///
    /// Returns [`TranscodeProgress`] with need-output status.
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) fn need_output_progress(
        &self,
        required: NonZeroUsize,
        available: usize,
    ) -> TranscodeProgress {
        TranscodeProgress::need_output(
            self.output_cursor,
            required,
            available,
            self.read(),
            self.written(),
        )
    }
}
