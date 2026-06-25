// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Generic encode error used by codec-backed encoder adapters.

use thiserror::Error;

/// Error reported by codec-backed buffered encoder adapters.
///
/// The wrapped codec remains responsible for domain-specific encode failures.
/// This type adds adapter-level domain failures that cannot be represented by
/// the wrapped codec itself, such as a value outside the codec's encodable
/// domain. Buffer index and capacity failures are represented by
/// [`crate::TranscodeError`].
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum CodecEncodeError<E> {
    /// The wrapped codec reported an encode error.
    #[error("codec encode error at input index {input_index}: {source}")]
    Encode {
        /// Error returned by the wrapped codec.
        #[source]
        source: E,
        /// Absolute input index of the value being encoded.
        input_index: usize,
    },

    /// The wrapped codec reported an error while resetting encode state.
    #[error("codec encode reset error: {source}")]
    EncodeReset {
        /// Error returned by [`crate::Codec::encode_reset`].
        #[source]
        source: E,
    },

    /// The wrapped codec reported an error while flushing encode state.
    #[error("codec encode flush error: {source}")]
    EncodeFlush {
        /// Error returned by [`crate::Codec::encode_flush`].
        #[source]
        source: E,
    },

    /// The wrapped codec cannot represent the input value.
    #[error("unencodable value at input index {input_index}")]
    UnencodableValue {
        /// Absolute input index of the value being encoded.
        input_index: usize,
    },
}

impl<E> CodecEncodeError<E> {
    /// Creates an error wrapping a codec-specific encode error.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by the wrapped codec.
    /// - `input_index`: Absolute input index of the value being encoded.
    ///
    /// # Returns
    ///
    /// Returns a codec encode error wrapper.
    #[inline(always)]
    #[must_use]
    pub const fn encode(source: E, input_index: usize) -> Self {
        Self::Encode {
            source,
            input_index,
        }
    }

    /// Creates an error wrapping a codec-specific encode-reset error.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by [`crate::Codec::encode_reset`].
    ///
    /// # Returns
    ///
    /// Returns a codec encode-reset error wrapper.
    #[inline(always)]
    #[must_use]
    pub const fn encode_reset(source: E) -> Self {
        Self::EncodeReset { source }
    }

    /// Creates an error wrapping a codec-specific encode-flush error.
    ///
    /// # Parameters
    ///
    /// - `source`: Error returned by [`crate::Codec::encode_flush`].
    ///
    /// # Returns
    ///
    /// Returns a codec encode-flush error wrapper.
    #[inline(always)]
    #[must_use]
    pub const fn encode_flush(source: E) -> Self {
        Self::EncodeFlush { source }
    }

    /// Creates an unencodable-value error.
    ///
    /// # Parameters
    ///
    /// - `input_index`: Absolute input index of the value being encoded.
    ///
    /// # Returns
    ///
    /// Returns an unencodable-value error.
    #[inline(always)]
    #[must_use]
    pub const fn unencodable_value(input_index: usize) -> Self {
        Self::UnencodableValue { input_index }
    }

    /// Extracts the wrapped codec source error, when this variant has one.
    ///
    /// # Returns
    ///
    /// Returns `Some(source)` for codec encode, reset, and flush failures.
    /// Returns `None` for adapter-only failures.
    #[inline(always)]
    #[must_use]
    pub fn into_source(self) -> Option<E> {
        match self {
            Self::Encode { source, .. }
            | Self::EncodeReset { source }
            | Self::EncodeFlush { source } => Some(source),
            Self::UnencodableValue { .. } => None,
        }
    }
}
