// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Domain error reported by buffered decode engines.

use thiserror::Error;

/// Error reported by [`crate::TranscodeDecodeEngine`].
///
/// This type keeps codec lifecycle failures separate from policy hook failures.
/// Facade adapters may flatten both branches when they intentionally use the
/// same public domain error type for codec and hook failures.
///
/// # Type Parameters
///
/// - `C`: Codec decode error type.
/// - `H`: Decode hook error type.
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum TranscodeDecodeEngineError<C, H> {
    /// The wrapped codec failed while decoding input.
    #[error("codec decode error at input index {input_index}: {source}")]
    CodecDecode {
        /// Error returned by the wrapped codec.
        #[source]
        source: C,
        /// Absolute input index at which the engine called the wrapped codec.
        input_index: usize,
    },

    /// The wrapped codec failed while resetting decode state.
    #[error("codec decode reset error: {source}")]
    CodecReset {
        /// Error returned by [`crate::Codec::decode_reset`].
        #[source]
        source: C,
    },

    /// The wrapped codec failed while flushing decode state.
    #[error("codec decode flush error: {source}")]
    CodecFlush {
        /// Error returned by [`crate::Codec::decode_flush`].
        #[source]
        source: C,
    },

    /// The decode policy hook rejected or failed input.
    #[error("{0}")]
    Hook(#[source] H),
}

impl<C, H> TranscodeDecodeEngineError<C, H> {
    /// Creates an error from a codec decode failure.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by the wrapped codec.
    /// - `input_index`: Absolute input index used for the codec call.
    ///
    /// # Returns
    ///
    /// Returns a decode-engine codec decode error.
    #[inline(always)]
    #[must_use]
    pub const fn codec_decode(source: C, input_index: usize) -> Self {
        Self::CodecDecode {
            source,
            input_index,
        }
    }

    /// Creates an error from a codec decode-reset failure.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by [`crate::Codec::decode_reset`].
    ///
    /// # Returns
    ///
    /// Returns a decode-engine codec reset error.
    #[inline(always)]
    #[must_use]
    pub const fn codec_reset(source: C) -> Self {
        Self::CodecReset { source }
    }

    /// Creates an error from a codec decode-flush failure.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by [`crate::Codec::decode_flush`].
    ///
    /// # Returns
    ///
    /// Returns a decode-engine codec flush error.
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
    /// Returns a decode-engine hook error.
    #[inline(always)]
    #[must_use]
    pub const fn hook(error: H) -> Self {
        Self::Hook(error)
    }
}
