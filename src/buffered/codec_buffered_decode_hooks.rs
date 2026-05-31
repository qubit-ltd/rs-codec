/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Policy hooks used by the default codec-backed buffered decoder.

use super::{
    buffered_decode_hooks::BufferedDecodeHooks,
    decode_action::DecodeAction,
    decode_context::DecodeContext,
};
use crate::{
    Codec,
    CodecDecodeError,
    DecodeErrorInfo,
    DecodeFailure,
};

/// Policy hooks for [`super::CodecBufferedDecoder`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(super) struct CodecBufferedDecodeHooks;

impl<C, Unit, Value> BufferedDecodeHooks<C, Unit, Value> for CodecBufferedDecodeHooks
where
    C: Codec<Value, Unit>,
    C::DecodeError: DecodeErrorInfo,
    Unit: Copy,
{
    type Error = CodecDecodeError<C::DecodeError>;

    /// Converts codec decode failures into policy-free buffered actions.
    fn handle_decode_error(
        &mut self,
        _codec: &C,
        error: C::DecodeError,
        context: DecodeContext,
    ) -> Result<DecodeAction<Value>, Self::Error> {
        match error.failure() {
            DecodeFailure::Incomplete { required_total, .. } => Ok(DecodeAction::NeedInput { required_total }),
            DecodeFailure::Invalid { .. } => Err(CodecDecodeError::decode(error, context.input_index)),
        }
    }
}
