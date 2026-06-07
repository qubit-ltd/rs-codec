// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy hooks used by buffered convert engines.

use super::{
    buffered_decode_hooks::BufferedDecodeHooks, buffered_encode_hooks::BufferedEncodeHooks,
};
use crate::Codec;

/// Policy hooks for [`crate::BufferedConvertEngine`].
///
/// Convert hooks no longer own decoded pending values or the conversion loop.
/// The engine owns source/target cursor state, retained decoded values, output
/// capacity checks, and final progress reporting. Hooks only select the
/// decode/encode policies used by the internal buffered engines and map their
/// errors into one converter-level error type.
///
/// # Type Parameters
///
/// - `D`: Source-side decode codec owned by the converter engine.
/// - `E`: Target-side encode codec owned by the converter engine.
pub trait BufferedConvertHooks<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
{
    /// Error type returned by the buffered converter.
    type Error;

    /// Error type returned by the selected decode hooks.
    type DecodeError;

    /// Error type returned by the selected encode hooks.
    type EncodeError;

    /// Decode policy hooks used by the internal buffered decoder.
    type DecodeHooks: BufferedDecodeHooks<D, Error = Self::DecodeError>;

    /// Encode policy hooks used by the internal buffered encoder.
    type EncodeHooks: BufferedEncodeHooks<E, Error = Self::EncodeError>;

    /// Creates decode policy hooks for the internal buffered decoder.
    ///
    /// # Parameters
    ///
    /// - `decode_codec`: Source codec owned by the converter engine.
    /// - `encode_codec`: Target codec owned by the converter engine.
    ///
    /// # Returns
    ///
    /// Returns the decode hooks used by the internal buffered decoder.
    fn create_decode_hooks(&self, decode_codec: &D, encode_codec: &E) -> Self::DecodeHooks;

    /// Creates encode policy hooks for the internal buffered encoder.
    ///
    /// # Parameters
    ///
    /// - `decode_codec`: Source codec owned by the converter engine.
    /// - `encode_codec`: Target codec owned by the converter engine.
    ///
    /// # Returns
    ///
    /// Returns the encode hooks used by the internal buffered encoder.
    fn create_encode_hooks(&self, decode_codec: &D, encode_codec: &E) -> Self::EncodeHooks;

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

    /// Builds an error for a caller-supplied source input index outside the
    /// input slice.
    ///
    /// The engine calls this hook before it reads source input. Keeping this
    /// construction in the hook lets converter adapters preserve their concrete
    /// error type without a separate public factory trait.
    ///
    /// # Parameters
    ///
    /// - `decode_codec`: Source codec owned by the engine.
    /// - `index`: Invalid absolute input index supplied by the caller.
    /// - `input_len`: Length of the input slice.
    ///
    /// # Returns
    ///
    /// Returns the hook-specific invalid-input-index error.
    fn invalid_input_index(&self, decode_codec: &D, index: usize, input_len: usize) -> Self::Error;

    /// Resets conversion-level hook-owned state.
    ///
    /// The common engine clears pending decoded values and resets internal
    /// decode/encode engines separately.
    #[inline(always)]
    fn reset(&mut self) {}
}
