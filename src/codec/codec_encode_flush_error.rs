// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Encode-flush lifecycle error marker.

use super::codec_encode_error::CodecEncodeError;

/// Error marker used when converting encode-flush lifecycle failures.
///
/// This wrapper keeps [`From`] based lifecycle error propagation available
/// without treating every codec encode error as if it happened at a synthetic
/// input index.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CodecEncodeFlushError<E> {
    /// Error returned by [`crate::Codec::encode_flush`].
    source: E,
}

impl<E> CodecEncodeFlushError<E> {
    /// Creates an encode-flush lifecycle error marker.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by [`crate::Codec::encode_flush`].
    ///
    /// # Returns
    ///
    /// Returns a lifecycle error marker carrying `source`.
    #[inline(always)]
    #[must_use]
    pub const fn new(source: E) -> Self {
        Self { source }
    }

    /// Returns the wrapped codec error.
    ///
    /// # Returns
    ///
    /// Returns the encode-flush error by reference.
    #[inline(always)]
    #[must_use]
    pub const fn source(&self) -> &E {
        &self.source
    }

    /// Unwraps the marker into the wrapped codec error.
    ///
    /// # Returns
    ///
    /// Returns the wrapped encode-flush error.
    #[inline(always)]
    #[must_use]
    pub fn into_source(self) -> E {
        self.source
    }
}

impl<E> From<E> for CodecEncodeFlushError<E> {
    /// Wraps an encode-flush lifecycle error.
    #[inline(always)]
    fn from(source: E) -> Self {
        Self::new(source)
    }
}

impl<E> From<CodecEncodeFlushError<E>> for CodecEncodeError<E> {
    /// Converts an encode-flush lifecycle error into the generic adapter error.
    #[inline(always)]
    fn from(error: CodecEncodeFlushError<E>) -> Self {
        Self::encode_flush(error.into_source())
    }
}
