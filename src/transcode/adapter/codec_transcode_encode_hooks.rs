// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy hooks used by the default codec-backed buffered encoder.

use super::super::engine::TranscodeEncodeHooks;
use crate::{
    Codec,
    CodecEncodeError,
    EncodeUnencodableAction,
};

/// Policy hooks for [`crate::CodecTranscodeEncoder`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(in crate::transcode) struct CodecTranscodeEncodeHooks;

impl<C> TranscodeEncodeHooks<C> for CodecTranscodeEncodeHooks
where
    C: Codec,
{
    type Error = CodecEncodeError<C::EncodeError>;

    /// Rejects values outside the wrapped codec's encodable domain.
    #[inline(always)]
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut C,
        _value: &C::Value,
        input_index: usize,
    ) -> Result<EncodeUnencodableAction<C::Value>, Self::Error> {
        Err(CodecEncodeError::unencodable_value(input_index))
    }
}
