// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy hooks used by the default codec-backed buffered encoder.

use core::num::NonZeroUsize;

use super::super::encode_context::EncodeContext;
use super::super::engine::TranscodeEncodeHooks;
use crate::{
    Codec,
    CodecEncodeError,
    EncodeOutcome,
};

/// Policy hooks for [`crate::CodecTranscodeEncoder`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(in crate::transcode) struct CodecTranscodeEncodeHooks;

impl<C> TranscodeEncodeHooks<C> for CodecTranscodeEncodeHooks
where
    C: Codec,
{
    type Error = CodecEncodeError<C::EncodeError>;

    /// Encodes one value through the wrapped codec.
    ///
    /// For fixed-width codecs (`MIN_UNITS_PER_VALUE == MAX_UNITS_PER_VALUE`)
    /// the capacity check uses the compile-time constant directly, avoiding a
    /// virtual `encode_len` call per value.
    #[inline(always)]
    fn encode_value(
        &mut self,
        codec: &mut C,
        context: EncodeContext<'_, C::Value, C::Unit>,
    ) -> Result<EncodeOutcome, Self::Error> {
        if !codec.can_encode_value(context.input_value()) {
            return Err(CodecEncodeError::unencodable_value(context.input_index()));
        }
        // Fixed-width codecs: skip the encode_len call and use the constant.
        // Variable-width codecs: query the exact length for this value.
        let required = if C::MIN_UNITS_PER_VALUE == C::MAX_UNITS_PER_VALUE {
            C::MAX_UNITS_PER_VALUE
        } else {
            codec.encode_len(context.input_value())
        };
        if context.available_output() < required.get() {
            return Ok(EncodeOutcome::need_output(required));
        }
        // Destructure to satisfy the borrow checker: input_value() and
        // output() cannot both be called on the same context expression.
        let (value, input_index, output, output_index) = context.into_parts();
        // SAFETY: The capacity check above reserves the exact value width.
        unsafe { codec.encode(value, output, output_index) }
            .map(NonZeroUsize::get)
            .map(EncodeOutcome::consumed)
            .map_err(|error| CodecEncodeError::encode(error, input_index))
    }
}
