// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy hooks used by the default codec-backed buffered decoder.

use super::{
    transcode_decode_hooks::TranscodeDecodeHooks,
};
use super::super::{decode_action::DecodeAction, decode_context::DecodeContext};
use crate::{Codec, CodecDecodeError};

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
    /// - `error`: Decode error produced by the low-level codec.
    /// - `context`: Decoding context carrying input position.
    ///
    /// # Returns
    ///
    /// Returns a convert status action wrapped as `CodecDecodeError`.
    #[inline(always)]
    fn handle_decode_error(
        &mut self,
        _codec: &mut C,
        error: C::DecodeError,
        context: DecodeContext,
    ) -> Result<DecodeAction<C::Value>, Self::Error> {
        Err(CodecDecodeError::decode(error, context.input_index))
    }

    /// Maps flush errors into generic codec decode errors.
    #[inline(always)]
    fn map_decode_flush_error(&mut self, _codec: &mut C, error: C::DecodeError) -> Self::Error {
        CodecDecodeError::decode(error, 0)
    }

    /// Creates an invalid input index error.
    ///
    /// # Parameters
    ///
    /// - `_codec`: Low-level codec instance.
    /// - `index`: Invalid absolute input index.
    /// - `input_len`: Input slice length.
    ///
    /// # Returns
    ///
    /// Returns a codec decode error describing the invalid index.
    #[inline(always)]
    fn invalid_input_index(
        &mut self,
        _codec: &mut C,
        index: usize,
        input_len: usize,
    ) -> Self::Error {
        CodecDecodeError::invalid_input_index(index, input_len)
    }

    /// Creates an invalid output index error.
    ///
    /// # Parameters
    ///
    /// - `_codec`: Low-level codec instance.
    /// - `index`: Invalid absolute output index.
    /// - `output_len`: Output slice length.
    ///
    /// # Returns
    ///
    /// Returns a codec decode error describing the invalid output index.
    #[inline(always)]
    fn invalid_output_index(
        &mut self,
        _codec: &mut C,
        index: usize,
        output_len: usize,
    ) -> Self::Error {
        CodecDecodeError::invalid_output_index(index, output_len)
    }
}
