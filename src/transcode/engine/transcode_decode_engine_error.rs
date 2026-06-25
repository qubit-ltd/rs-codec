// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Domain error reported by buffered decode engines.

use thiserror::Error;

use crate::CodecDecodeError;

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
    /// The wrapped codec failed during decode lifecycle work.
    #[error("{0}")]
    Codec(#[source] CodecDecodeError<C>),

    /// The decode policy hook rejected or failed input.
    #[error("{0}")]
    Hook(#[source] H),
}

impl<C, H> TranscodeDecodeEngineError<C, H> {
    /// Creates an error from a codec lifecycle failure.
    ///
    /// # Parameters
    ///
    /// - `error`: Codec decode adapter error.
    ///
    /// # Returns
    ///
    /// Returns a decode-engine codec error.
    #[inline(always)]
    #[must_use]
    pub const fn codec(error: CodecDecodeError<C>) -> Self {
        Self::Codec(error)
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
