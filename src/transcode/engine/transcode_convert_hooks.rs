// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy hooks used by buffered convert engines.

use crate::Codec;

/// Error mapping hooks for [`crate::TranscodeConvertEngine`].
///
/// Convert hooks no longer create decode or encode policy hooks. The converter
/// engine owns those components directly. This trait only maps decode-side and
/// encode-side errors into one converter-level error type, plus optional
/// conversion-level reset state.
///
/// # Type Parameters
///
/// - `D`: Source-side decode codec owned by the converter engine.
/// - `E`: Target-side encode codec owned by the converter engine.
pub trait TranscodeConvertHooks<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
{
    /// Domain error type returned by the buffered converter.
    type Error;

    /// Error type returned by the selected decode hooks.
    type DecodeError;

    /// Error type returned by the selected encode hooks.
    type EncodeError;

    /// Maps a decode-engine error into the converter error type.
    ///
    /// # Parameters
    ///
    /// - `error`: Error returned by the selected decode hooks.
    ///
    /// # Returns
    ///
    /// Returns the converter-level error.
    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error;

    /// Maps an encode-engine error into the converter error type.
    ///
    /// # Parameters
    ///
    /// - `error`: Error returned by the selected encode hooks.
    ///
    /// # Returns
    ///
    /// Returns the converter-level error.
    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error;

    /// Runs conversion-level hook cleanup before stream reset.
    ///
    /// The common engine clears pending decoded values and resets internal
    /// decode/encode engines separately.
    #[inline(always)]
    fn before_reset(&mut self) {}
}
