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
    pub(super) const fn new() -> Self {
        Self
    }
}

impl<D, E, Value, InputUnit, OutputUnit> BufferedConvertHooks<D, E, InputUnit, Value, OutputUnit>
    for CodecBufferedConvertHooks
where
    D: Codec<Value, InputUnit>,
    E: Codec<Value, OutputUnit>,
    InputUnit: Copy,
    OutputUnit: Copy,
{
    type DecodeError = CodecDecodeError<D::DecodeError>;
    type DecodeHooks = CodecBufferedDecodeHooks;
    type EncodeError = CodecEncodeError<E::EncodeError>;
    type EncodeHooks = CodecBufferedEncodeHooks;
    type Error = CodecConvertError<D::DecodeError, E::EncodeError>;

    /// Creates strict codec-backed decode hooks.
    fn create_decode_hooks(&self, _decode_codec: &D, _encode_codec: &E) -> Self::DecodeHooks {
        CodecBufferedDecodeHooks
    }

    /// Creates strict codec-backed encode hooks.
    fn create_encode_hooks(&self, _decode_codec: &D, _encode_codec: &E) -> Self::EncodeHooks {
        CodecBufferedEncodeHooks
    }

    /// Maps decoder errors into converter decode errors.
    #[inline(always)]
    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        CodecConvertError::decode(error)
    }

    /// Maps encoder errors into converter encode errors.
    #[inline(always)]
    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        match error {
            CodecEncodeError::Encode { source, .. } => CodecConvertError::encode(source),
            CodecEncodeError::InvalidInputIndex { .. } => {
                unreachable!("codec converter encodes pending values from in-bounds source positions")
            }
        }
    }

    /// Creates an invalid source input index error.
    #[inline(always)]
    fn invalid_input_index(&self, _decode_codec: &D, index: usize, input_len: usize) -> Self::Error {
        CodecConvertError::decode(CodecDecodeError::invalid_input_index(index, input_len))
    }

    /// Resets stateless codec-backed converter hooks.
    #[inline(always)]
    fn reset(&mut self) {}
}
