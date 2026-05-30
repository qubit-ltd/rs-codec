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
    Codec,
    CodecDecodeError,
    DecodeErrorInfo,
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
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct CodecBufferedDecoder<C, Unit> {
    /// Common buffered decoding engine.
    engine: BufferedDecodeEngine<C, CodecBufferedDecodeHooks, Unit>,
}

impl<C, Unit> CodecBufferedDecoder<C, Unit> {
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

    /// Returns the wrapped codec.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the wrapped low-level codec.
    #[must_use]
    #[inline(always)]
    pub const fn codec(&self) -> &C {
        self.engine.codec()
    }

    /// Returns a mutable reference to the wrapped codec.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the wrapped low-level codec.
    #[must_use]
    #[inline(always)]
    pub fn codec_mut(&mut self) -> &mut C {
        self.engine.codec_mut()
    }

    /// Consumes the adapter and returns the wrapped codec.
    ///
    /// # Returns
    ///
    /// Returns the codec supplied at construction time.
    #[must_use]
    #[inline(always)]
    pub fn into_codec(self) -> C {
        self.engine.into_codec()
    }
}

impl<C, Value, Unit> Transcoder<Unit, Value> for CodecBufferedDecoder<C, Unit>
where
    C: Codec<Value, Unit>,
    C::DecodeError: DecodeErrorInfo,
    Unit: Copy,
{
    type Error = CodecDecodeError<C::DecodeError>;

    /// Returns an upper bound for decoded values produced from `input_len` units.
    #[inline(always)]
    fn max_output_len(&self, input_len: usize) -> Option<usize> {
        self.engine.max_output_len::<Value>(input_len)
    }

    /// Returns the maximum values emitted by finishing internal state.
    #[inline(always)]
    fn max_finish_output_len(&self) -> Option<usize> {
        self.engine.max_finish_output_len::<Value>()
    }

    /// Resets hook-owned state.
    #[inline(always)]
    fn reset(&mut self) {
        self.engine.reset::<Value>();
    }

    /// Decodes source units into logical values.
    #[inline(always)]
    fn transcode(
        &mut self,
        input: &[Unit],
        input_index: usize,
        output: &mut [Value],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        self.engine.transcode::<Value>(input, input_index, output, output_index)
    }

    /// Finishes internally retained output after EOF.
    #[inline(always)]
    fn finish(&mut self, output: &mut [Value], output_index: usize) -> Result<TranscodeProgress, Self::Error> {
        self.engine.finish::<Value>(output, output_index)
    }
}

impl<C, Value, Unit> BufferedDecoder<Unit, Value> for CodecBufferedDecoder<C, Unit>
where
    C: Codec<Value, Unit>,
    C::DecodeError: DecodeErrorInfo,
    Unit: Copy,
{
}
