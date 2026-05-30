/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Policy hooks used by the default codec-backed buffered encoder.

use super::{
    buffered_encode_hooks::BufferedEncodeHooks,
    encode_plan::EncodePlan,
};
use crate::{
    Codec,
    CodecEncodeError,
};

/// Policy hooks for [`super::CodecBufferedEncoder`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(super) struct CodecBufferedEncodeHooks;

impl<C, Value, Unit> BufferedEncodeHooks<C, Value, Unit> for CodecBufferedEncodeHooks
where
    C: Codec<Value, Unit>,
    Unit: Copy,
{
    type Error = CodecEncodeError<C::EncodeError>;
    type PlanPayload = ();

    /// Prepares a conservative one-value encoding plan.
    #[inline(always)]
    fn prepare_encode(
        &mut self,
        codec: &C,
        _input_value: &Value,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanPayload>, Self::Error> {
        Ok(EncodePlan::new(codec.max_units_per_value().get(), ()))
    }

    /// Writes one value by delegating to the wrapped codec.
    #[inline(always)]
    unsafe fn write_encode(
        &mut self,
        codec: &C,
        input_value: &Value,
        input_index: usize,
        _plan_payload: Self::PlanPayload,
        output: &mut [Unit],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        // SAFETY: The engine checked that the prepared max-width capacity is
        // available before calling this method.
        unsafe { codec.encode_unchecked(input_value, output, output_index) }
            .map_err(|error| CodecEncodeError::encode(error, input_index))
    }
}
