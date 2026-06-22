// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Mutable state for one buffered encode call.

use core::num::NonZeroUsize;

use super::super::{encode_context::EncodeContext, transcode_progress::TranscodeProgress};
use super::cursor_state::CursorState;
use super::encode_step::EncodeStep;

/// Mutable state for one buffered encode call.
pub(in crate::transcode) struct EncodeState<'a, Value, Unit> {
    /// Complete input value slice visible to the encoder.
    input: &'a [Value],
    /// Complete output unit slice visible to the encoder.
    output: &'a mut [Unit],
    /// Shared absolute input/output cursors.
    cursor: CursorState,
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
            output,
            cursor: CursorState::new(input_index, output_index),
        }
    }

    /// Returns whether there is still input to encode.
    ///
    /// # Returns
    ///
    /// Returns `true` when more input values remain.
    #[inline(always)]
    pub(in crate::transcode) fn has_input(&self) -> bool {
        self.cursor.input_cursor() < self.input.len()
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
        let value =
            unsafe { qubit_io::UncheckedSlice::get(self.input, self.cursor.input_cursor()) };
        EncodeContext {
            input_value: value,
            input_index: self.cursor.input_cursor(),
            output: &mut *self.output,
            output_index: self.cursor.output_cursor(),
        }
    }

    /// Returns the number of writable output units from the current cursor.
    ///
    /// # Returns
    ///
    /// Returns writable output capacity from the current output cursor.
    #[inline(always)]
    fn available_output(&self) -> usize {
        self.output
            .len()
            .saturating_sub(self.cursor.output_cursor())
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
    pub(in crate::transcode) fn accept_written_value(&mut self, written: usize) {
        assert!(
            written <= self.available_output(),
            "encode step wrote beyond available output",
        );
        self.cursor.advance(1, written);
    }

    /// Returns completed progress for the current cursors.
    ///
    /// # Returns
    ///
    /// Returns a completed [`TranscodeProgress`] with consumed input and output
    /// counters.
    #[inline(always)]
    pub(in crate::transcode) fn complete_progress(&self) -> TranscodeProgress {
        TranscodeProgress::complete(self.cursor.read(), self.cursor.written())
    }

    /// Returns progress for a known missing output capacity.
    ///
    /// # Parameters
    ///
    /// - `required`: Total output units required from the current output
    ///   position.
    /// - `available`: Output units currently writable at the stop boundary.
    ///
    /// # Returns
    ///
    /// Returns [`TranscodeProgress::need_output`] with missing-capacity
    /// counters.
    #[inline(always)]
    pub(in crate::transcode) fn need_output_progress_with(
        &self,
        required: NonZeroUsize,
        available: usize,
    ) -> TranscodeProgress {
        TranscodeProgress::need_output(
            self.cursor.output_cursor(),
            required,
            available,
            self.cursor.read(),
            self.cursor.written(),
        )
    }

    /// Applies one normalized encode step to this encode state.
    ///
    /// # Parameters
    ///
    /// - `step`: Encode step produced by the encode engine.
    ///
    /// # Returns
    ///
    /// Returns stop progress when output is insufficient, otherwise `None`.
    #[inline]
    pub(in crate::transcode) fn apply_step(
        &mut self,
        step: EncodeStep,
    ) -> Option<TranscodeProgress> {
        match step {
            EncodeStep::Written { written } => {
                self.accept_written_value(written);
                None
            }
            EncodeStep::NeedOutput {
                required,
                available,
            } => Some(self.need_output_progress_with(required, available)),
        }
    }
}
