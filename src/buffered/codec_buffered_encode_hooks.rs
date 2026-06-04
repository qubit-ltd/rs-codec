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

impl<C> BufferedEncodeHooks<C> for CodecBufferedEncodeHooks
where
    C: Codec,
{
    type Error = CodecEncodeError<C::EncodeError>;
    type PlanAction = ();

    /// Prepares a conservative one-value encoding plan.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec for width calculation.
    /// - `_input_value`: Input value to be encoded.
    /// - `_input_index`: Absolute index of the input value.
    ///
    /// # Returns
    ///
    /// Returns an [`EncodePlan`] whose action is defaulted to unit.
    #[inline(always)]
    fn prepare_encode(
        &mut self,
        codec: &C,
        _input_value: &C::Value,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        Ok(EncodePlan::new(codec.max_units_per_value().get(), ()))
    }

    /// Writes one value by delegating to the wrapped codec.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec used for actual writing.
    /// - `context`: Encode context with prepared action and output cursor.
    ///
    /// # Returns
    ///
    /// Returns the number of units written.
    ///
    /// # Errors
    ///
    /// Returns a codec encode error when the codec fails.
    #[inline(always)]
    unsafe fn write_encode(
        &mut self,
        codec: &C,
        context: EncodeContext<'_, C::Value, C::Unit>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        // SAFETY: The engine checked that the prepared max-width capacity is
        // available before calling this method.
        unsafe { codec.encode_unchecked(context.input_value, context.output, context.output_index) }
            .map_err(|error| CodecEncodeError::encode(error, context.input_index))
    }

    /// Creates an invalid input index error.
    ///
    /// # Parameters
    ///
    /// - `_codec`: Low-level codec for context only.
    /// - `index`: Invalid absolute input index.
    /// - `input_len`: Length of the input value slice.
    ///
    /// # Returns
    ///
    /// Returns an encode invalid-input-index error.
    #[inline(always)]
    fn invalid_input_index(&mut self, _codec: &C, index: usize, input_len: usize) -> Self::Error {
        CodecEncodeError::invalid_input_index(index, input_len)
    }

    /// Creates an invalid output index error.
    ///
    /// # Parameters
    ///
    /// - `_codec`: Low-level codec for context only.
    /// - `index`: Invalid absolute output index.
    /// - `output_len`: Output slice length.
    ///
    /// # Returns
    ///
    /// Returns an encode invalid-output-index error.
    #[inline(always)]
    fn invalid_output_index(&mut self, _codec: &C, index: usize, output_len: usize) -> Self::Error {
        CodecEncodeError::invalid_output_index(index, output_len)
    }
}
