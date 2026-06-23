// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy hooks used by the default codec-backed buffered decoder.

use super::super::engine::TranscodeDecodeHooks;
use super::super::{
    decode_context::DecodeContext,
    decode_invalid_action::DecodeInvalidAction,
};
use core::num::NonZeroUsize;

use crate::{
    Codec,
    CodecDecodeError,
};

/// Policy hooks for [`crate::CodecTranscodeDecoder`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(in crate::transcode) struct CodecTranscodeDecodeHooks;

impl<C> TranscodeDecodeHooks<C> for CodecTranscodeDecodeHooks
where
    C: Codec,
{
    type Error = CodecDecodeError<C::DecodeError>;

    /// Converts codec decode failures into strict buffered decode errors.
    ///
    /// # Parameters
    ///
    /// - `_codec`: Low-level codec instance.
    /// - `error`: Invalid domain error produced by the low-level codec.
    /// - `_consumed`: Invalid units that a non-strict policy may consume.
    /// - `context`: Decoding context carrying input position.
    ///
    /// # Returns
    ///
    /// Returns a convert status action wrapped as `CodecDecodeError`.
    #[inline(always)]
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut C,
        error: C::DecodeError,
        _consumed: Option<NonZeroUsize>,
        context: DecodeContext,
    ) -> Result<DecodeInvalidAction<C::Value>, Self::Error> {
        Err(CodecDecodeError::decode(error, context.input_index()))
    }
}
