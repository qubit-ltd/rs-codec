/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Mutable state for one buffered decode call.

use core::num::NonZeroUsize;

use super::{
    decode_context::DecodeContext,
    transcode_progress::TranscodeProgress,
};

/// Mutable state for one buffered decode call.
pub(super) struct DecodeState<'a, Unit, Value> {
    /// Complete input unit slice visible to the decoder.
    input: &'a [Unit],
    /// Absolute source index where this call starts.
    input_start: usize,
    /// Complete output value slice visible to the decoder.
    output: &'a mut [Value],
    /// Absolute output index where this call starts.
    output_start: usize,
    /// Minimum input units required to attempt one decode.
    min_units: NonZeroUsize,
    /// Absolute source index for the next decode attempt.
    input_cursor: usize,
    /// Absolute output index for the next emitted value.
    output_cursor: usize,
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
    /// - `min_units`: Minimum input units required to attempt one decode.
    ///
    /// # Returns
    ///
    /// Returns initialized decode state with cursors at the requested start
    /// positions.
    pub(super) fn new(
        input: &'a [Unit],
        input_index: usize,
        output: &'a mut [Value],
        output_index: usize,
        min_units: NonZeroUsize,
    ) -> Self {
        debug_assert!(input_index <= input.len(), "input index must be within the input slice");

        Self {
            input,
            input_start: input_index,
            output,
            output_start: output_index,
            min_units,
            input_cursor: input_index,
            output_cursor: output_index,
        }
    }

    /// Returns the complete input slice.
    #[inline(always)]
    pub(super) fn input(&self) -> &[Unit] {
        self.input
    }

    /// Returns the current input cursor.
    #[inline(always)]
    pub(super) fn input_cursor(&self) -> usize {
        self.input_cursor
    }

    /// Returns whether there is still input to decode.
    #[inline(always)]
    pub(super) fn has_input(&self) -> bool {
        self.input_cursor < self.input.len()
    }

    /// Returns whether the output cursor is within the visible output slice.
    #[inline(always)]
    pub(super) fn output_cursor_in_bounds(&self) -> bool {
        self.output_cursor <= self.output.len()
    }

    /// Returns whether the output slice has no slot for the next value.
    #[inline(always)]
    pub(super) fn needs_output(&self) -> bool {
        self.output_cursor == self.output.len()
    }

    /// Returns input units visible from the current input cursor.
    #[inline(always)]
    fn available(&self) -> usize {
        self.input.len() - self.input_cursor
    }

    /// Returns whether more input is required before decoding can continue.
    #[inline(always)]
    pub(super) fn needs_input(&self) -> bool {
        self.available() < self.min_units.get()
    }

    /// Returns the additional input units required by the minimum decode width.
    #[inline(always)]
    fn required_input(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.min_units.get() - self.available()).expect("missing input is non-zero")
    }

    /// Returns a public decode context snapshot.
    pub(super) fn context(&self) -> DecodeContext {
        DecodeContext::new(
            self.input_start,
            self.input_cursor,
            self.output_start,
            self.output_cursor,
            self.available(),
        )
    }

    /// Advances the input cursor without emitting output.
    #[inline(always)]
    pub(super) fn skip(&mut self, consumed: NonZeroUsize) {
        let consumed = consumed.get();
        debug_assert!(
            consumed <= self.available(),
            "decode step consumed beyond available input",
        );
        self.input_cursor += consumed;
    }

    /// Emits a decoded value and advances both cursors.
    #[inline(always)]
    pub(super) fn emit(&mut self, value: Value, consumed: NonZeroUsize) {
        let consumed = consumed.get();
        debug_assert!(
            consumed <= self.available(),
            "decode step consumed beyond available input",
        );
        debug_assert!(!self.needs_output(), "decode step emitted without output capacity",);
        // SAFETY: `needs_output()` returned false, so the output cursor points
        // at a writable slot.
        unsafe {
            *self.output.get_unchecked_mut(self.output_cursor) = value;
        }
        self.input_cursor += consumed;
        self.output_cursor += 1;
    }

    /// Returns completed progress for the current cursors.
    pub(super) fn complete_progress(&self) -> TranscodeProgress {
        TranscodeProgress::complete(
            self.input_cursor - self.input_start,
            self.output_cursor - self.output_start,
        )
    }

    /// Returns progress for a missing output slot.
    pub(super) fn need_output_progress(&self) -> TranscodeProgress {
        let context = self.context();
        TranscodeProgress::need_output(
            context.output_index,
            NonZeroUsize::MIN,
            0,
            self.input_cursor - self.input_start,
            self.output_cursor - self.output_start,
        )
    }

    /// Returns progress for a missing input unit.
    pub(super) fn need_input_progress(&self) -> TranscodeProgress {
        let context = self.context();
        TranscodeProgress::need_input(
            context.input_index,
            self.required_input(),
            context.available,
            self.input_cursor - self.input_start,
            self.output_cursor - self.output_start,
        )
    }

    /// Returns progress for a policy-selected need-input stop.
    ///
    /// # Parameters
    ///
    /// - `additional`: Additional source units required to continue.
    /// - `available`: Source units visible at the stop boundary.
    ///
    /// # Returns
    ///
    /// Returns progress at the current decode cursor.
    pub(super) fn need_input_progress_with(&self, additional: NonZeroUsize, available: usize) -> TranscodeProgress {
        TranscodeProgress::need_input(
            self.input_cursor,
            additional,
            available,
            self.input_cursor - self.input_start,
            self.output_cursor - self.output_start,
        )
    }
}
