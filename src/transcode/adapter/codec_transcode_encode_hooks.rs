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
    /// # Parameters
    ///
    /// - `codec`: Low-level codec for width calculation and writing.
    /// - `context`: Input value and output cursor.
    ///
    /// # Returns
    ///
    /// Returns whether the value was consumed or needs more output capacity.
    #[inline(always)]
    fn encode_value(
        &mut self,
        codec: &mut C,
        context: EncodeContext<'_, C::Value, C::Unit>,
    ) -> Result<EncodeOutcome, Self::Error> {
        if !codec.can_encode_value(context.input_value) {
            return Err(CodecEncodeError::unencodable_value(
                context.input_index,
            ));
        }
        let required = codec.encode_len(context.input_value);
        if context.available_output() < required.get() {
            return Ok(EncodeOutcome::need_output(required));
        }
        // SAFETY: The capacity check above reserves the exact value width.
        unsafe {
            codec.encode(
                context.input_value,
                context.output,
                context.output_index,
            )
        }
        .map(NonZeroUsize::get)
        .map(EncodeOutcome::consumed)
        .map_err(|error| CodecEncodeError::encode(error, context.input_index))
    }
}
