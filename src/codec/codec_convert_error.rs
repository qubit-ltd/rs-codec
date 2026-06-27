// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Generic conversion error used by codec converter adapters.

use thiserror::Error;

use super::{
    codec_decode_error::CodecDecodeError,
    codec_encode_error::CodecEncodeError,
};

/// Domain error reported by codec-backed converter adapters.
///
/// A converter first decodes source units into a logical value and then encodes
/// that value into target units. This error keeps those two failure sources
/// explicit while framework buffer failures remain in
/// [`crate::TranscodeError`].
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
#[non_exhaustive]
pub enum CodecConvertError<D, E> {
    /// Source-unit decoding failed.
    #[error("codec conversion decode error: {0}")]
    Decode(#[source] CodecDecodeError<D>),

    /// Target-unit encoding failed.
    #[error("codec conversion encode error: {0}")]
    Encode(#[source] CodecEncodeError<E>),
}

impl<D, E> CodecConvertError<D, E> {
    /// Creates a conversion error from a decode-side failure.
    ///
    /// # Parameters
    ///
    /// - `source`: Decode error reported while reading source units.
    ///
    /// # Returns
    ///
    /// Returns a decode-side conversion error.
    #[inline(always)]
    #[must_use]
    pub const fn decode(source: CodecDecodeError<D>) -> Self {
        Self::Decode(source)
    }

    /// Creates a conversion error from an encode-side failure.
    ///
    /// # Parameters
    ///
    /// - `source`: Encode error reported while writing target units.
    ///
    /// # Returns
    ///
    /// Returns an encode-side conversion error.
    #[inline(always)]
    #[must_use]
    pub const fn encode(source: CodecEncodeError<E>) -> Self {
        Self::Encode(source)
    }
}
