// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Mutable state for one buffered decode call.

use core::num::NonZeroUsize;

use super::super::{decode_context::DecodeContext, transcode_progress::TranscodeProgress};
use super::cursor_state::CursorState;
use super::decode_step::DecodeStep;

/// Mutable state for one buffered decode call.
pub(in crate::transcode) struct DecodeState<'a, Unit, Value> {
    /// Complete input unit slice visible to the decoder.
    input: &'a [Unit],
    /// Complete output value slice visible to the decoder.
    output: &'a mut [Value],
    /// Shared absolute input/output cursors.
    cursor: CursorState,
}

impl<'a, Unit, Value> DecodeState<'a, Unit, Value> {
    /// Creates mutable decode state.
    ///
    /// # Parameters
    ///
    /// - `input`: Complete input unit slice visible to the decoder.
    /// - `input_index`: Absolute source index where decoding starts.
    /// - `output`: Complete output value slice visible to the decoder.
    /// - `output_index`: Absolute output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns initialized decode state with cursors at the requested start
    /// positions.
    #[inline(always)]
    pub(in crate::transcode) fn new(
        input: &'a [Unit],
        input_index: usize,
        output: &'a mut [Value],
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

    /// Returns the complete input slice.
    ///
    /// # Returns
    ///
    /// Returns the full input slice visible to this decode call.
    #[inline(always)]
    pub(in crate::transcode) fn input(&self) -> &[Unit] {
        self.input
    }

    /// Returns whether there is still input to decode.
    ///
    /// # Returns
    ///
    /// Returns `true` when there are input units remaining.
    #[inline(always)]
    pub(in crate::transcode) fn has_input(&self) -> bool {
        self.cursor.input_cursor() < self.input.len()
    }

    /// Returns whether the output slice has no slot for the next value.
    ///
    /// # Returns
    ///
    /// Returns `true` when no output slot remains for the next value.
    #[inline(always)]
    pub(in crate::transcode) fn needs_output(&self) -> bool {
        self.cursor.output_cursor() == self.output.len()
    }

    /// Returns input units visible from the current input cursor.
    #[inline(always)]
    fn available(&self) -> usize {
        self.input.len() - self.cursor.input_cursor()
    }

    /// Returns a public decode context snapshot.
    ///
    /// # Returns
    ///
    /// Returns the current [`DecodeContext`].
    #[inline(always)]
    pub(in crate::transcode) fn context(&self) -> DecodeContext {
        DecodeContext::new(
            self.cursor.input_start(),
            self.cursor.input_cursor(),
            self.cursor.output_start(),
            self.cursor.output_cursor(),
            self.available(),
        )
    }

    /// Advances the input cursor without emitting output.
    ///
    /// # Parameters
    ///
    /// - `consumed`: Input units consumed by the current operation.
    ///
    /// # Returns
    ///
    /// Returns unit `()`.
    #[inline(always)]
    pub(in crate::transcode) fn skip(&mut self, consumed: NonZeroUsize) {
        let consumed = consumed.get();
        assert!(
            consumed <= self.available(),
            "decode step consumed beyond available input",
        );
        self.cursor.advance_input(consumed);
    }

    /// Emits a decoded value and advances both cursors.
    ///
    /// # Parameters
    ///
    /// - `value`: Decoded value to write to output.
    /// - `consumed`: Input units consumed by this decode step.
    ///
    /// # Returns
    ///
    /// Returns unit `()`.
    #[inline(always)]
    pub(in crate::transcode) fn emit(&mut self, value: Value, consumed: NonZeroUsize) {
        let consumed = consumed.get();
        assert!(
            consumed <= self.available(),
            "decode step consumed beyond available input",
        );
        assert!(
            !self.needs_output(),
            "decode step emitted without output capacity",
        );
        // SAFETY: `needs_output()` returned false, so the output cursor points
        // at a writable slot.
        unsafe {
            *qubit_io::UncheckedSlice::get_mut(self.output, self.cursor.output_cursor()) = value;
        }
        self.cursor.advance(consumed, 1);
    }

    /// Returns completed progress for the current cursors.
    ///
    /// # Returns
    ///
    /// Returns a completed [`TranscodeProgress`].
    #[inline(always)]
    pub(in crate::transcode) fn complete_progress(&self) -> TranscodeProgress {
        TranscodeProgress::complete(self.cursor.read(), self.cursor.written())
    }

    /// Returns progress for a missing output slot.
    ///
    /// # Returns
    ///
    /// Returns progress with [`TranscodeStatus::NeedOutput`].
    #[inline(always)]
    pub(in crate::transcode) fn need_output_progress(&self) -> TranscodeProgress {
        let context = self.context();
        TranscodeProgress::need_output(
            context.output_index(),
            NonZeroUsize::MIN,
            0,
            self.cursor.read(),
            self.cursor.written(),
        )
    }

    /// Returns progress for a policy-selected need-input stop.
    ///
    /// # Parameters
    ///
    /// - `required`: Total source units required from the current input
    ///   position.
    /// - `available`: Source units visible at the stop boundary.
    ///
    /// # Returns
    ///
    /// Returns progress at the current decode cursor.
    #[inline(always)]
    pub(in crate::transcode) fn need_input_progress_with(
        &self,
        required: NonZeroUsize,
        available: usize,
    ) -> TranscodeProgress {
        TranscodeProgress::need_input(
            self.cursor.input_cursor(),
            required,
            available,
            self.cursor.read(),
            self.cursor.written(),
        )
    }

    /// Applies one normalized decode step to this decode state.
    ///
    /// # Parameters
    ///
    /// - `step`: Decoded step produced by the decode engine.
    ///
    /// # Returns
    ///
    /// Returns optional [`TranscodeProgress`] when decoding must stop in this
    /// call.
    #[must_use]
    #[inline]
    pub(in crate::transcode) fn apply_decode_step(
        &mut self,
        step: DecodeStep<Value>,
    ) -> Option<TranscodeProgress> {
        match step {
            DecodeStep::Decoded {
                value, consumed, ..
            } => {
                if self.needs_output() {
                    return Some(self.need_output_progress());
                }
                self.emit(value, consumed);
                None
            }
            DecodeStep::Skipped { consumed } => {
                self.skip(consumed);
                None
            }
            DecodeStep::NeedInput {
                required,
                available,
            } => Some(self.need_input_progress_with(required, available)),
        }
    }
}
