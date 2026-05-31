/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Mutable state for one buffered conversion call.

use core::num::NonZeroUsize;

use super::{
    decode_attempt::DecodeAttempt,
    decode_context::DecodeContext,
    transcode_progress::TranscodeProgress,
};

/// Mutable state for one buffered conversion call.
///
/// `ConvertState` is an internal cursor helper owned by
/// [`crate::BufferedConvertEngine`]. Hook implementations receive narrower
/// context objects and never own converter cursor state.
pub(crate) struct ConvertState<'a, Input, Output> {
    /// Complete input unit slice visible to the converter.
    input: &'a [Input],
    /// Absolute input index where this call starts.
    input_start: usize,
    /// Complete output unit slice visible to the converter.
    output: &'a mut [Output],
    /// Absolute output index where this call starts.
    output_start: usize,
    /// Absolute input index for the next conversion step.
    input_cursor: usize,
    /// Absolute output index for the next write.
    output_cursor: usize,
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
    pub(crate) fn new(input: &'a [Input], input_index: usize, output: &'a mut [Output], output_index: usize) -> Self {
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

    /// Returns the complete input slice.
    #[must_use]
    #[inline(always)]
    pub(crate) fn input(&self) -> &[Input] {
        self.input
    }

    /// Returns the complete output slice mutably.
    #[inline(always)]
    pub(crate) fn output_mut(&mut self) -> &mut [Output] {
        self.output
    }

    /// Returns the current input cursor.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn input_cursor(&self) -> usize {
        self.input_cursor
    }

    /// Returns the current output cursor.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn output_cursor(&self) -> usize {
        self.output_cursor
    }

    /// Returns whether there is still input to convert.
    #[must_use]
    #[inline(always)]
    pub(crate) fn has_input(&self) -> bool {
        self.input_cursor < self.input.len()
    }

    /// Returns input units visible from the current input cursor.
    #[must_use]
    #[inline(always)]
    pub(crate) fn available_input(&self) -> usize {
        self.input.len() - self.input_cursor
    }

    /// Returns writable output units visible from the current output cursor.
    #[must_use]
    #[inline(always)]
    pub(crate) fn available_output(&self) -> usize {
        self.output.len().saturating_sub(self.output_cursor)
    }

    /// Returns a public decode context snapshot at the current cursors.
    ///
    /// # Returns
    ///
    /// Returns context values suitable for decode-error hook dispatch.
    #[must_use]
    pub(crate) fn decode_context(&self) -> DecodeContext {
        DecodeContext::new(
            self.input_start,
            self.input_cursor,
            self.output_start,
            self.output_cursor,
            self.available_input(),
        )
    }

    /// Returns whether the output cursor is within the visible output slice.
    #[must_use]
    #[inline(always)]
    pub(crate) fn output_cursor_in_bounds(&self) -> bool {
        self.output_cursor <= self.output.len()
    }

    /// Advances the input cursor.
    ///
    /// # Parameters
    ///
    /// - `read`: Number of input units consumed by the conversion step.
    #[inline(always)]
    pub(crate) fn advance_input(&mut self, read: usize) {
        debug_assert!(read <= self.available_input(), "conversion step read beyond input");
        self.input_cursor += read;
    }

    /// Advances the output cursor.
    ///
    /// # Parameters
    ///
    /// - `written`: Number of output units written by the conversion step.
    #[inline(always)]
    pub(crate) fn advance_output(&mut self, written: usize) {
        debug_assert!(
            written <= self.available_output(),
            "conversion step wrote beyond output",
        );
        self.output_cursor += written;
    }

    /// Returns input units consumed since this call started.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn read(&self) -> usize {
        self.input_cursor - self.input_start
    }

    /// Returns output units written since this call started.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn written(&self) -> usize {
        self.output_cursor - self.output_start
    }

    /// Returns a need-input decode attempt when fewer than `min_units` remain.
    ///
    /// # Parameters
    ///
    /// - `min_units`: Minimum source units required to attempt one decode.
    ///
    /// # Returns
    ///
    /// Returns `Some` when decoding must stop for more input, otherwise `None`.
    #[must_use]
    #[inline(always)]
    pub(crate) fn need_input_for_min_units<Value>(&self, min_units: usize) -> Option<DecodeAttempt<Value>> {
        let available = self.available_input();
        if available < min_units {
            Some(DecodeAttempt::need_input(
                NonZeroUsize::new(min_units - available).expect("missing input is non-zero"),
                available,
            ))
        } else {
            None
        }
    }

    /// Returns completed progress for the current cursors.
    #[must_use]
    pub(crate) fn complete_progress(&self) -> TranscodeProgress {
        TranscodeProgress::complete(self.read(), self.written())
    }

    /// Returns progress for missing input.
    ///
    /// # Parameters
    ///
    /// - `additional`: Additional input units required to continue.
    /// - `available`: Input units currently available at the stop boundary.
    ///
    /// # Returns
    ///
    /// Returns [`TranscodeProgress`] with [`TranscodeStatus::NeedInput`].
    #[must_use]
    pub(crate) fn need_input_progress(&self, additional: NonZeroUsize, available: usize) -> TranscodeProgress {
        TranscodeProgress::need_input(
            self.input_cursor,
            additional.get(),
            available,
            self.read(),
            self.written(),
        )
    }

    /// Returns progress for missing output.
    ///
    /// # Parameters
    ///
    /// - `additional`: Additional output units required to continue.
    /// - `available`: Output units currently available at the stop boundary.
    ///
    /// # Returns
    ///
    /// Returns [`TranscodeProgress`] with [`TranscodeStatus::NeedOutput`].
    #[must_use]
    pub(crate) fn need_output_progress(&self, additional: NonZeroUsize, available: usize) -> TranscodeProgress {
        TranscodeProgress::need_output(
            self.output_cursor,
            additional.get(),
            available,
            self.read(),
            self.written(),
        )
    }
}
