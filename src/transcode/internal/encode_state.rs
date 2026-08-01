// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Mutable state for one buffered encode call.

use core::num::NonZeroUsize;

use super::super::engine::{EncodeContext, EncodeOutcome};
use super::super::transcode_progress::TranscodeProgress;
use super::transcode_state::TranscodeState;

/// Mutable state for one buffered encode call.
pub(in crate::transcode) struct EncodeState<'a, Value, Unit> {
    /// Shared input/output state for this encode call.
    state: TranscodeState<'a, Value, Unit>,
}

/// Engine-owned mutable state for one encode attempt.
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
    #[inline(always)]
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
    #[inline(always)]
    pub(in crate::transcode) fn value(&self) -> &Value {
        self.value
    }

    /// Returns the absolute input index.
    #[inline(always)]
    pub(in crate::transcode) const fn input_index(&self) -> usize {
        self.input_index
    }

    /// Returns writable output capacity.
    #[inline(always)]
    pub(in crate::transcode) fn available_output(&self) -> usize {
        self.output.len().saturating_sub(self.output_index)
    }

    /// Returns the read-only policy view for this attempt.
    #[inline(always)]
    pub(in crate::transcode) fn context(&self) -> EncodeContext<'_, Value> {
        EncodeContext::new(
            self.value,
            self.input_index,
            self.output_index,
            self.available_output(),
        )
    }

    /// Returns all mutable engine state for the encode operation.
    #[inline(always)]
    pub(in crate::transcode) fn into_parts(self) -> (&'a Value, usize, &'a mut [Unit], usize) {
        (self.value, self.input_index, self.output, self.output_index)
    }
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
        Self {
            state: TranscodeState::new(input, input_index, output, output_index),
        }
    }

    /// Returns whether there is still input to encode.
    ///
    /// # Returns
    ///
    /// Returns `true` when more input values remain.
    #[inline(always)]
    pub(in crate::transcode) fn has_input(&self) -> bool {
        self.state.has_input()
    }

    /// Returns an encode context at the current cursors.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `self.has_input()` returned `true`.
    #[inline(always)]
    pub(in crate::transcode) unsafe fn context_unchecked(
        &mut self,
    ) -> EncodeAttempt<'_, Value, Unit> {
        let input_index = self.state.input_cursor();
        let output_index = self.state.output_cursor();
        let (input, output) = self.state.input_output_mut();
        // SAFETY: Guaranteed by the caller.
        let value = unsafe { input.get_unchecked(input_index) };
        EncodeAttempt::new(value, input_index, output, output_index)
    }

    /// Returns the number of writable output units from the current cursor.
    ///
    /// # Returns
    ///
    /// Returns writable output capacity from the current output cursor.
    #[inline(always)]
    fn available_output(&self) -> usize {
        self.state.available_output()
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
            "EncodeOutcome::Consumed wrote beyond available output",
        );
        self.state.advance(1, written);
    }

    /// Returns completed progress for the current cursors.
    ///
    /// # Returns
    ///
    /// Returns a completed [`TranscodeProgress`] with consumed input and output
    /// counters.
    #[inline(always)]
    pub(in crate::transcode) fn complete_progress(&self) -> TranscodeProgress {
        self.state.complete_progress()
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
        self.state.need_output_progress(required, available)
    }

    /// Applies one encode outcome to this encode state.
    ///
    /// # Parameters
    ///
    /// - `outcome`: Encode outcome produced by the encode engine.
    ///
    /// # Returns
    ///
    /// Returns stop progress when output is insufficient, otherwise `None`.
    #[inline]
    pub(in crate::transcode) fn apply_encode_outcome(
        &mut self,
        outcome: EncodeOutcome,
    ) -> Option<TranscodeProgress> {
        match outcome {
            EncodeOutcome::Consumed { written } => {
                self.accept_written_value(written);
                None
            }
            EncodeOutcome::NeedOutput { required } => {
                let available = self.available_output();
                assert!(
                    required.get() > available,
                    "EncodeOutcome::NeedOutput required capacity must exceed available output",
                );
                Some(self.need_output_progress_with(required, available))
            }
        }
    }
}
