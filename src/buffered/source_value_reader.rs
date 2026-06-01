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

use core::marker::PhantomData;

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
pub(super) struct SourceValueReader<'a, D, E, H, Input, Value, Output>
where
    D: Codec<Value, Input>,
    E: Codec<Value, Output>,
    H: BufferedConvertHooks<D, E, Input, Value, Output>,
    Input: Copy,
    Output: Copy,
{
    /// Source-side buffered decoder engine.
    engine: &'a mut BufferedDecodeEngine<D, H::DecodeHooks, Input, Value>,
    /// Conversion hooks used for error mapping.
    hooks: &'a H,
    /// Binds this helper to the target codec, value, and output unit types.
    marker: PhantomData<fn(E, Value, Output)>,
}

impl<'a, D, E, H, Input, Value, Output> SourceValueReader<'a, D, E, H, Input, Value, Output>
where
    D: Codec<Value, Input>,
    E: Codec<Value, Output>,
    H: BufferedConvertHooks<D, E, Input, Value, Output>,
    Input: Copy,
    Output: Copy,
{
    /// Creates a source-side reader.
    #[inline(always)]
    pub(super) const fn new(
        engine: &'a mut BufferedDecodeEngine<D, H::DecodeHooks, Input, Value>,
        hooks: &'a H,
    ) -> Self {
        Self {
            engine,
            hooks,
            marker: PhantomData,
        }
    }

    /// Reads the next source value or source-side stop condition.
    #[inline(always)]
    pub(super) fn read_next(
        &mut self,
        state: &ConvertState<'_, Input, Output>,
    ) -> ConvertDecodeAttemptResult<D, E, H, Input, Value, Output> {
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
    #[inline]
    pub(super) fn finish_one(
        &mut self,
        decoded: &mut [Value; 1],
    ) -> ConvertProgressResult<D, E, H, Input, Value, Output> {
        match self.engine.finish(decoded, 0) {
            Ok(finish) => Ok(finish),
            Err(error) => Err(self.hooks.map_decode_error(error)),
        }
    }
}
