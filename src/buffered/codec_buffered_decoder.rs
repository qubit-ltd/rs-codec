/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Buffered decoder adapter backed by a low-level codec.

use super::{
    BufferedDecoder,
    FinishError,
    TranscodeProgress,
    Transcoder,
    buffered_decode_engine::BufferedDecodeEngine,
    codec_buffered_decode_hooks::CodecBufferedDecodeHooks,
};
use crate::{
    CapacityError,
    Codec,
    CodecDecodeError,
};

/// Decodes encoded units into caller-provided value buffers by using a [`Codec`].
///
/// `CodecBufferedDecoder` is a policy-free bridge from the low-level unchecked
/// [`Codec`] contract to [`Transcoder`] and [`BufferedDecoder`]. It leaves
/// incomplete input tails in the caller-provided input slice; callers own
/// input-buffer refill and EOF incomplete-tail policy.
///
/// # Type Parameters
///
/// - `C`: Low-level codec used to decode values.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct CodecBufferedDecoder<C> {
    /// Common buffered decoding engine.
    engine: BufferedDecodeEngine<C, CodecBufferedDecodeHooks>,
}

impl<C> CodecBufferedDecoder<C>
where
    C: Codec,
{
    /// Creates a buffered decoder backed by `codec`.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec used to decode values.
    ///
    /// # Returns
    ///
    /// Returns a buffered decoder adapter for the supplied codec.
    #[must_use]
    #[inline(always)]
    pub const fn new(codec: C) -> Self {
        Self {
            engine: BufferedDecodeEngine::new(codec, CodecBufferedDecodeHooks),
        }
    }
}

impl<C> Transcoder<C::Unit, C::Value> for CodecBufferedDecoder<C>
where
    C: Codec,
{
    type Error = CodecDecodeError<C::DecodeError>;

    /// Returns an upper bound for decoded values produced from `input_len` units.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Source units the caller plans to decode.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound for decoded values.
    #[inline(always)]
    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        self.engine.max_output_len(input_len)
    }

    /// Returns the maximum values emitted by finishing internal state.
    ///
    /// # Returns
    ///
    /// Returns the number of values that may still be emitted by finishing state.
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

    /// Decodes source units into logical values.
    ///
    /// # Parameters
    ///
    /// - `input`: Source unit slice.
    /// - `input_index`: Absolute source index where decoding starts.
    /// - `output`: Destination value slice.
    /// - `output_index`: Absolute output value index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns conversion progress for consumed and written counters.
    ///
    /// # Errors
    ///
    /// Returns a decode error if conversion fails under hook policy.
    #[inline(always)]
    fn transcode(
        &mut self,
        input: &[C::Unit],
        input_index: usize,
        output: &mut [C::Value],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        self.engine.transcode(input, input_index, output, output_index)
    }

    /// Finishes internally retained output after EOF.
    ///
    /// # Parameters
    ///
    /// - `output`: Destination value slice for final retained values.
    /// - `output_index`: Absolute output value index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of values written by finalization.
    ///
    /// # Errors
    ///
    /// Returns a finish error if finalization cannot complete.
    #[inline(always)]
    fn finish(&mut self, output: &mut [C::Value], output_index: usize) -> Result<usize, FinishError<Self::Error>> {
        self.engine.finish(output, output_index)
    }
}

impl<C> BufferedDecoder<C::Unit, C::Value> for CodecBufferedDecoder<C> where C: Codec {}
