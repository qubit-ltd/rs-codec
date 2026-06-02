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

use core::num::NonZeroUsize;

use super::{
    buffered_convert_hooks::BufferedConvertHooks,
    buffered_encode_engine::BufferedEncodeEngine,
    convert_encode_result::ConvertEncodeResult,
    convert_error_of::ConvertProgressResult,
    convert_state::ConvertState,
    encode_context::EncodeContext,
    pending_encode_step::PendingEncodeStep,
    pending_value::PendingValue,
};
use crate::Codec;

/// Target-side writer object used by the converter coordinator.
pub(super) struct TargetValueWriter<'a, D, E, H>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    H: BufferedConvertHooks<D, E>,
{
    /// Target-side buffered encoder engine.
    engine: &'a mut BufferedEncodeEngine<E, H::EncodeHooks>,
    /// Conversion hooks used for error mapping.
    hooks: &'a H,
}

impl<'a, D, E, H> TargetValueWriter<'a, D, E, H>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    H: BufferedConvertHooks<D, E>,
{
    /// Creates a target-side writer.
    ///
    /// # Type Parameters
    ///
    /// - `D`: Source codec used by the converter.
    /// - `E`: Target codec used by the converter.
    /// - `H`: Converter-level hook aggregator.
    ///
    /// # Parameters
    ///
    /// - `engine`: Mutable reference to the shared target encode engine.
    /// - `hooks`: Converter hooks used to map encode errors.
    ///
    /// # Returns
    ///
    /// Returns a target-side writer bound to the provided engine.
    #[inline(always)]
    pub(super) const fn new(engine: &'a mut BufferedEncodeEngine<E, H::EncodeHooks>, hooks: &'a H) -> Self {
        Self { engine, hooks }
    }

    /// Encodes one pending source value at the current output cursor.
    ///
    /// # Parameters
    ///
    /// - `pending`: Decoded source value waiting for target encoding.
    /// - `state`: Current conversion state exposing output cursor and capacity.
    ///
    /// # Returns
    ///
    /// - Returns `Ok(PendingEncodeStep::written)` when the value is fully encoded.
    /// - Returns `Ok(PendingEncodeStep::need_output)` when output capacity is
    ///   insufficient and more output units are required.
    ///
    /// # Errors
    ///
    /// Returns converter-level errors when source plan or encode preparation fails.
    #[inline(always)]
    pub(super) fn write_pending(
        &mut self,
        pending: PendingValue<D::Value>,
        state: &mut ConvertState<'_, D::Unit, E::Unit>,
    ) -> ConvertEncodeResult<D, E, H> {
        let input_index = pending.input_index();
        let output_index = state.output_cursor();
        let available = state.available_output();
        let plan = match self.engine.prepare_value(pending.value(), input_index) {
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
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output unit slice visible to the encoder.
    /// - `output_index`: Absolute output index where final output starts.
    ///
    /// # Returns
    ///
    /// Returns finalization [`TranscodeProgress`].
    ///
    /// # Errors
    ///
    /// Returns a converter-level error when target finalize hooks fail.
    #[inline]
    pub(super) fn finish(&mut self, output: &mut [E::Unit], output_index: usize) -> ConvertProgressResult<D, E, H> {
        match self.engine.finish(output, output_index) {
            Ok(finish) => Ok(finish),
            Err(error) => Err(self.hooks.map_encode_error(error)),
        }
    }
}
