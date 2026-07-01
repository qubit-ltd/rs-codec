// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy hooks used by the default codec-backed buffered decoder.

use super::super::engine::{
    DecodeContext,
    DecodeInvalidAction,
    TranscodeDecodeHooks,
};
use core::num::NonZeroUsize;

use crate::{
    Codec,
    TranscodeDecodeError,
};

/// Policy hooks for [`crate::CodecTranscodeDecoder`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(in crate::transcode) struct CodecTranscodeDecodeHooks;

impl<C> TranscodeDecodeHooks<C> for CodecTranscodeDecodeHooks
where
    C: Codec,
{
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
    /// Returns the strict invalid-decode policy action.
    #[inline(always)]
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut C,
        _error: &C::DecodeError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<C::Value>, TranscodeDecodeError<C>> {
        Ok(DecodeInvalidAction::Reject)
    }
}
