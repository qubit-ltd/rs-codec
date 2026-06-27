// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Domain error reported by buffered convert engines.

use thiserror::Error;

/// Error reported by [`crate::TranscodeConvertEngine`].
///
/// Conversion owns both a decode side and an encode side. This type preserves
/// which side produced the domain error while framework buffer failures remain
/// in [`crate::TranscodeError`].
///
/// # Type Parameters
///
/// - `D`: Decode-side engine error type.
/// - `E`: Encode-side engine error type.
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
#[non_exhaustive]
pub enum TranscodeConvertEngineError<D, E> {
    /// Source decoding failed.
    #[error("decode side failed: {0}")]
    Decode(#[source] D),

    /// Target encoding failed.
    #[error("encode side failed: {0}")]
    Encode(#[source] E),
}

impl<D, E> TranscodeConvertEngineError<D, E> {
    /// Creates a converter error from the decode side.
    ///
    /// # Parameters
    ///
    /// - `error`: Decode-side engine error.
    ///
    /// # Returns
    ///
    /// Returns a converter decode error.
    #[inline(always)]
    #[must_use]
    pub const fn decode(error: D) -> Self {
        Self::Decode(error)
    }

    /// Creates a converter error from the encode side.
    ///
    /// # Parameters
    ///
    /// - `error`: Encode-side engine error.
    ///
    /// # Returns
    ///
    /// Returns a converter encode error.
    #[inline(always)]
    #[must_use]
    pub const fn encode(error: E) -> Self {
        Self::Encode(error)
    }
}
