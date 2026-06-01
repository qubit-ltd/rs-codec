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
/// - `Unit`: Encoded unit type accepted by the codec.
/// - `Value`: Logical value produced by the decoder.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct CodecBufferedDecoder<C, Unit, Value> {
    /// Common buffered decoding engine.
    engine: BufferedDecodeEngine<C, CodecBufferedDecodeHooks, Unit, Value>,
}

impl<C, Unit, Value> CodecBufferedDecoder<C, Unit, Value>
where
    C: Codec<Value, Unit>,
    Unit: Copy,
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
    pub const fn new(codec: C) -> Self {
        Self {
            engine: BufferedDecodeEngine::new(codec, CodecBufferedDecodeHooks),
        }
    }
}

impl<C, Unit, Value> Transcoder<Unit, Value> for CodecBufferedDecoder<C, Unit, Value>
where
    C: Codec<Value, Unit>,
    Unit: Copy,
{
    type Error = CodecDecodeError<C::DecodeError>;

    /// Returns an upper bound for decoded values produced from `input_len` units.
    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        self.engine.max_output_len(input_len)
    }

    /// Returns the maximum values emitted by finishing internal state.
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(self.engine.max_finish_output_len())
    }

    /// Resets hook-owned state.
    fn reset(&mut self) {
        self.engine.reset();
    }

    /// Decodes source units into logical values.
    fn transcode(
        &mut self,
        input: &[Unit],
        input_index: usize,
        output: &mut [Value],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        self.engine.transcode(input, input_index, output, output_index)
    }

    /// Finishes internally retained output after EOF.
    fn finish(&mut self, output: &mut [Value], output_index: usize) -> Result<TranscodeProgress, Self::Error> {
        self.engine.finish(output, output_index)
    }
}

impl<C, Unit, Value> BufferedDecoder<Unit, Value> for CodecBufferedDecoder<C, Unit, Value>
where
    C: Codec<Value, Unit>,
    Unit: Copy,
{
}
