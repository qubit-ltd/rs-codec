// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Mutable state for one buffered conversion call.

use core::num::NonZeroUsize;

use super::super::{decode_context::DecodeContext, transcode_progress::TranscodeProgress};
use super::cursor_state::CursorState;
use super::{decode_step::DecodeStep, pending_value::PendingValue};

/// Mutable state for one buffered conversion call.
///
/// `ConvertState` is an internal cursor helper owned by
/// [`crate::TranscodeConvertEngine`]. Hook implementations receive narrower
/// context objects and never own converter cursor state.
pub(crate) struct ConvertState<'a, Input, Output> {
    /// Complete input unit slice visible to the converter.
    input: &'a [Input],
    /// Complete output unit slice visible to the converter.
    output: &'a mut [Output],
    /// Shared absolute input/output cursors.
    cursor: CursorState,
}

impl<'a, Input, Output> ConvertState<'a, Input, Output> {
    /// Creates mutable conversion state.
    ///
    /// # Parameters
    ///
    /// - `input`: Complete input unit slice visible to the converter.
    /// - `input_index`: Absolute input index where conversion starts.
    /// - `output`: Complete output unit slice visible to the converter.
    /// - `output_index`: Absolute output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns initialized conversion state with cursors at the requested start
    /// positions.
    #[must_use]
    #[inline(always)]
    pub(crate) fn new(
        input: &'a [Input],
        input_index: usize,
        output: &'a mut [Output],
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
    /// # Type Parameters
    ///
    /// - `Input`: Source unit type visible to conversion.
    /// - `Output`: Target unit type visible to conversion.
    ///
    /// # Returns
    ///
    /// Returns the full input slice.
    #[must_use]
    #[inline(always)]
    pub(crate) fn input(&self) -> &[Input] {
        self.input
    }

    /// Returns the complete output slice mutably.
    ///
    /// # Returns
    ///
    /// Returns the full mutable output slice.
    #[inline(always)]
    pub(crate) fn output_mut(&mut self) -> &mut [Output] {
        self.output
    }

    /// Returns the current output cursor.
    ///
    /// # Returns
    ///
    /// Returns current output cursor.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn output_cursor(&self) -> usize {
        self.cursor.output_cursor()
    }

    /// Returns whether there is still input to convert.
    ///
    /// # Returns
    ///
    /// Returns `true` when more input units remain.
    #[must_use]
    #[inline(always)]
    pub(crate) fn has_input(&self) -> bool {
        self.cursor.input_cursor() < self.input.len()
    }

    /// Returns input units visible from the current input cursor.
    ///
    /// # Returns
    ///
    /// Returns remaining input units visible from `input_cursor`.
    #[must_use]
    #[inline(always)]
    pub(crate) fn available_input(&self) -> usize {
        self.input.len() - self.cursor.input_cursor()
    }

    /// Returns writable output units visible from the current output cursor.
    ///
    /// # Returns
    ///
    /// Returns remaining writable output capacity from `output_cursor`.
    #[must_use]
    #[inline(always)]
    pub(crate) fn available_output(&self) -> usize {
        self.output
            .len()
            .saturating_sub(self.cursor.output_cursor())
    }

    /// Returns a public decode context snapshot at the current cursors.
    ///
    /// # Returns
    ///
    /// Returns context values suitable for decode-error hook dispatch.
    #[must_use]
    #[inline(always)]
    pub(crate) fn decode_context(&self) -> DecodeContext {
        DecodeContext::new(
            self.cursor.input_start(),
            self.cursor.input_cursor(),
            self.cursor.output_start(),
            self.cursor.output_cursor(),
            self.available_input(),
        )
    }

    /// Advances the input cursor.
    ///
    /// # Parameters
    ///
    /// - `read`: Number of input units consumed by the conversion step.
    #[inline(always)]
    pub(crate) fn advance_input(&mut self, read: usize) {
        assert!(
            read <= self.available_input(),
            "conversion step read beyond input"
        );
        self.cursor.advance_input(read);
    }

    /// Advances the output cursor.
    ///
    /// # Parameters
    ///
    /// - `written`: Number of output units written by the conversion step.
    #[inline(always)]
    pub(crate) fn advance_output(&mut self, written: usize) {
        assert!(
            written <= self.available_output(),
            "conversion step wrote beyond output",
        );
        self.cursor.advance_output(written);
    }

    /// Returns input units consumed since this call started.
    ///
    /// # Returns
    ///
    /// Returns consumed input units relative to `input_start`.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn read(&self) -> usize {
        self.cursor.read()
    }

    /// Returns output units written since this call started.
    ///
    /// # Returns
    ///
    /// Returns written output units relative to `output_start`.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn written(&self) -> usize {
        self.cursor.written()
    }

    /// Returns completed progress for the current cursors.
    ///
    /// # Returns
    ///
    /// Returns [`TranscodeProgress::complete`]-style state.
    #[must_use]
    #[inline(always)]
    pub(crate) fn complete_progress(&self) -> TranscodeProgress {
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
    /// Returns [`TranscodeProgress`] with [`TranscodeStatus::NeedInput`].
    #[must_use]
    #[inline(always)]
    pub(crate) fn need_input_progress(
        &self,
        required: NonZeroUsize,
        available: usize,
    ) -> TranscodeProgress {
        TranscodeProgress::need_input(
            self.cursor.input_cursor(),
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
    /// Returns [`TranscodeProgress`] with [`TranscodeStatus::NeedOutput`].
    #[must_use]
    #[inline(always)]
    pub(crate) fn need_output_progress(
        &self,
        required: NonZeroUsize,
        available: usize,
    ) -> TranscodeProgress {
        TranscodeProgress::need_output(
            self.cursor.output_cursor(),
            required,
            available,
            self.read(),
            self.written(),
        )
    }

    /// Applies one normalized decode step to this conversion state.
    ///
    /// # Parameters
    ///
    /// - `step`: Decoded step produced by the decode engine.
    /// - `encode`: Callback to encode a decoded value into the target side.
    ///
    /// # Type Parameters
    ///
    /// - `Value`: Decoded logical value type.
    /// - `Error`: Error type produced by the encode callback.
    /// - `F`: Callback type that consumes one decoded value.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(progress))` when conversion must stop and `Ok(None)`
    /// when it can continue.
    #[inline]
    pub(crate) fn apply_decode_step<Value, Error, F>(
        &mut self,
        step: DecodeStep<Value>,
        mut encode: F,
    ) -> Result<Option<TranscodeProgress>, Error>
    where
        F: FnMut(
            PendingValue<Value>,
            &mut ConvertState<'_, Input, Output>,
        ) -> Result<Option<TranscodeProgress>, Error>,
    {
        match step {
            DecodeStep::Decoded {
                value,
                consumed,
                input_index,
            } => {
                self.advance_input(consumed.get());
                encode(PendingValue::new(value, input_index), self)
            }
            DecodeStep::Skipped { consumed } => {
                self.advance_input(consumed.get());
                Ok(None)
            }
            DecodeStep::NeedInput {
                required,
                available,
            } => Ok(Some(self.need_input_progress(required, available))),
        }
    }
}
