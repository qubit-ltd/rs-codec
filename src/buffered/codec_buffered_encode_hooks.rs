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
    encode_context::EncodeContext,
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
    type PlanAction = ();

    /// Prepares a conservative one-value encoding plan.
    #[inline(always)]
    fn prepare_encode(
        &mut self,
        codec: &C,
        _input_value: &Value,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        Ok(EncodePlan::new(codec.max_units_per_value().get(), ()))
    }

    /// Writes one value by delegating to the wrapped codec.
    #[inline(always)]
    unsafe fn write_encode(
        &mut self,
        codec: &C,
        context: EncodeContext<'_, Value, Unit, Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        // SAFETY: The engine checked that the prepared max-width capacity is
        // available before calling this method.
        unsafe { codec.encode_unchecked(context.input_value, context.output, context.output_index) }
            .map_err(|error| CodecEncodeError::encode(error, context.input_index))
    }

    /// Creates an invalid input index error.
    #[inline(always)]
    fn invalid_input_index(&mut self, _codec: &C, index: usize, input_len: usize) -> Self::Error {
        CodecEncodeError::invalid_input_index(index, input_len)
    }
}
