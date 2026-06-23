// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared input/output cursor bookkeeping for transcode states.

/// Shared absolute cursor state for one transcode call.
///
/// `CursorState` records the call start positions and the live input/output
/// cursors used by [`super::encode_state::EncodeState`],
/// [`super::decode_state::DecodeState`], and
/// [`super::convert_state::ConvertState`]. Progress counters are derived from
/// the difference between each cursor and its corresponding start position.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::transcode) struct CursorState {
    /// Absolute input index where this call starts.
    input_start: usize,
    /// Absolute output index where this call starts.
    output_start: usize,
    /// Absolute input index for the next operation.
    input_cursor: usize,
    /// Absolute output index for the next write.
    output_cursor: usize,
}

impl CursorState {
    /// Creates cursor state with both cursors at their call starts.
    ///
    /// # Parameters
    ///
    /// - `input_start`: Absolute input index where this call begins.
    /// - `output_start`: Absolute output index where this call begins.
    ///
    /// # Returns
    ///
    /// Returns cursor state with `input_cursor == input_start` and
    /// `output_cursor == output_start`.
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) const fn new(input_start: usize, output_start: usize) -> Self {
        Self {
            input_start,
            output_start,
            input_cursor: input_start,
            output_cursor: output_start,
        }
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

    /// Returns input units consumed since this call started.
    ///
    /// # Returns
    ///
    /// Returns consumed input units relative to [`Self::input_start`].
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) const fn read(&self) -> usize {
        self.input_cursor - self.input_start
    }

    /// Returns output units written since this call started.
    ///
    /// # Returns
    ///
    /// Returns written output units relative to [`Self::output_start`].
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
    pub(in crate::transcode) fn advance(&mut self, read: usize, written: usize) {
        self.advance_input(read);
        self.advance_output(written);
    }
}
