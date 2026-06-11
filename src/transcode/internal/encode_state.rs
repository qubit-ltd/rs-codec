// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Mutable state for one buffered encode call.

use core::num::NonZeroUsize;

use super::super::{
    encode_context::EncodeContext,
    transcode_progress::TranscodeProgress,
};

/// Mutable state for one buffered encode call.
pub(in crate::transcode) struct EncodeState<'a, Value, Unit> {
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
    #[inline(always)]
    pub(in crate::transcode) fn new(
        input: &'a [Value],
        input_index: usize,
        output: &'a mut [Unit],
        output_index: usize,
    ) -> Self {
        debug_assert!(
            input_index <= input.len(),
            "input index must be within the input slice"
        );

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
    ///
    /// # Returns
    ///
    /// Returns `true` when more input values remain.
    #[inline(always)]
    pub(in crate::transcode) fn has_input(&self) -> bool {
        self.input_cursor < self.input.len()
    }

    /// Returns an encode context at the current cursors.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `self.has_input()` returned `true`.
    #[inline(always)]
    pub(in crate::transcode) unsafe fn context_unchecked(
        &mut self,
    ) -> EncodeContext<'_, Value, Unit> {
        // SAFETY: Guaranteed by the caller.
        let value = unsafe { self.input.get_unchecked(self.input_cursor) };
        EncodeContext {
            input_value: value,
            input_index: self.input_cursor,
            output: &mut *self.output,
            output_index: self.output_cursor,
        }
    }

    /// Returns the number of writable output units from the current cursor.
    ///
    /// # Returns
    ///
    /// Returns writable output capacity from the current output cursor.
    #[inline(always)]
    fn available_output(&self) -> usize {
        self.output.len().saturating_sub(self.output_cursor)
    }

    /// Accepts a completed one-value write and advances both cursors.
    ///
    /// # Parameters
    ///
    /// - `written`: Output units written by the last encode call.
    ///
    /// # Returns
    ///
    /// Returns unit `()`, while advancing `input_cursor` and `output_cursor`.
    #[inline(always)]
    pub(in crate::transcode) fn accept_written_value(
        &mut self,
        written: usize,
    ) {
        assert!(
            written <= self.available_output(),
            "encode step wrote beyond available output",
        );
        self.input_cursor += 1;
        self.output_cursor += written;
    }

    /// Returns completed progress for the current cursors.
    ///
    /// # Returns
    ///
    /// Returns a completed [`TranscodeProgress`] with consumed input and output
    /// counters.
    #[inline(always)]
    pub(in crate::transcode) fn complete_progress(&self) -> TranscodeProgress {
        TranscodeProgress::complete(
            self.input_cursor - self.input_start,
            self.output_cursor - self.output_start,
        )
    }

    /// Returns progress for a known missing output capacity.
    ///
    /// # Parameters
    ///
    /// - `additional`: Additional output units required before encoding can
    ///   continue.
    /// - `available`: Output units currently writable at the stop boundary.
    ///
    /// # Returns
    ///
    /// Returns [`TranscodeProgress::need_output`] with missing-capacity
    /// counters.
    #[inline(always)]
    pub(in crate::transcode) fn need_output_progress_with(
        &self,
        additional: NonZeroUsize,
        available: usize,
    ) -> TranscodeProgress {
        TranscodeProgress::need_output(
            self.output_cursor,
            additional,
            available,
            self.input_cursor - self.input_start,
            self.output_cursor - self.output_start,
        )
    }
}
