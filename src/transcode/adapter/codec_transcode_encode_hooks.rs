// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy hooks used by the default codec-backed buffered encoder.

use super::super::engine::EncodeContext;
use super::super::engine::EncodeUnencodableAction;
use super::super::engine::TranscodeEncodeHooks;
use crate::Codec;
use crate::TranscodeEncodeErrorOf;

/// Policy hooks for [`crate::CodecTranscodeEncoder`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(in crate::transcode) struct CodecTranscodeEncodeHooks;

impl<C> TranscodeEncodeHooks<C> for CodecTranscodeEncodeHooks
where
    C: Codec,
{
    /// Rejects values outside the wrapped codec's encodable domain.
    #[inline(always)]
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut C,
        _context: &EncodeContext<'_, C::Value>,
    ) -> Result<EncodeUnencodableAction<C::Value>, TranscodeEncodeErrorOf<C>> {
        Ok(EncodeUnencodableAction::Reject)
    }
}
