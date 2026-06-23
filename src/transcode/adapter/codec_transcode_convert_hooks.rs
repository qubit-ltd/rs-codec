// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy hooks used by the default codec-backed buffered converter.

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
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) const fn new() -> Self {
        Self
    }
}

impl<D, E> TranscodeConvertHooks<D, E> for CodecTranscodeConvertHooks
where
    D: Codec,
    E: Codec<Value = D::Value>,
{
    type DecodeError = CodecDecodeError<D::DecodeError>;
    type EncodeError = CodecEncodeError<E::EncodeError>;
    type Error = CodecConvertError<D::DecodeError, E::EncodeError>;

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

    /// Runs stateless codec-backed converter cleanup before reset.
    ///
    /// # Parameters
    ///
    /// - `self`: Converter hooks instance.
    ///
    /// # Returns
    ///
    /// Returns unit `()`.
    #[inline(always)]
    fn before_reset(&mut self) {}
}
