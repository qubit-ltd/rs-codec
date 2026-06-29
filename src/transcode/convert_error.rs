// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Domain error used by transcode converters.

use thiserror::Error;

use crate::Codec;

use super::transcode_error::TranscodeError;

/// Domain error produced by one side of a converter pipeline.
///
/// The generic parameters are domain error types, not codec types. Framework
/// buffer failures remain in the outer [`TranscodeError`].
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
#[non_exhaustive]
pub enum ConvertError<D, E> {
    /// Source decoding failed.
    #[error("decode side failed: {0}")]
    Decode(#[source] D),

    /// Target encoding failed.
    #[error("encode side failed: {0}")]
    Encode(#[source] E),
}

impl<D, E> ConvertError<D, E> {
    /// Creates a decode-side converter error.
    #[inline(always)]
    #[must_use]
    pub const fn decode(source: D) -> Self {
        Self::Decode(source)
    }

    /// Creates an encode-side converter error.
    #[inline(always)]
    #[must_use]
    pub const fn encode(source: E) -> Self {
        Self::Encode(source)
    }
}

/// Intermediate error used by codec-backed converters.
pub type TranscodeConvertError<D, E> =
    TranscodeError<ConvertError<<D as Codec>::DecodeError, <E as Codec>::EncodeError>>;
