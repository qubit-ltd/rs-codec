/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Policy hooks used by buffered convert engines.

use core::num::NonZeroUsize;

use super::{
    buffered_decode_hooks::BufferedDecodeHooks,
    buffered_encode_hooks::BufferedEncodeHooks,
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
/// - `D`: Source-side codec owned by the converter engine.
/// - `E`: Target-side codec owned by the converter engine.
/// - `Input`: Source unit type.
/// - `Value`: Logical value decoded from `Input` and encoded into target units.
pub trait BufferedConvertHooks<D, E, Input, Value>
where
    D: Codec<Value, Input>,
    Input: Copy,
{
    /// Decode policy hooks used by the internal buffered decoder.
    type DecodeHooks: BufferedDecodeHooks<D, Input, Value>;

    /// Encode policy hooks used by the internal buffered encoder.
    type EncodeHooks;

    /// Error type returned by the selected encode hooks.
    type EncodeError<Output>
    where
        E: Codec<Value, Output>,
        Output: Copy;

    /// Error type returned by the buffered converter.
    type Error<Output>
    where
        E: Codec<Value, Output>,
        Output: Copy,
        Self::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = Self::EncodeError<Output>>;

    /// Creates decode policy hooks for the internal buffered decoder.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Source codec owned by the converter engine.
    /// - `encoder`: Target codec owned by the converter engine.
    ///
    /// # Returns
    ///
    /// Returns the decode hooks used by the internal buffered decoder.
    fn create_decode_hooks(&self, decoder: &D, encoder: &E) -> Self::DecodeHooks;

    /// Creates encode policy hooks for the internal buffered encoder.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Source codec owned by the converter engine.
    /// - `encoder`: Target codec owned by the converter engine.
    ///
    /// # Returns
    ///
    /// Returns the encode hooks used by the internal buffered encoder.
    fn create_encode_hooks(&self, decoder: &D, encoder: &E) -> Self::EncodeHooks;

    /// Maps a decode-engine error into the converter error type.
    ///
    /// # Parameters
    ///
    /// - `error`: Error returned by the selected decode hooks.
    ///
    /// # Returns
    ///
    /// Returns the converter-level error.
    fn map_decode_error<Output>(
        &self,
        error: <Self::DecodeHooks as BufferedDecodeHooks<D, Input, Value>>::Error,
    ) -> Self::Error<Output>
    where
        E: Codec<Value, Output>,
        Output: Copy,
        Self::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = Self::EncodeError<Output>>;

    /// Maps an encode-engine error into the converter error type.
    ///
    /// # Parameters
    ///
    /// - `error`: Error returned by the selected encode hooks.
    ///
    /// # Returns
    ///
    /// Returns the converter-level error.
    fn map_encode_error<Output>(&self, error: Self::EncodeError<Output>) -> Self::Error<Output>
    where
        E: Codec<Value, Output>,
        Output: Copy,
        Self::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = Self::EncodeError<Output>>;

    /// Builds an error for a caller-supplied source input index outside the input slice.
    ///
    /// The engine calls this hook before it reads source input. Keeping this
    /// construction in the hook lets converter adapters preserve their concrete
    /// error type without a separate public factory trait.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Source codec owned by the engine.
    /// - `index`: Invalid absolute input index supplied by the caller.
    /// - `input_len`: Length of the input slice.
    ///
    /// # Returns
    ///
    /// Returns the hook-specific invalid-input-index error.
    fn invalid_input_index<Output>(&self, decoder: &D, index: usize, input_len: usize) -> Self::Error<Output>
    where
        E: Codec<Value, Output>,
        Output: Copy,
        Self::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = Self::EncodeError<Output>>;

    /// Returns the additional output units requested when `output_index` is invalid.
    ///
    /// The default uses the target codec maximum value width.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Source codec owned by the engine.
    /// - `encoder`: Target codec owned by the engine.
    ///
    /// # Returns
    ///
    /// Returns at least one additional output unit.
    #[must_use]
    #[inline(always)]
    fn invalid_output_additional<Output>(&self, _decoder: &D, encoder: &E) -> NonZeroUsize
    where
        E: Codec<Value, Output>,
        Output: Copy,
        Self::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = Self::EncodeError<Output>>,
    {
        encoder.max_units_per_value()
    }

    /// Resets conversion-level hook-owned state.
    ///
    /// The common engine clears pending decoded values and resets internal
    /// decode/encode engines separately.
    #[inline(always)]
    fn reset(&mut self) {}
}
