/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Policy hooks used by buffered encoder engines.

use super::{
    encode_plan::EncodePlan,
    transcode_progress::TranscodeProgress,
};
use crate::{
    Codec,
    EncodeErrorFactory,
};

/// Policy hooks for [`crate::BufferedEncodeEngine`].
///
/// Hooks own policy state, such as replacement or ignore behavior, but not the
/// codec or engine cursor state. The engine passes the codec into hook methods
/// when policy code needs codec metadata or one-value encode operations.
///
/// # Type Parameters
///
/// - `C`: Low-level codec owned by the engine.
/// - `Value`: Logical input value type.
/// - `Unit`: Encoded output unit type.
pub trait BufferedEncodeHooks<C, Value, Unit>
where
    C: Codec<Value, Unit>,
    Unit: Copy,
{
    /// Error type returned by the buffered encoder.
    type Error: EncodeErrorFactory<C>;

    /// Concrete payload stored in [`EncodePlan::payload`].
    type PlanPayload;

    /// Returns the maximum output units needed for `input_len` values.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `input_len`: Number of input values the caller plans to encode.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound derived from the codec's
    /// [`Codec::max_units_per_value`], or `None` on overflow.
    #[must_use]
    #[inline(always)]
    fn max_output_len(&self, codec: &C, input_len: usize) -> Option<usize> {
        input_len.checked_mul(codec.max_units_per_value().get())
    }

    /// Returns an upper bound for units emitted by finishing hook-owned state.
    ///
    /// `finish` never receives more input values. Implementations must only
    /// report output derived from hook-owned state that remains after the caller
    /// has supplied all input.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    ///
    /// # Returns
    ///
    /// Returns a finite upper bound when known.
    #[must_use]
    #[inline(always)]
    fn max_finish_output_len(&self, _codec: &C) -> Option<usize> {
        Some(0)
    }

    /// Prepares an encoding plan for one input value.
    ///
    /// This method must not write output. It decides the output capacity bound
    /// needed before [`write_encode`](Self::write_encode) may be called and
    /// returns an implementation-specific plan payload.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `input_value`: Input value being encoded.
    /// - `input_index`: Absolute input index of `value`.
    ///
    /// # Returns
    ///
    /// Returns the write plan for `value`.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when this value cannot be encoded under the hook
    /// policy.
    fn prepare_encode(
        &mut self,
        codec: &C,
        input_value: &Value,
        input_index: usize,
    ) -> Result<EncodePlan<Self::PlanPayload>, Self::Error>;

    /// Writes one input value according to a previously prepared plan.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `input_value`: Input value being encoded.
    /// - `input_index`: Absolute input index of `value`.
    /// - `plan_payload`: Plan payload returned by [`prepare_encode`](Self::prepare_encode).
    /// - `output`: Complete output unit slice visible to the encoder.
    /// - `output_index`: Start position in `output` where writing begins.
    ///
    /// # Returns
    ///
    /// Returns the number of output units written.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when writing fails under the hook policy.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that at least the corresponding
    /// [`EncodePlan::max_output_units`] units are writable from `output_index`.
    unsafe fn write_encode(
        &mut self,
        codec: &C,
        input_value: &Value,
        input_index: usize,
        plan_payload: Self::PlanPayload,
        output: &mut [Unit],
        output_index: usize,
    ) -> Result<usize, Self::Error>;

    /// Finishes hook-owned state and writes any retained output units.
    ///
    /// The default implementation is a no-op for stateless encode hooks.
    /// Stateful hooks may emit final units such as reset sequences, checksums, or
    /// trailers. If `output` does not provide enough capacity, return
    /// [`crate::TranscodeStatus::NeedOutput`] and keep the unwritten state for a
    /// later `finish` call.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `output`: Complete output unit slice visible to the hook.
    /// - `output_index`: Absolute output unit index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns progress for units written by finalization.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when hook-owned state cannot be finalized.
    #[inline(always)]
    fn finish(
        &mut self,
        codec: &C,
        output: &mut [Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if output_index > output.len() {
            let additional = self.max_finish_output_len(codec).unwrap_or(1).max(1);
            return Ok(TranscodeProgress::need_output(output_index, additional, 0, 0, 0));
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    /// Resets hook-owned policy state.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    #[inline(always)]
    fn reset(&mut self, _codec: &C) {}
}
