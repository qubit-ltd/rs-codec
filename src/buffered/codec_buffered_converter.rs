/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Buffered converter adapter backed by two low-level codecs.

use core::marker::PhantomData;

use super::{
    BufferedConvertEngine,
    BufferedConverter,
    TranscodeProgress,
    Transcoder,
    codec_buffered_convert_hooks::CodecBufferedConvertHooks,
};
use crate::{
    Codec,
    CodecConvertError,
    DecodeErrorInfo,
};

/// Converts source units to target units through a decoded value by using codecs.
///
/// The converter decodes one source value with the decoder codec, then encodes
/// that value with the encoder codec. If the current output buffer cannot hold
/// the encoded value, the already decoded value is retained by the common
/// converter engine and must be drained before more source input is consumed.
/// Incomplete source tails are left in the caller-provided input slice; callers
/// own input-buffer refill and EOF incomplete-tail policy.
///
/// # Type Parameters
///
/// - `D`: Low-level codec used to decode source units.
/// - `E`: Low-level codec used to encode target units.
/// - `Value`: Logical value decoded by `D` and encoded by `E`.
/// - `InputUnit`: Encoded source unit type accepted by `D`.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct CodecBufferedConverter<D, E, Value, InputUnit> {
    /// Common buffered converter engine.
    engine: BufferedConvertEngine<D, E, CodecBufferedConvertHooks<Value>, InputUnit>,
    /// Binds the adapter to one decoded logical value and source unit type.
    marker: PhantomData<fn(Value, InputUnit)>,
}

impl<D, E, Value, InputUnit> CodecBufferedConverter<D, E, Value, InputUnit> {
    /// Creates a buffered converter backed by decoder and encoder codecs.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Low-level codec used to decode source units.
    /// - `encoder`: Low-level codec used to encode target units.
    ///
    /// # Returns
    ///
    /// Returns a buffered converter adapter for the supplied codecs.
    #[must_use]
    #[inline(always)]
    pub const fn new(decoder: D, encoder: E) -> Self {
        Self {
            engine: BufferedConvertEngine::new(decoder, encoder, CodecBufferedConvertHooks::new()),
            marker: PhantomData,
        }
    }

    /// Returns the wrapped decoder codec.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the wrapped decoder codec.
    #[must_use]
    #[inline(always)]
    pub const fn decoder(&self) -> &D {
        self.engine.decoder()
    }

    /// Returns the wrapped encoder codec.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the wrapped encoder codec.
    #[must_use]
    #[inline(always)]
    pub const fn encoder(&self) -> &E {
        self.engine.encoder()
    }

    /// Returns a mutable reference to the wrapped decoder codec.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the wrapped decoder codec.
    #[must_use]
    #[inline(always)]
    pub fn decoder_mut(&mut self) -> &mut D {
        self.engine.decoder_mut()
    }

    /// Returns a mutable reference to the wrapped encoder codec.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the wrapped encoder codec.
    #[must_use]
    #[inline(always)]
    pub fn encoder_mut(&mut self) -> &mut E {
        self.engine.encoder_mut()
    }

    /// Consumes the adapter and returns the wrapped codecs.
    ///
    /// # Returns
    ///
    /// Returns `(decoder, encoder)` supplied at construction time.
    #[must_use]
    #[inline(always)]
    pub fn into_codecs(self) -> (D, E) {
        let (decoder, encoder, _) = self.engine.into_parts();
        (decoder, encoder)
    }
}

impl<D, E, Value, InputUnit, OutputUnit> Transcoder<InputUnit, OutputUnit>
    for CodecBufferedConverter<D, E, Value, InputUnit>
where
    D: Codec<Value, InputUnit>,
    D::DecodeError: DecodeErrorInfo,
    E: Codec<Value, OutputUnit>,
    InputUnit: Copy,
    OutputUnit: Copy,
{
    type Error = CodecConvertError<D::DecodeError, E::EncodeError>;

    /// Returns an upper bound for target units produced from `input_len` units.
    #[inline(always)]
    fn max_output_len(&self, input_len: usize) -> Option<usize> {
        self.engine.max_output_len::<Value, OutputUnit>(input_len)
    }

    /// Returns the maximum target units emitted by finishing internal state.
    #[inline(always)]
    fn max_finish_output_len(&self) -> Option<usize> {
        self.engine.max_finish_output_len::<Value, OutputUnit>()
    }

    /// Clears retained pending output.
    #[inline(always)]
    fn reset(&mut self) {
        self.engine.reset::<Value, OutputUnit>();
    }

    /// Converts source units into target units.
    #[inline]
    fn transcode(
        &mut self,
        input: &[InputUnit],
        input_index: usize,
        output: &mut [OutputUnit],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        self.engine
            .transcode::<Value, OutputUnit>(input, input_index, output, output_index)
    }

    /// Finishes internally retained output after EOF.
    #[inline]
    fn finish(&mut self, output: &mut [OutputUnit], output_index: usize) -> Result<TranscodeProgress, Self::Error> {
        self.engine.finish::<Value, OutputUnit>(output, output_index)
    }
}

impl<D, E, Value, InputUnit, OutputUnit> BufferedConverter<InputUnit, OutputUnit>
    for CodecBufferedConverter<D, E, Value, InputUnit>
where
    D: Codec<Value, InputUnit>,
    D::DecodeError: DecodeErrorInfo,
    E: Codec<Value, OutputUnit>,
    InputUnit: Copy,
    OutputUnit: Copy,
{
}
