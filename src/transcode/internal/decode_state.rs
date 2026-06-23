// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Mutable state for one buffered decode call.

use core::num::NonZeroUsize;

use super::super::{
    decode_context::DecodeContext,
    transcode_progress::TranscodeProgress,
};
use super::decode_step::DecodeStep;
use super::transcode_state::TranscodeState;

/// Mutable state for one buffered decode call.
pub(in crate::transcode) struct DecodeState<'a, Unit, Value> {
    /// Shared input/output state for this decode call.
    state: TranscodeState<'a, Unit, Value>,
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
        Self {
            state: TranscodeState::new(
                input,
                input_index,
                output,
                output_index,
            ),
        }
    }

    /// Returns the complete input slice.
    ///
    /// # Returns
    ///
    /// Returns the full input slice visible to this decode call.
    #[inline(always)]
    pub(in crate::transcode) fn input(&self) -> &[Unit] {
        self.state.input()
    }

    /// Returns whether there is still input to decode.
    ///
    /// # Returns
    ///
    /// Returns `true` when there are input units remaining.
    #[inline(always)]
    pub(in crate::transcode) fn has_input(&self) -> bool {
        self.state.has_input()
    }

    /// Returns whether the output slice has no slot for the next value.
    ///
    /// # Returns
    ///
    /// Returns `true` when no output slot remains for the next value.
    #[inline(always)]
    pub(in crate::transcode) fn needs_output(&self) -> bool {
        self.state.needs_output()
    }

    /// Returns input units visible from the current input cursor.
    #[inline(always)]
    fn available(&self) -> usize {
        self.state.available_input()
    }

    /// Returns a public decode context snapshot.
    ///
    /// # Returns
    ///
    /// Returns the current [`DecodeContext`].
    #[inline(always)]
    pub(in crate::transcode) fn context(&self) -> DecodeContext {
        DecodeContext::new(
            self.state.input_start(),
            self.state.input_cursor(),
            self.state.output_start(),
            self.state.output_cursor(),
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
        self.state.advance_input(consumed);
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
    pub(in crate::transcode) fn emit(
        &mut self,
        value: Value,
        consumed: NonZeroUsize,
    ) {
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
        let output_cursor = self.state.output_cursor();
        unsafe {
            *qubit_io::UncheckedSlice::get_mut(
                self.state.output_mut(),
                output_cursor,
            ) = value;
        }
        self.state.advance(consumed, 1);
    }

    /// Returns completed progress for the current cursors.
    ///
    /// # Returns
    ///
    /// Returns a completed [`TranscodeProgress`].
    #[inline(always)]
    pub(in crate::transcode) fn complete_progress(&self) -> TranscodeProgress {
        self.state.complete_progress()
    }

    /// Returns progress for a missing output slot.
    ///
    /// # Returns
    ///
    /// Returns progress with [`TranscodeStatus::NeedOutput`].
    #[inline(always)]
    pub(in crate::transcode) fn need_output_progress(
        &self,
    ) -> TranscodeProgress {
        self.state.need_output_progress(NonZeroUsize::MIN, 0)
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
        self.state.need_input_progress(required, available)
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
    #[inline]
    #[must_use]
    pub(in crate::transcode) fn apply_decode_step(
        &mut self,
        step: DecodeStep<Value>,
    ) -> Option<TranscodeProgress> {
        match step {
            DecodeStep::Decoded {
                value, consumed, ..
            } => {
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
