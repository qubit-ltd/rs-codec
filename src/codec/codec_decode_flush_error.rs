// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Decode-flush lifecycle error marker.

use super::codec_decode_error::CodecDecodeError;

/// Error marker used when converting decode-flush lifecycle failures.
///
/// This wrapper keeps [`From`] based lifecycle error propagation available
/// without treating every codec decode error as if it happened at a synthetic
/// input index.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CodecDecodeFlushError<E> {
    /// Error returned by [`crate::Codec::decode_flush`].
    source: E,
}

impl<E> CodecDecodeFlushError<E> {
    /// Creates a decode-flush lifecycle error marker.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by [`crate::Codec::decode_flush`].
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
    /// Returns the decode-flush error by reference.
    #[inline(always)]
    #[must_use]
    pub const fn source(&self) -> &E {
        &self.source
    }

    /// Unwraps the marker into the wrapped codec error.
    ///
    /// # Returns
    ///
    /// Returns the wrapped decode-flush error.
    #[inline(always)]
    #[must_use]
    pub fn into_source(self) -> E {
        self.source
    }
}

impl<E> From<E> for CodecDecodeFlushError<E> {
    /// Wraps a decode-flush lifecycle error.
    #[inline(always)]
    fn from(source: E) -> Self {
        Self::new(source)
    }
}

impl<E> From<CodecDecodeFlushError<E>> for CodecDecodeError<E> {
    /// Converts a decode-flush lifecycle error into the generic adapter error.
    #[inline(always)]
    fn from(error: CodecDecodeFlushError<E>) -> Self {
        Self::decode_flush(error.into_source())
    }
}
