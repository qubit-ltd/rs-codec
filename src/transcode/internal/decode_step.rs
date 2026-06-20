// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal decode-step result used by buffered converters.

use core::num::NonZeroUsize;

use super::super::transcode_progress::TranscodeProgress;
use super::{convert_state::ConvertState, decode_state::DecodeState, pending_value::PendingValue};

/// Result of one decode step in the converter loop.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::transcode) enum DecodeStep<Value> {
    /// A source value was decoded or emitted by policy.
    Decoded {
        /// Decoded logical value.
        value: Value,
        /// Number of consumed source units.
        consumed: NonZeroUsize,
        /// Source input index used for downstream encode context.
        input_index: usize,
    },
    /// Source input was consumed without producing a value.
    Skipped {
        /// Number of consumed source units.
        consumed: NonZeroUsize,
    },
    /// More source input is required before decoding can continue.
    NeedInput {
        /// Additional source units required to continue.
        additional: NonZeroUsize,
        /// Source units available at the incomplete boundary.
        available: usize,
    },
}

impl<Value> DecodeStep<Value> {
    /// Creates a decoded-value step.
    ///
    /// # Parameters
    ///
    /// - `value`: Decoded logical value.
    /// - `consumed`: Number of consumed source units.
    /// - `input_index`: Source index used for downstream encode context.
    ///
    /// # Returns
    ///
    /// Returns a decoded step.
    #[inline(always)]
    pub(in crate::transcode) const fn decoded(
        value: Value,
        consumed: NonZeroUsize,
        input_index: usize,
    ) -> Self {
        Self::Decoded {
            value,
            consumed,
            input_index,
        }
    }

    /// Creates a skipped-input step.
    ///
    /// # Parameters
    ///
    /// - `consumed`: Number of consumed source units.
    ///
    /// # Returns
    ///
    /// Returns a skipped step.
    #[inline(always)]
    pub(in crate::transcode) const fn skipped(consumed: NonZeroUsize) -> Self {
        Self::Skipped { consumed }
    }

    /// Creates a missing-input step.
    ///
    /// # Parameters
    ///
    /// - `additional`: Additional source units required to continue.
    /// - `available`: Source units currently available.
    ///
    /// # Returns
    ///
    /// Returns a need-input step.
    #[inline(always)]
    pub(in crate::transcode) const fn need_input(
        additional: NonZeroUsize,
        available: usize,
    ) -> Self {
        Self::NeedInput {
            additional,
            available,
        }
    }

    /// Applies this decode step to the current conversion state.
    ///
    /// For each variant:
    ///
    /// - `Decoded`: advances input cursor and passes the decoded value into the
    ///   converter encode path.
    /// - `Skipped`: only advances input cursor.
    /// - `NeedInput`: returns [`TranscodeProgress`] requesting additional
    ///   input.
    ///
    /// # Parameters
    ///
    /// - `state`: Current conversion-call state.
    /// - `encode`: Callback to emit a decoded value into the shared convert
    ///   state.
    ///
    /// # Type Parameters
    ///
    /// - `Input`: Input-unit type stored in the active [`ConvertState`].
    /// - `Output`: Output-unit type stored in the active [`ConvertState`].
    /// - `Error`: Error type produced by the encode callback.
    /// - `F`: Callback type that consumes a decoded value and updates state.
    ///
    /// # Returns
    ///
    /// Returns:
    /// - `Ok(Some(progress))` when conversion must stop,
    /// - `Ok(None)` when conversion can continue, or
    /// - `Err(error)` when emitting the decoded value fails.
    #[inline]
    pub(in crate::transcode) fn apply_to_convert_state<Input, Output, Error, F>(
        self,
        state: &mut ConvertState<'_, Input, Output>,
        mut encode: F,
    ) -> Result<Option<TranscodeProgress>, Error>
    where
        F: FnMut(
            PendingValue<Value>,
            &mut ConvertState<'_, Input, Output>,
        ) -> Result<Option<TranscodeProgress>, Error>,
    {
        match self {
            Self::Decoded {
                value,
                consumed,
                input_index,
            } => {
                state.advance_input(consumed.get());
                encode(PendingValue::new(value, input_index), state)
            }
            Self::Skipped { consumed } => {
                state.advance_input(consumed.get());
                Ok(None)
            }
            Self::NeedInput {
                additional,
                available,
            } => Ok(Some(state.need_input_progress(additional, available))),
        }
    }

    /// Applies this decode step to the current decode state.
    ///
    /// # Parameters
    ///
    /// - `state`: Current decode-call state.
    ///
    /// # Type Parameters
    ///
    /// - `Unit`: Decode input-unit type represented by this [`DecodeState`].
    ///
    /// # Returns
    ///
    /// Returns optional public stop progress when decoding cannot continue in
    /// this call.
    #[must_use]
    #[inline]
    pub(in crate::transcode) fn apply_to_decode_state<Unit>(
        self,
        state: &mut DecodeState<'_, Unit, Value>,
    ) -> Option<TranscodeProgress> {
        match self {
            Self::Decoded {
                value, consumed, ..
            } => {
                if state.needs_output() {
                    return Some(state.need_output_progress());
                }
                state.emit(value, consumed);
                None
            }
            Self::Skipped { consumed } => {
                state.skip(consumed);
                None
            }
            Self::NeedInput {
                additional,
                available,
            } => Some(state.need_input_progress_with(additional, available)),
        }
    }
}
