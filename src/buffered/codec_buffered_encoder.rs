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
    TranscodeProgress,
    Transcoder,
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
/// [`Codec`] contract to the buffered [`Transcoder`] and [`BufferedEncoder`]
/// contracts. It encodes complete values only; when the remaining output
/// capacity is smaller than `codec.max_units_per_value()`, it stops before
/// consuming the next input value and reports [`crate::TranscodeStatus::NeedOutput`].
///
/// # Type Parameters
///
/// - `C`: Low-level codec used to encode values.
/// - `Value`: Logical value accepted by the encoder.
/// - `Unit`: Encoded output unit type produced by the codec.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CodecBufferedEncoder<C, Value, Unit> {
    /// Common buffered encoding engine.
    engine: BufferedEncodeEngine<C, CodecBufferedEncodeHooks, Value, Unit>,
}

impl<C, Value, Unit> CodecBufferedEncoder<C, Value, Unit>
where
    C: Codec<Value, Unit>,
    Unit: Copy,
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
    pub const fn new(codec: C) -> Self {
        Self {
            engine: BufferedEncodeEngine::new(codec, CodecBufferedEncodeHooks),
        }
    }
}

impl<C, Value, Unit> Transcoder<Value, Unit> for CodecBufferedEncoder<C, Value, Unit>
where
    C: Codec<Value, Unit>,
    Unit: Copy,
{
    type Error = CodecEncodeError<C::EncodeError>;

    /// Returns the maximum number of output units needed for `input_len` values.
    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        self.engine.max_output_len(input_len)
    }

    /// Returns the maximum units emitted by finishing internal state.
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(self.engine.max_finish_output_len())
    }

    /// Resets hook-owned state.
    fn reset(&mut self) {
        self.engine.reset();
    }

    /// Encodes values into the supplied output buffer.
    fn transcode(
        &mut self,
        input: &[Value],
        input_index: usize,
        output: &mut [Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        self.engine.transcode(input, input_index, output, output_index)
    }

    /// Finishes internally retained output after EOF.
    fn finish(&mut self, output: &mut [Unit], output_index: usize) -> Result<TranscodeProgress, Self::Error> {
        self.engine.finish(output, output_index)
    }
}

impl<C, Value, Unit> BufferedEncoder<Value, Unit> for CodecBufferedEncoder<C, Value, Unit>
where
    C: Codec<Value, Unit>,
    Unit: Copy,
{
}
