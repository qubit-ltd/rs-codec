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
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CodecBufferedEncoder<C> {
    /// Common buffered encoding engine.
    engine: BufferedEncodeEngine<C, CodecBufferedEncodeHooks>,
}

impl<C> CodecBufferedEncoder<C> {
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

impl<C, Value, Unit> Transcoder<Value, Unit> for CodecBufferedEncoder<C>
where
    C: Codec<Value, Unit>,
    Unit: Copy,
{
    type Error = CodecEncodeError<C::EncodeError>;

    /// Returns the maximum number of output units needed for `input_len` values.
    #[inline(always)]
    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        self.engine.max_output_len::<Value, Unit>(input_len)
    }

    /// Returns the maximum units emitted by finishing internal state.
    #[inline(always)]
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(self.engine.max_finish_output_len::<Value, Unit>())
    }

    /// Resets hook-owned state.
    #[inline(always)]
    fn reset(&mut self) {
        self.engine.reset::<Value, Unit>();
    }

    /// Encodes values into the supplied output buffer.
    #[inline(always)]
    fn transcode(
        &mut self,
        input: &[Value],
        input_index: usize,
        output: &mut [Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        self.engine
            .transcode::<Value, Unit>(input, input_index, output, output_index)
    }

    /// Finishes internally retained output after EOF.
    #[inline(always)]
    fn finish(&mut self, output: &mut [Unit], output_index: usize) -> Result<TranscodeProgress, Self::Error> {
        self.engine.finish::<Value, Unit>(output, output_index)
    }
}

impl<C, Value, Unit> BufferedEncoder<Value, Unit> for CodecBufferedEncoder<C>
where
    C: Codec<Value, Unit>,
    Unit: Copy,
{
}
