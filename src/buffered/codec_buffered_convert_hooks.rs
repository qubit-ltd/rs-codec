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
    buffered_encode_hooks::BufferedEncodeHooks,
    codec_buffered_decode_hooks::CodecBufferedDecodeHooks,
    codec_buffered_encode_hooks::CodecBufferedEncodeHooks,
};
use crate::{
    Codec,
    CodecConvertError,
    CodecDecodeError,
    CodecEncodeError,
    DecodeErrorInfo,
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

impl<D, E, Value, InputUnit> BufferedConvertHooks<D, E, InputUnit, Value> for CodecBufferedConvertHooks
where
    D: Codec<Value, InputUnit>,
    D::DecodeError: DecodeErrorInfo,
    InputUnit: Copy,
{
    type DecodeHooks = CodecBufferedDecodeHooks;
    type EncodeHooks = CodecBufferedEncodeHooks;
    type EncodeError<OutputUnit>
        = CodecEncodeError<E::EncodeError>
    where
        E: Codec<Value, OutputUnit>,
        OutputUnit: Copy;
    type Error<OutputUnit>
        = CodecConvertError<D::DecodeError, E::EncodeError>
    where
        E: Codec<Value, OutputUnit>,
        OutputUnit: Copy,
        CodecBufferedEncodeHooks: BufferedEncodeHooks<E, Value, OutputUnit, Error = Self::EncodeError<OutputUnit>>;

    /// Creates strict codec-backed decode hooks.
    fn create_decode_hooks(&self, _decoder: &D, _encoder: &E) -> Self::DecodeHooks {
        CodecBufferedDecodeHooks
    }

    /// Creates strict codec-backed encode hooks.
    fn create_encode_hooks(&self, _decoder: &D, _encoder: &E) -> Self::EncodeHooks {
        CodecBufferedEncodeHooks
    }

    /// Maps decoder errors into converter decode errors.
    #[inline(always)]
    fn map_decode_error<OutputUnit>(&self, error: CodecDecodeError<D::DecodeError>) -> Self::Error<OutputUnit>
    where
        E: Codec<Value, OutputUnit>,
        OutputUnit: Copy,
        CodecBufferedEncodeHooks: BufferedEncodeHooks<E, Value, OutputUnit, Error = Self::EncodeError<OutputUnit>>,
    {
        CodecConvertError::decode(error)
    }

    /// Maps encoder errors into converter encode errors.
    #[inline(always)]
    fn map_encode_error<OutputUnit>(&self, error: Self::EncodeError<OutputUnit>) -> Self::Error<OutputUnit>
    where
        E: Codec<Value, OutputUnit>,
        OutputUnit: Copy,
        CodecBufferedEncodeHooks: BufferedEncodeHooks<E, Value, OutputUnit, Error = Self::EncodeError<OutputUnit>>,
    {
        match error {
            CodecEncodeError::Encode { source, .. } => CodecConvertError::encode(source),
            CodecEncodeError::InvalidInputIndex { .. } => {
                unreachable!("codec converter encodes pending values from in-bounds source positions")
            }
        }
    }

    /// Resets stateless codec-backed converter hooks.
    #[inline(always)]
    fn reset(&mut self) {}
}
