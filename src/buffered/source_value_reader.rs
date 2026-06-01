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
    buffered_encode_hooks::BufferedEncodeHooks,
    convert_decode_attempt_result::ConvertDecodeAttemptResult,
    convert_state::ConvertState,
    decode_step::DecodeStep,
};
use crate::Codec;

/// Source-side reader object used by the converter coordinator.
pub(super) struct SourceValueReader<'a, D, E, H, Input, Value>
where
    D: Codec<Value, Input>,
    H: BufferedConvertHooks<D, E, Input, Value>,
    Input: Copy,
{
    /// Source-side buffered decoder engine.
    engine: &'a mut BufferedDecodeEngine<D, H::DecodeHooks, Input>,
    /// Conversion hooks used for error mapping.
    hooks: &'a H,
    /// Binds this helper to the target codec and value types.
    marker: PhantomData<fn(E, Value)>,
}

impl<'a, D, E, H, Input, Value> SourceValueReader<'a, D, E, H, Input, Value>
where
    D: Codec<Value, Input>,
    H: BufferedConvertHooks<D, E, Input, Value>,
    Input: Copy,
{
    /// Creates a source-side reader.
    #[inline(always)]
    pub(super) const fn new(engine: &'a mut BufferedDecodeEngine<D, H::DecodeHooks, Input>, hooks: &'a H) -> Self {
        Self {
            engine,
            hooks,
            marker: PhantomData,
        }
    }

    /// Reads the next source value or source-side stop condition.
    #[inline(always)]
    pub(super) fn read_next<Output>(
        &mut self,
        state: &ConvertState<'_, Input, Output>,
    ) -> ConvertDecodeAttemptResult<D, E, H, Input, Value, Output>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
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
                    Err(error) => return Err(self.hooks.map_decode_error::<Output>(error)),
                };
                Ok(action.into_step(context.input_index, context.available))
            }
        }
    }

    /// Lets source-side finish hooks emit at most one final value.
    #[inline]
    pub(super) fn finish_one<Output>(
        &mut self,
        decoded: &mut [Value; 1],
    ) -> Result<super::transcode_progress::TranscodeProgress, <H as BufferedConvertHooks<D, E, Input, Value>>::Error<Output>>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Output: Copy,
    {
        match self.engine.finish::<Value>(decoded, 0) {
            Ok(finish) => Ok(finish),
            Err(error) => Err(self.hooks.map_decode_error::<Output>(error)),
        }
    }
}
