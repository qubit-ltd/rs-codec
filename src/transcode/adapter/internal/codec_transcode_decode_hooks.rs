// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy hooks used by the default codec-backed buffered decoder.

use core::num::NonZeroUsize;

use super::super::super::engine::DecodeContext;
use super::super::super::engine::DecodeInvalidAction;
use super::super::super::engine::TranscodeDecodeHooks;
use crate::CapacityError;
use crate::Codec;
use crate::TranscodeDecodeErrorOf;

/// Policy hooks for [`crate::CodecTranscodeDecoder`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(in crate::transcode) struct CodecTranscodeDecodeHooks;

impl<C> TranscodeDecodeHooks<C> for CodecTranscodeDecodeHooks
where
    C: Codec,
{
    /// Returns the strict decode capacity, including one possible EOF-tail
    /// slot.
    ///
    /// Rejected tails may still require an output slot before policy dispatch.
    /// Reset and finish output are counted independently by the engine.
    ///
    /// # Parameters
    ///
    /// - `_codec`: Codec whose non-zero minimum width is validated by the
    ///   engine.
    /// - `input_len`: Source units planned for the streaming phase.
    ///
    /// # Returns
    ///
    /// Returns the source length divided by the minimum width, rounded up.
    #[inline]
    fn max_transcode_output_len(&self, _codec: &C, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len.div_ceil(C::MIN_UNITS_PER_VALUE))
    }

    /// Converts codec decode failures into strict buffered decode errors.
    ///
    /// # Parameters
    ///
    /// - `_codec`: Low-level codec instance.
    /// - `_error`: Invalid domain error produced by the low-level codec.
    /// - `_consumed`: Invalid units that a non-strict policy may consume.
    /// - `_context`: Decoding context carrying input position.
    ///
    /// # Returns
    ///
    /// Returns the strict invalid-decode policy action.
    ///
    /// # Errors
    ///
    /// This strict hook never returns an error; it always rejects invalid
    /// input.
    #[inline]
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut C,
        _error: &C::DecodeError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<C::Value>, TranscodeDecodeErrorOf<C>> {
        Ok(DecodeInvalidAction::Reject)
    }
}
