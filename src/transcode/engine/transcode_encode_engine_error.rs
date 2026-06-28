// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Domain error reported by buffered encode engines.

use thiserror::Error;

/// Error reported by [`crate::TranscodeEncodeEngine`].
///
/// This type keeps codec lifecycle failures separate from policy hook failures.
/// Facade adapters may flatten both branches when they intentionally use the
/// same public domain error type for codec and hook failures.
///
/// # Type Parameters
///
/// - `C`: Codec encode error type.
/// - `H`: Encode hook error type.
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum TranscodeEncodeEngineError<C, H> {
    /// The wrapped codec failed while encoding an input value.
    #[error("codec encode error at input index {input_index}: {source}")]
    CodecEncode {
        /// Error returned by the wrapped codec.
        #[source]
        source: C,
        /// Absolute input index of the value being encoded.
        input_index: usize,
    },

    /// The wrapped codec failed while resetting encode state.
    #[error("codec encode reset error: {source}")]
    CodecReset {
        /// Error returned by [`crate::Codec::encode_reset`].
        #[source]
        source: C,
    },

    /// The wrapped codec failed while flushing encode state.
    #[error("codec encode flush error: {source}")]
    CodecFlush {
        /// Error returned by [`crate::Codec::encode_flush`].
        #[source]
        source: C,
    },

    /// The encode policy hook rejected or failed a value.
    #[error("{0}")]
    Hook(#[source] H),
}

impl<C, H> TranscodeEncodeEngineError<C, H> {
    /// Creates an error from a codec encode failure.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by the wrapped codec.
    /// - `input_index`: Absolute input index of the value being encoded.
    ///
    /// # Returns
    ///
    /// Returns an encode-engine codec encode error.
    #[inline(always)]
    #[must_use]
    pub const fn codec_encode(source: C, input_index: usize) -> Self {
        Self::CodecEncode {
            source,
            input_index,
        }
    }

    /// Creates an error from a codec encode-reset failure.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by [`crate::Codec::encode_reset`].
    ///
    /// # Returns
    ///
    /// Returns an encode-engine codec reset error.
    #[inline(always)]
    #[must_use]
    pub const fn codec_reset(source: C) -> Self {
        Self::CodecReset { source }
    }

    /// Creates an error from a codec encode-flush failure.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by [`crate::Codec::encode_flush`].
    ///
    /// # Returns
    ///
    /// Returns an encode-engine codec flush error.
    #[inline(always)]
    #[must_use]
    pub const fn codec_flush(source: C) -> Self {
        Self::CodecFlush { source }
    }

    /// Creates an error from a hook failure.
    ///
    /// # Parameters
    ///
    /// - `error`: Hook error.
    ///
    /// # Returns
    ///
    /// Returns an encode-engine hook error.
    #[inline(always)]
    #[must_use]
    pub const fn hook(error: H) -> Self {
        Self::Hook(error)
    }
}
