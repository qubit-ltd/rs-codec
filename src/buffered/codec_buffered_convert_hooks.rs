/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Policy hooks used by the default codec-backed buffered converter.

use super::{
    buffered_convert_hooks::BufferedConvertHooks,
    codec_buffered_decode_hooks::CodecBufferedDecodeHooks,
    codec_buffered_encode_hooks::CodecBufferedEncodeHooks,
};
use crate::{
    Codec,
    CodecConvertError,
    CodecDecodeError,
    CodecEncodeError,
};

/// Policy hooks for [`super::CodecBufferedConverter`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(super) struct CodecBufferedConvertHooks;

impl CodecBufferedConvertHooks {
    /// Creates codec-backed converter hooks.
    ///
    /// # Returns
    ///
    /// Returns stateless converter hooks.
    #[must_use]
    #[inline(always)]
    pub(super) const fn new() -> Self {
        Self
    }
}

impl<D, E> BufferedConvertHooks<D, E> for CodecBufferedConvertHooks
where
    D: Codec,
    E: Codec<Value = D::Value>,
{
    type DecodeError = CodecDecodeError<D::DecodeError>;
    type DecodeHooks = CodecBufferedDecodeHooks;
    type EncodeError = CodecEncodeError<E::EncodeError>;
    type EncodeHooks = CodecBufferedEncodeHooks;
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
    /// Returns decode hooks that map decode failures directly to codec decode errors.
    #[inline(always)]
    fn create_decode_hooks(&self, _decode_codec: &D, _encode_codec: &E) -> Self::DecodeHooks {
        CodecBufferedDecodeHooks
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
    /// Returns encode hooks that map encode failures directly to codec encode errors.
    #[inline(always)]
    fn create_encode_hooks(&self, _decode_codec: &D, _encode_codec: &E) -> Self::EncodeHooks {
        CodecBufferedEncodeHooks
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
        match error {
            CodecEncodeError::Encode { source, .. } => CodecConvertError::encode(source),
            CodecEncodeError::InvalidInputIndex { .. } => {
                unreachable!("codec converter encodes pending values from in-bounds source positions")
            }
        }
    }

    /// Creates an invalid source input index error.
    ///
    /// # Parameters
    ///
    /// - `_decode_codec`: Source codec for which the caller-supplied index is invalid.
    /// - `index`: Invalid source input index.
    /// - `input_len`: Length of the source input slice.
    ///
    /// # Returns
    ///
    /// Returns a converter-level error describing the invalid source index.
    #[inline(always)]
    fn invalid_input_index(&self, _decode_codec: &D, index: usize, input_len: usize) -> Self::Error {
        CodecConvertError::decode(CodecDecodeError::invalid_input_index(index, input_len))
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
