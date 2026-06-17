// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy hooks used by the default codec-backed buffered encoder.

use core::num::NonZeroUsize;

use super::super::engine::TranscodeEncodeHooks;
use super::super::{
    encode_context::EncodeContext,
    encode_plan::EncodePlan,
};
use crate::{
    Codec,
    CodecEncodeError,
};

/// Policy hooks for [`crate::CodecTranscodeEncoder`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(in crate::transcode) struct CodecTranscodeEncodeHooks;

impl<C> TranscodeEncodeHooks<C> for CodecTranscodeEncodeHooks
where
    C: Codec,
{
    type Error = CodecEncodeError<C::EncodeError>;
    type PlanAction = ();

    /// Prepares an exact one-value encoding plan.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec for width calculation.
    /// - `input_value`: Input value to be encoded.
    /// - `_input_index`: Absolute index of the input value.
    ///
    /// # Returns
    ///
    /// Returns an [`EncodePlan`] whose action is defaulted to unit.
    #[inline(always)]
    fn prepare_encode(
        &mut self,
        codec: &mut C,
        input_value: &C::Value,
        input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        if !codec.can_encode_value(input_value) {
            return Err(CodecEncodeError::unencodable_value(input_index));
        }
        Ok(EncodePlan::new(codec.encode_len(input_value).get(), ()))
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
        codec: &mut C,
        context: EncodeContext<'_, C::Value, C::Unit>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        // SAFETY: The engine checked that the prepared exact-value capacity is
        // available before calling this method.
        unsafe {
            codec.encode(
                context.input_value,
                context.output,
                context.output_index,
            )
        }
        .map(NonZeroUsize::get)
        .map_err(|error| CodecEncodeError::encode(error, context.input_index))
    }

    /// Maps reset errors into generic codec encode errors.
    #[inline(always)]
    fn map_encode_reset_error(
        &mut self,
        _codec: &mut C,
        error: C::EncodeError,
    ) -> Self::Error {
        CodecEncodeError::encode(error, 0)
    }
}
