// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Mutable state for one buffered conversion call.

use core::num::NonZeroUsize;

use super::super::{
    engine::{
        DecodeContext,
        EncodeOutcome,
    },
    transcode_progress::TranscodeProgress,
};
use super::transcode_state::TranscodeState;
use super::{
    decode_step::DecodeStep,
    pending_value::PendingValue,
};

/// Mutable state for one buffered conversion call.
///
/// `ConvertState` is an internal cursor helper owned by
/// [`crate::TranscodeConvertEngine`]. Hook implementations receive narrower
/// context objects and never own converter cursor state.
pub(crate) struct ConvertState<'a, Input, Output> {
    /// Shared input/output state for this conversion call.
    state: TranscodeState<'a, Input, Output>,
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
    #[inline(always)]
    #[must_use]
    pub(crate) fn new(
        input: &'a [Input],
        input_index: usize,
        output: &'a mut [Output],
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
    /// # Type Parameters
    ///
    /// - `Input`: Source unit type visible to conversion.
    /// - `Output`: Target unit type visible to conversion.
    ///
    /// # Returns
    ///
    /// Returns the full input slice.
    #[inline(always)]
    #[must_use]
    pub(crate) fn input(&self) -> &[Input] {
        self.state.input()
    }

    /// Returns the complete output slice mutably.
    ///
    /// # Returns
    ///
    /// Returns the full mutable output slice.
    #[inline(always)]
    pub(crate) fn output_mut(&mut self) -> &mut [Output] {
        self.state.output_mut()
    }

    /// Returns the current output cursor.
    ///
    /// # Returns
    ///
    /// Returns current output cursor.
    #[inline(always)]
    #[must_use]
    pub(crate) fn output_cursor(&self) -> usize {
        self.state.output_cursor()
    }

    /// Returns whether there is still input to convert.
    ///
    /// # Returns
    ///
    /// Returns `true` when more input units remain.
    #[inline(always)]
    #[must_use]
    pub(crate) fn has_input(&self) -> bool {
        self.state.has_input()
    }

    /// Returns input units visible from the current input cursor.
    ///
    /// # Returns
    ///
    /// Returns remaining input units visible from `input_cursor`.
    #[inline(always)]
    #[must_use]
    pub(crate) fn available_input(&self) -> usize {
        self.state.available_input()
    }

    /// Returns writable output units visible from the current output cursor.
    ///
    /// # Returns
    ///
    /// Returns remaining writable output capacity from `output_cursor`.
    #[inline(always)]
    #[must_use]
    pub(crate) fn available_output(&self) -> usize {
        self.state.available_output()
    }

    /// Returns a public decode context snapshot at the current cursors.
    ///
    /// # Returns
    ///
    /// Returns context values suitable for decode-error hook dispatch.
    #[inline(always)]
    #[must_use]
    pub(crate) fn decode_context(&self) -> DecodeContext {
        DecodeContext::new(
            self.state.input_start(),
            self.state.input_cursor(),
            self.state.output_start(),
            self.state.output_cursor(),
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
        self.state.advance_input(read);
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
        self.state.advance_output(written);
    }

    /// Returns input units consumed since this call started.
    ///
    /// # Returns
    ///
    /// Returns consumed input units relative to `input_start`.
    #[inline(always)]
    #[must_use]
    pub(crate) fn read(&self) -> usize {
        self.state.read()
    }

    /// Returns output units written since this call started.
    ///
    /// # Returns
    ///
    /// Returns written output units relative to `output_start`.
    #[inline(always)]
    #[must_use]
    pub(crate) fn written(&self) -> usize {
        self.state.written()
    }

    /// Returns completed progress for the current cursors.
    ///
    /// # Returns
    ///
    /// Returns [`TranscodeProgress::complete`]-style state.
    #[inline(always)]
    #[must_use]
    pub(crate) fn complete_progress(&self) -> TranscodeProgress {
        self.state.complete_progress()
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
    #[inline(always)]
    #[must_use]
    pub(crate) fn need_input_progress(
        &self,
        required: NonZeroUsize,
        available: usize,
    ) -> TranscodeProgress {
        self.state.need_input_progress(required, available)
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
    #[inline(always)]
    #[must_use]
    pub(crate) fn need_output_progress(
        &self,
        required: NonZeroUsize,
        available: usize,
    ) -> TranscodeProgress {
        self.state.need_output_progress(required, available)
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

    /// Applies one normalized encode outcome to this conversion state.
    ///
    /// # Parameters
    ///
    /// - `outcome`: Encode outcome produced by the target encoder hooks.
    ///
    /// # Returns
    ///
    /// Returns `Some(progress)` when target output is insufficient and
    /// conversion must stop, otherwise returns `None` after advancing the
    /// output cursor.
    #[inline]
    #[must_use]
    pub(crate) fn apply_encode_outcome(
        &mut self,
        outcome: EncodeOutcome,
    ) -> Option<TranscodeProgress> {
        match outcome {
            EncodeOutcome::Consumed { written } => {
                self.advance_output(written);
                None
            }
            EncodeOutcome::NeedOutput { required } => {
                let available = self.available_output();
                assert!(
                    required.get() > available,
                    "EncodeOutcome::NeedOutput required capacity must exceed available output",
                );
                Some(self.need_output_progress(required, available))
            }
        }
    }
}
