// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Domain error reported by buffered encode engines.

use thiserror::Error;

use crate::CodecEncodeError;

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
    /// The wrapped codec failed during encode lifecycle work.
    #[error("{0}")]
    Codec(#[source] CodecEncodeError<C>),

    /// The encode policy hook rejected or failed a value.
    #[error("{0}")]
    Hook(#[source] H),
}

impl<C, H> TranscodeEncodeEngineError<C, H> {
    /// Creates an error from a codec lifecycle failure.
    ///
    /// # Parameters
    ///
    /// - `error`: Codec encode adapter error.
    ///
    /// # Returns
    ///
    /// Returns an encode-engine codec error.
    #[inline(always)]
    #[must_use]
    pub const fn codec(error: CodecEncodeError<C>) -> Self {
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
    /// Returns an encode-engine hook error.
    #[inline(always)]
    #[must_use]
    pub const fn hook(error: H) -> Self {
        Self::Hook(error)
    }
}
