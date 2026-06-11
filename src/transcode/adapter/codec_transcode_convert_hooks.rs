// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy hooks used by the default codec-backed buffered converter.

use super::{
    codec_transcode_decode_hooks::CodecTranscodeDecodeHooks,
    codec_transcode_encode_hooks::CodecTranscodeEncodeHooks,
};
use crate::{
    Codec,
    CodecConvertError,
    CodecDecodeError,
    CodecEncodeError,
    TranscodeConvertHooks,
};

/// Policy hooks for [`crate::CodecTranscodeConverter`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(in crate::transcode) struct CodecTranscodeConvertHooks;

impl CodecTranscodeConvertHooks {
    /// Creates codec-backed converter hooks.
    ///
    /// # Returns
    ///
    /// Returns stateless converter hooks.
    #[must_use]
    #[inline(always)]
    pub(in crate::transcode) const fn new() -> Self {
        Self
    }
}

impl<D, E> TranscodeConvertHooks<D, E> for CodecTranscodeConvertHooks
where
    D: Codec,
    E: Codec<Value = D::Value>,
{
    type ErrorContext = ();
    type DecodeError = CodecDecodeError<D::DecodeError>;
    type DecodeHooks = CodecTranscodeDecodeHooks;
    type EncodeError = CodecEncodeError<E::EncodeError>;
    type EncodeHooks = CodecTranscodeEncodeHooks;
    type Error = CodecConvertError<D::DecodeError, E::EncodeError>;

    /// Creates strict codec-backed decode hooks.
    ///
    /// # Parameters
    ///
    /// - `_decode_codec`: Source codec for reference only.
    /// - `_encode_codec`: Target codec for reference only.
    ///
    /// # Returns
    ///
    /// Returns decode hooks that map decode failures directly to codec decode
    /// errors.
    #[inline(always)]
    fn create_decode_hooks(
        &self,
        _decode_codec: &D,
        _encode_codec: &E,
    ) -> Self::DecodeHooks {
        CodecTranscodeDecodeHooks
    }

    /// Creates strict codec-backed encode hooks.
    ///
    /// # Parameters
    ///
    /// - `_decode_codec`: Source codec for reference only.
    /// - `_encode_codec`: Target codec for reference only.
    ///
    /// # Returns
    ///
    /// Returns encode hooks that map encode failures directly to codec encode
    /// errors.
    #[inline(always)]
    fn create_encode_hooks(
        &self,
        _decode_codec: &D,
        _encode_codec: &E,
    ) -> Self::EncodeHooks {
        CodecTranscodeEncodeHooks
    }

    /// Maps decoder errors into converter decode errors.
    ///
    /// # Parameters
    ///
    /// - `error`: Decode-side error from the source codec layer.
    ///
    /// # Returns
    ///
    /// Returns a converter-level decode error.
    #[inline(always)]
    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        CodecConvertError::decode(error)
    }

    /// Maps encoder errors into converter encode errors.
    ///
    /// # Parameters
    ///
    /// - `error`: Encode-side error from the target codec layer.
    ///
    /// # Returns
    ///
    /// Returns a converter-level encode error.
    #[inline]
    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        CodecConvertError::encode(error)
    }

    #[inline(always)]
    fn error_context(
        &self,
        _decode_codec: &D,
        _encode_codec: &E,
    ) -> Self::ErrorContext {
    }

    /// Resets stateless codec-backed converter hooks.
    ///
    /// # Parameters
    ///
    /// - `self`: Converter hooks instance.
    ///
    /// # Returns
    ///
    /// Returns unit `()`.
    #[inline(always)]
    fn reset(&mut self) {}
}
