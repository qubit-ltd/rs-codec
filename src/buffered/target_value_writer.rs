/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Target-side writer object used by the converter coordinator.

use core::{
    marker::PhantomData,
    num::NonZeroUsize,
};

use super::{
    buffered_convert_hooks::BufferedConvertHooks,
    buffered_encode_engine::BufferedEncodeEngine,
    convert_encode_result::ConvertEncodeResult,
    convert_state::ConvertState,
    encode_context::EncodeContext,
    pending_encode_step::PendingEncodeStep,
    pending_value::PendingValue,
};
use crate::Codec;

/// Target-side writer object used by the converter coordinator.
pub(super) struct TargetValueWriter<'a, D, E, H, Input, Value, Output>
where
    D: Codec<Value, Input>,
    E: Codec<Value, Output>,
    H: BufferedConvertHooks<D, E, Input, Value, Output>,
    Input: Copy,
    Output: Copy,
{
    /// Target-side buffered encoder engine.
    engine: &'a mut BufferedEncodeEngine<E, H::EncodeHooks>,
    /// Conversion hooks used for error mapping.
    hooks: &'a H,
    /// Binds this helper to the source codec, value, and output unit types.
    marker: PhantomData<fn(D, Input, Value, Output)>,
}

impl<'a, D, E, H, Input, Value, Output> TargetValueWriter<'a, D, E, H, Input, Value, Output>
where
    D: Codec<Value, Input>,
    E: Codec<Value, Output>,
    H: BufferedConvertHooks<D, E, Input, Value, Output>,
    Input: Copy,
    Output: Copy,
{
    /// Creates a target-side writer.
    #[inline(always)]
    pub(super) const fn new(engine: &'a mut BufferedEncodeEngine<E, H::EncodeHooks>, hooks: &'a H) -> Self {
        Self {
            engine,
            hooks,
            marker: PhantomData,
        }
    }

    /// Encodes one pending source value at the current output cursor.
    #[inline(always)]
    pub(super) fn write_pending(
        &mut self,
        pending: PendingValue<Value>,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> ConvertEncodeResult<D, E, H, Input, Value, Output> {
        let input_index = pending.input_index();
        let output_index = state.output_cursor();
        let available = state.available_output();
        let plan = match self.engine.prepare_value::<Value, Output>(pending.value(), input_index) {
            Ok(plan) => plan,
            Err(error) => return Err(self.hooks.map_encode_error(error)),
        };
        let required = plan.max_output_units;
        if available < required {
            let additional = NonZeroUsize::new(required - available).expect("missing output is non-zero");
            return Ok(PendingEncodeStep::need_output(pending, additional, available));
        }

        let written = {
            let output = state.output_mut();
            let context = EncodeContext {
                input_value: pending.value(),
                input_index,
                plan_action: plan.action,
                output,
                output_index,
            };
            // SAFETY: The capacity check above proves the prepared output bound.
            match unsafe { self.engine.write_prepared_value(context) } {
                Ok(written) => written,
                Err(error) => return Err(self.hooks.map_encode_error(error)),
            }
        };
        debug_assert!(
            written <= required,
            "BufferedConvertEngine encode hook wrote beyond its prepared capacity bound",
        );
        Ok(PendingEncodeStep::written(written))
    }

    /// Finishes target-side hook-owned output.
    #[inline]
    pub(super) fn finish(
        &mut self,
        output: &mut [Output],
        output_index: usize,
    ) -> Result<
        super::transcode_progress::TranscodeProgress,
        <H as BufferedConvertHooks<D, E, Input, Value, Output>>::Error,
    > {
        match self.engine.finish::<Value, Output>(output, output_index) {
            Ok(finish) => Ok(finish),
            Err(error) => Err(self.hooks.map_encode_error(error)),
        }
    }
}
