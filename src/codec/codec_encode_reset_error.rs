// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Encode-reset lifecycle error marker.

use super::codec_encode_error::CodecEncodeError;

/// Error marker used when converting encode-reset lifecycle failures.
///
/// This wrapper keeps [`From`] based lifecycle error propagation available
/// without treating every codec encode error as if it happened at a synthetic
/// input index.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CodecEncodeResetError<E> {
    /// Error returned by [`crate::Codec::encode_reset`].
    source: E,
}

impl<E> CodecEncodeResetError<E> {
    /// Creates an encode-reset lifecycle error marker.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by [`crate::Codec::encode_reset`].
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
    /// Returns the encode-reset error by reference.
    #[inline(always)]
    #[must_use]
    pub const fn source(&self) -> &E {
        &self.source
    }

    /// Unwraps the marker into the wrapped codec error.
    ///
    /// # Returns
    ///
    /// Returns the wrapped encode-reset error.
    #[inline(always)]
    #[must_use]
    pub fn into_source(self) -> E {
        self.source
    }
}

impl<E> From<E> for CodecEncodeResetError<E> {
    /// Wraps an encode-reset lifecycle error.
    #[inline(always)]
    fn from(source: E) -> Self {
        Self::new(source)
    }
}

impl<E> From<CodecEncodeResetError<E>> for CodecEncodeError<E> {
    /// Converts an encode-reset lifecycle error into the generic adapter
    /// error.
    #[inline(always)]
    fn from(error: CodecEncodeResetError<E>) -> Self {
        Self::encode_reset(error.into_source())
    }
}
