/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Source-side reader object used by the converter coordinator.

use super::{
    buffered_convert_hooks::BufferedConvertHooks,
    buffered_decode_engine::BufferedDecodeEngine,
    convert_decode_attempt_result::ConvertDecodeAttemptResult,
    convert_error_of::ConvertProgressResult,
    convert_state::ConvertState,
    decode_step::DecodeStep,
};
use crate::Codec;

/// Source-side reader object used by the converter coordinator.
pub(super) struct SourceValueReader<'a, D, E, H>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    H: BufferedConvertHooks<D, E>,
{
    /// Source-side buffered decoder engine.
    engine: &'a mut BufferedDecodeEngine<D, H::DecodeHooks>,
    /// Conversion hooks used for error mapping.
    hooks: &'a H,
}

impl<'a, D, E, H> SourceValueReader<'a, D, E, H>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    H: BufferedConvertHooks<D, E>,
{
    /// Creates a source-side reader.
    ///
    /// # Type Parameters
    ///
    /// - `D`: Source codec used by the buffered decode engine.
    /// - `E`: Target codec; its value type must match `D::Value`.
    /// - `H`: Converter-level policy hooks shared by decode/encode steps.
    ///
    /// # Parameters
    ///
    /// - `engine`: Mutable reference to the shared source decode engine.
    /// - `hooks`: Converter hook object used to map decode errors.
    ///
    /// # Returns
    ///
    /// Returns a source reader bound to the supplied engine and hooks.
    #[inline(always)]
    pub(super) const fn new(engine: &'a mut BufferedDecodeEngine<D, H::DecodeHooks>, hooks: &'a H) -> Self {
        Self { engine, hooks }
    }

    /// Reads the next source value or a source-side stop condition.
    ///
    /// The method is the coordinator entry for source-side reads in the main
    /// conversion loop:
    ///
    /// 1. If the current slice has fewer than `codec.min_units_per_value()`,
    ///    it returns `DecodeStep::NeedInput`.
    /// 2. Otherwise it calls unchecked decode at the current input cursor.
    /// 3. Decode success produces `DecodeStep::Decoded`.
    /// 4. Decode failure is mapped through decode hooks and then normalized into
    ///    `DecodeStep` via `DecodeAction::into_step`.
    ///
    /// # Parameters
    ///
    /// - `state`: Current conversion state shared by decode/encode halves; used
    ///   for input cursor, minimum-width checks, and stop-progression accounting.
    ///
    /// # Returns
    ///
    /// Returns one decode attempt step that may be:
    /// - `DecodeStep::Decoded` with a recovered logical value and consumed
    ///   input count,
    /// - a continue signal for `NeedInput`,
    /// - or `NeedOutput` mapped from decode-policy results.
    ///
    /// # Errors
    ///
    /// Returns a converter-level error when:
    /// - source decode fails and cannot be converted into a decode policy action;
    /// - decode hook mapping fails and `H::map_decode_error` produces an
    ///   error.
    #[inline(always)]
    pub(super) fn read_next(
        &mut self,
        state: &ConvertState<'_, D::Unit, E::Unit>,
    ) -> ConvertDecodeAttemptResult<D, E, H> {
        let available = state.available_input();
        let min_units = self.engine.codec.min_units_per_value().get();
        if let Some(attempt) = state.need_input_for_min_units(min_units) {
            return Ok(attempt);
        }

        let input_index = state.input_cursor();
        let result = {
            // SAFETY: The state has at least `min_units_per_value()` units
            // available from `input_index`.
            unsafe { self.engine.decode_unchecked_at(state.input(), input_index) }
        };
        match result {
            Ok((value, consumed)) => {
                debug_assert!(
                    consumed.get() <= available,
                    "Codec::decode_unchecked consumed beyond available input",
                );
                Ok(DecodeStep::decoded(value, consumed, input_index))
            }
            Err(error) => {
                let context = state.decode_context();
                let action = match self.engine.handle_decode_error(error, context) {
                    Ok(action) => action,
                    Err(error) => return Err(self.hooks.map_decode_error(error)),
                };
                Ok(action.into_step(context.input_index, context.available))
            }
        }
    }

    /// Lets source-side finish hooks emit at most one final value.
    ///
    /// # Parameters
    ///
    /// - `decoded`: Caller-provided single-element scratch buffer. The function
    ///   passes this buffer to `BufferedDecodeEngine::finish`; on completion the
    ///   first slot may contain a final logical value.
    ///
    /// # Returns
    ///
    /// Returns conversion progress reporting whether a final value was emitted
    /// (`written == 1`) or the decoder only updated internal state.
    ///
    /// # Errors
    ///
    /// Returns a converter-level error when:
    /// - the source finish policy cannot be finalized, or
    /// - finish hook errors are mapped by `H::map_decode_error`.
    #[inline]
    pub(super) fn finish_one(&mut self, decoded: &mut [D::Value; 1]) -> ConvertProgressResult<D, E, H> {
        match self.engine.finish(decoded, 0) {
            Ok(finish) => Ok(finish),
            Err(error) => Err(self.hooks.map_decode_error(error)),
        }
    }
}
