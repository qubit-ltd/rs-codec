// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Decode-reset lifecycle error marker.

use super::codec_decode_error::CodecDecodeError;

/// Error marker used when converting decode-reset lifecycle failures.
///
/// This wrapper keeps [`From`] based lifecycle error propagation available
/// without treating every codec decode error as if it happened at a synthetic
/// input index.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CodecDecodeResetError<E> {
    /// Error returned by [`crate::Codec::decode_reset`].
    source: E,
}

impl<E> CodecDecodeResetError<E> {
    /// Creates a decode-reset lifecycle error marker.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by [`crate::Codec::decode_reset`].
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
    /// Returns the decode-reset error by reference.
    #[inline(always)]
    #[must_use]
    pub const fn source(&self) -> &E {
        &self.source
    }

    /// Unwraps the marker into the wrapped codec error.
    ///
    /// # Returns
    ///
    /// Returns the wrapped decode-reset error.
    #[inline(always)]
    #[must_use]
    pub fn into_source(self) -> E {
        self.source
    }
}

impl<E> From<E> for CodecDecodeResetError<E> {
    /// Wraps a decode-reset lifecycle error.
    #[inline(always)]
    fn from(source: E) -> Self {
        Self::new(source)
    }
}

impl<E> From<CodecDecodeResetError<E>> for CodecDecodeError<E> {
    /// Converts a decode-reset lifecycle error into the generic adapter error.
    #[inline(always)]
    fn from(error: CodecDecodeResetError<E>) -> Self {
        Self::decode_reset(error.into_source())
    }
}
