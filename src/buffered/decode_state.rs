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
    transcode_status::TranscodeStatus,
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
    #[inline(always)]
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
    fn required_input(&self) -> usize {
        self.min_units.get() - self.available()
    }

    /// Returns a public decode context snapshot.
    #[inline(always)]
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
        self.output[self.output_cursor] = value;
        self.input_cursor += consumed;
        self.output_cursor += 1;
    }

    /// Normalizes a policy-reported consumption count.
    ///
    /// # Parameters
    ///
    /// - `consumed`: Units requested by the concrete policy.
    ///
    /// # Returns
    ///
    /// Returns a count within the visible input range. When input is available,
    /// the returned count is at least one so policy handling makes progress.
    #[inline(always)]
    pub(super) fn normalize_consumed(&self, consumed: usize) -> NonZeroUsize {
        let available = self.available();
        debug_assert!(available > 0, "decode action cannot consume empty input");
        let consumed = consumed.min(available).max(1);
        // SAFETY: The normalized count is clamped to at least one.
        unsafe { NonZeroUsize::new_unchecked(consumed) }
    }

    /// Returns completed progress for the current cursors.
    #[inline(always)]
    pub(super) fn complete_progress(&self) -> TranscodeProgress {
        TranscodeProgress::complete(
            self.input_cursor - self.input_start,
            self.output_cursor - self.output_start,
        )
    }

    /// Returns progress for a stop status at the current cursors.
    #[inline(always)]
    pub(super) fn status_progress(&self, status: TranscodeStatus) -> TranscodeProgress {
        TranscodeProgress::new(
            status,
            self.input_cursor - self.input_start,
            self.output_cursor - self.output_start,
        )
    }

    /// Returns progress for a missing output slot.
    #[inline(always)]
    pub(super) fn need_output_progress(&self) -> TranscodeProgress {
        let context = self.context();
        TranscodeProgress::need_output(
            context.output_index,
            1,
            0,
            self.input_cursor - self.input_start,
            self.output_cursor - self.output_start,
        )
    }

    /// Returns progress for a missing input unit.
    #[inline(always)]
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
}
