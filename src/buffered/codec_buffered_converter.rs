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
    CapacityError,
    Codec,
    CodecConvertError,
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
pub struct CodecBufferedConverter<D, E, Value, InputUnit>
where
    D: Codec<Value, InputUnit>,
    InputUnit: Copy,
{
    /// Common buffered converter engine.
    engine: BufferedConvertEngine<D, E, CodecBufferedConvertHooks, InputUnit, Value>,
    /// Binds the adapter to one decoded logical value and source unit type.
    marker: PhantomData<fn(Value, InputUnit)>,
}

impl<D, E, Value, InputUnit> CodecBufferedConverter<D, E, Value, InputUnit>
where
    D: Codec<Value, InputUnit>,
    InputUnit: Copy,
{
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
    pub fn new(decoder: D, encoder: E) -> Self {
        Self {
            engine: BufferedConvertEngine::new(decoder, encoder, CodecBufferedConvertHooks::new()),
            marker: PhantomData,
        }
    }
}

impl<D, E, Value, InputUnit, OutputUnit> Transcoder<InputUnit, OutputUnit>
    for CodecBufferedConverter<D, E, Value, InputUnit>
where
    D: Codec<Value, InputUnit>,
    E: Codec<Value, OutputUnit>,
    Value: Default,
    InputUnit: Copy,
    OutputUnit: Copy,
{
    type Error = CodecConvertError<D::DecodeError, E::EncodeError>;

    /// Returns an upper bound for target units produced from `input_len` units.
    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        self.engine.max_output_len::<OutputUnit>(input_len)
    }

    /// Returns the maximum target units emitted by finishing internal state.
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        self.engine.max_finish_output_len::<OutputUnit>()
    }

    /// Clears retained pending output.
    fn reset(&mut self) {
        self.engine.reset::<OutputUnit>();
    }

    /// Converts source units into target units.
    fn transcode(
        &mut self,
        input: &[InputUnit],
        input_index: usize,
        output: &mut [OutputUnit],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        self.engine
            .transcode::<OutputUnit>(input, input_index, output, output_index)
    }

    /// Finishes internally retained output after EOF.
    fn finish(&mut self, output: &mut [OutputUnit], output_index: usize) -> Result<TranscodeProgress, Self::Error> {
        self.engine.finish::<OutputUnit>(output, output_index)
    }
}

impl<D, E, Value, InputUnit, OutputUnit> BufferedConverter<InputUnit, OutputUnit>
    for CodecBufferedConverter<D, E, Value, InputUnit>
where
    D: Codec<Value, InputUnit>,
    E: Codec<Value, OutputUnit>,
    Value: Default,
    InputUnit: Copy,
    OutputUnit: Copy,
{
}
