/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Buffered encoder adapter backed by a low-level codec.

use super::{
    BufferedEncoder,
    BufferedTranscoder,
    FinishError,
    TranscodeProgress,
    buffered_encode_engine::BufferedEncodeEngine,
    codec_buffered_encode_hooks::CodecBufferedEncodeHooks,
};
use crate::{
    CapacityError,
    Codec,
    CodecEncodeError,
};

/// Encodes values into caller-provided output units by using a [`Codec`].
///
/// `CodecBufferedEncoder` is the default bridge from the low-level unchecked
/// [`Codec`] contract to the buffered [`BufferedTranscoder`] and [`BufferedEncoder`]
/// contracts. It encodes complete values only; when the remaining output
/// capacity is smaller than `codec.max_units_per_value()`, it stops before
/// consuming the next input value and reports [`crate::TranscodeStatus::NeedOutput`].
///
/// # Type Parameters
///
/// - `C`: Low-level codec used to encode values.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CodecBufferedEncoder<C> {
    /// Common buffered encoding engine.
    engine: BufferedEncodeEngine<C, CodecBufferedEncodeHooks>,
}

impl<C> CodecBufferedEncoder<C>
where
    C: Codec,
{
    /// Creates a buffered encoder backed by `codec`.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec used to encode values.
    ///
    /// # Returns
    ///
    /// Returns a buffered encoder adapter for the supplied codec.
    #[must_use]
    #[inline(always)]
    pub const fn new(codec: C) -> Self {
        Self {
            engine: BufferedEncodeEngine::new(codec, CodecBufferedEncodeHooks),
        }
    }
}

impl<C> BufferedTranscoder<C::Value, C::Unit> for CodecBufferedEncoder<C>
where
    C: Codec,
{
    type Error = CodecEncodeError<C::EncodeError>;

    /// Returns the maximum number of output units needed for `input_len` values.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Logical input values the caller plans to encode.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound for output units.
    #[inline(always)]
    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        self.engine.max_output_len(input_len)
    }

    /// Returns the maximum units emitted by finishing internal state.
    ///
    /// # Returns
    ///
    /// Returns the number of units that may be emitted by finishing state.
    #[inline(always)]
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(self.engine.max_finish_output_len())
    }

    /// Resets hook-owned state.
    ///
    /// # Returns
    ///
    /// Returns unit `()`.
    #[inline(always)]
    fn reset(&mut self) {
        self.engine.reset();
    }

    /// Encodes values into the supplied output buffer.
    ///
    /// # Parameters
    ///
    /// - `input`: Input value slice.
    /// - `input_index`: Absolute input index where encoding starts.
    /// - `output`: Destination unit slice.
    /// - `output_index`: Absolute output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns conversion progress for consumed input and produced output units.
    ///
    /// # Errors
    ///
    /// Returns an encode error when indices are invalid or when encoding cannot
    /// continue under current policy.
    #[inline(always)]
    fn transcode(
        &mut self,
        input: &[C::Value],
        input_index: usize,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        self.engine.transcode(input, input_index, output, output_index)
    }

    /// Finishes internally retained output after EOF.
    ///
    /// # Parameters
    ///
    /// - `output`: Destination unit slice for finalization output.
    /// - `output_index`: Absolute output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of units written by finalization.
    ///
    /// # Errors
    ///
    /// Returns a finish error if retained output cannot be fully emitted.
    #[inline(always)]
    fn finish(&mut self, output: &mut [C::Unit], output_index: usize) -> Result<usize, FinishError<Self::Error>> {
        self.engine.finish(output, output_index)
    }
}

impl<C> BufferedEncoder<C::Value, C::Unit> for CodecBufferedEncoder<C> where C: Codec {}
