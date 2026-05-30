/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Policy hooks used by buffered convert engines.

use core::num::NonZeroUsize;

use super::{
    convert_state::ConvertState,
    transcode_progress::TranscodeProgress,
};
use crate::ConvertErrorFactory;

/// Result of decoding the next source value during buffered conversion.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ConvertDecodeResult<Value> {
    /// A source value was decoded.
    Decoded {
        /// Decoded logical value.
        value: Value,
        /// Number of source input units consumed.
        consumed: NonZeroUsize,
    },

    /// Source input was consumed without producing a value.
    Skipped {
        /// Number of source input units consumed.
        consumed: NonZeroUsize,
    },

    /// More input is required before another value can be decoded.
    NeedInput {
        /// Additional source units required to continue.
        additional: NonZeroUsize,
        /// Source units available at the incomplete boundary.
        available: usize,
    },
}

/// Result of writing one decoded value during buffered conversion.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ConvertWriteResult {
    /// The value was fully written.
    Written {
        /// Number of target output units written.
        written: usize,
    },

    /// The value was retained by hooks because the output buffer is too small.
    NeedOutput {
        /// Additional target units required to continue.
        additional: NonZeroUsize,
        /// Target units available at the output boundary.
        available: usize,
        /// Target output units written before output capacity was exhausted.
        written: usize,
    },
}

/// Policy hooks for [`crate::BufferedConvertEngine`].
///
/// Hooks own policy state, such as pending decoded values. The engine owns the
/// source and target components, performs common index validation, and drives a
/// fixed decode-then-encode conversion loop through these hook methods.
///
/// # Type Parameters
///
/// - `D`: Source-side decoder or input component owned by the engine.
/// - `E`: Target-side encoder or output component owned by the engine.
/// - `Input`: Source unit type.
/// - `Value`: Logical value type decoded from source units and encoded to target units.
/// - `Output`: Target unit type.
pub trait BufferedConvertHooks<D, E, Input, Value, Output> {
    /// Error type returned by the buffered converter.
    type Error: ConvertErrorFactory<D>;

    /// Returns the additional output units requested when `output_index` is invalid.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Source component owned by the engine.
    /// - `encoder`: Target component owned by the engine.
    ///
    /// # Returns
    ///
    /// Returns at least one additional output unit.
    #[must_use]
    #[inline(always)]
    fn invalid_output_additional(&self, _decoder: &D, _encoder: &E) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    /// Returns an upper bound for target units produced from `input_len` units.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Source component owned by the engine.
    /// - `encoder`: Target component owned by the engine.
    /// - `input_len`: Number of source units the caller plans to convert.
    ///
    /// # Returns
    ///
    /// Returns a finite upper bound when known.
    #[must_use]
    fn max_output_len(&self, decoder: &D, encoder: &E, input_len: usize) -> Option<usize>;

    /// Returns an upper bound for target units emitted by finishing state.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Source component owned by the engine.
    /// - `encoder`: Target component owned by the engine.
    ///
    /// # Returns
    ///
    /// Returns a finite upper bound when known.
    #[must_use]
    fn max_finish_output_len(&self, decoder: &D, encoder: &E) -> Option<usize>;

    /// Resets hook-owned and component-owned conversion state.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Source component owned by the engine.
    /// - `encoder`: Target component owned by the engine.
    fn reset(&mut self, decoder: &mut D, encoder: &mut E);

    /// Writes any retained output before new input is consumed.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Source component owned by the engine.
    /// - `encoder`: Target component owned by the engine.
    /// - `state`: Mutable state for this conversion call.
    ///
    /// # Returns
    ///
    /// Returns `Some(progress)` when conversion must stop, or `None` when the
    /// engine should continue converting input.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when pending output cannot be written under the
    /// concrete policy.
    fn drain_pending(
        &mut self,
        decoder: &mut D,
        encoder: &mut E,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> Result<Option<TranscodeProgress>, Self::Error>;

    /// Decodes one source value at the current input cursor.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Source component owned by the engine.
    /// - `state`: Mutable state for this conversion call.
    ///
    /// # Returns
    ///
    /// Returns either one decoded value or a request for more input.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when decoding fails under the concrete policy.
    fn decode_next(
        &mut self,
        decoder: &mut D,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> Result<ConvertDecodeResult<Value>, Self::Error>;

    /// Writes one decoded value through the target component.
    ///
    /// Implementations that return [`ConvertWriteResult::NeedOutput`] must
    /// retain `value` in hook-owned state so a later call can drain it before
    /// consuming more source input.
    ///
    /// # Parameters
    ///
    /// - `encoder`: Target component owned by the engine.
    /// - `value`: Decoded logical value to write.
    /// - `state`: Mutable state for this conversion call.
    ///
    /// # Returns
    ///
    /// Returns the write result for `value`.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when encoding fails under the concrete policy.
    fn write_value(
        &mut self,
        encoder: &mut E,
        value: Value,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> Result<ConvertWriteResult, Self::Error>;

    /// Converts one value from the current state cursors.
    ///
    /// The default implementation is the fixed converter algorithm: decode one
    /// source value, advance the input cursor, then encode that value. Hooks
    /// customize only the primitive decode/write policy points.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Source component owned by the engine.
    /// - `encoder`: Target component owned by the engine.
    /// - `state`: Mutable state for this conversion call.
    ///
    /// # Returns
    ///
    /// Returns `Some(progress)` when conversion must stop, or `None` when the
    /// engine should continue converting input.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when decoding or encoding fails under the concrete
    /// policy.
    #[inline]
    fn convert_next(
        &mut self,
        decoder: &mut D,
        encoder: &mut E,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> Result<Option<TranscodeProgress>, Self::Error> {
        let decoded = self.decode_next(decoder, state)?;
        let (value, consumed) = match decoded {
            ConvertDecodeResult::Decoded { value, consumed } => (value, consumed),
            ConvertDecodeResult::Skipped { consumed } => {
                state.advance_input(consumed.get());
                return Ok(None);
            }
            ConvertDecodeResult::NeedInput { additional, available } => {
                return Ok(Some(state.need_input_progress(additional, available)));
            }
        };
        state.advance_input(consumed.get());

        match self.write_value(encoder, value, state)? {
            ConvertWriteResult::Written { written } => {
                state.advance_output(written);
                Ok(None)
            }
            ConvertWriteResult::NeedOutput {
                additional,
                available,
                written,
            } => {
                state.advance_output(written);
                Ok(Some(state.need_output_progress(additional, available)))
            }
        }
    }

    /// Finishes hook-owned and component-owned output after EOF.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Source component owned by the engine.
    /// - `encoder`: Target component owned by the engine.
    /// - `output`: Complete output unit slice visible to the converter.
    /// - `output_index`: Absolute output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns finalization progress.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when finalization fails under the concrete policy.
    fn finish(
        &mut self,
        decoder: &mut D,
        encoder: &mut E,
        output: &mut [Output],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error>;
}
