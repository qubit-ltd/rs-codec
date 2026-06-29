// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Buffered encoder adapter backed by a low-level codec.

use super::CodecTranscodeEncodeHooks;
use crate::{
    CapacityError,
    Codec,
    TranscodeEncodeEngine,
    TranscodeEncodeError,
    TranscodeEncoder,
    TranscodeError,
    TranscodeProgress,
    Transcoder,
};

/// Encodes values into caller-provided output units by using a [`Codec`].
///
/// `CodecTranscodeEncoder` is the default bridge from the low-level unchecked
/// [`Codec`] contract to the buffered [`Transcoder`] and
/// [`TranscodeEncoder`] contracts. It encodes complete values only; when the
/// remaining output capacity is smaller than `codec.encode_len(value)`, it
/// stops before consuming that input value and reports
/// [`crate::TranscodeStatus::NeedOutput`].
///
/// # Type Parameters
///
/// - `C`: Low-level codec used to encode values.
#[derive(Debug)]
pub struct CodecTranscodeEncoder<C> {
    /// Common buffered encoding engine.
    engine: TranscodeEncodeEngine<C, CodecTranscodeEncodeHooks>,
}

impl<C> CodecTranscodeEncoder<C>
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
    #[inline]
    #[must_use]
    pub fn new(codec: C) -> Self {
        Self {
            engine: TranscodeEncodeEngine::new(
                codec,
                CodecTranscodeEncodeHooks,
            ),
        }
    }
}

impl<C> Transcoder<C::Value, C::Unit> for CodecTranscodeEncoder<C>
where
    C: Codec,
{
    type Error = TranscodeEncodeError<C>;
    type DomainError = C::EncodeError;

    /// Returns the default streaming adapter error unchanged.
    #[inline(always)]
    fn map_error(
        &self,
        error: TranscodeError<Self::DomainError>,
    ) -> Self::Error {
        error
    }

    /// Gets the maximum number of output units needed for `input_len`
    /// values.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Logical input values the caller plans to encode.
    ///
    /// # Returns
    ///
    /// a conservative upper bound for output units.
    #[inline(always)]
    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        self.engine
            .max_transcode_output_len(input_len)
            .map_err(|_| CapacityError::OutputLengthOverflow)
    }

    /// Gets the maximum units emitted when resetting internal state.
    ///
    /// # Returns
    ///
    /// the maximum units emitted when resetting internal state.
    #[inline(always)]
    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        self.engine.max_reset_output_len()
    }

    /// Gets the maximum units emitted by finishing internal state.
    ///
    /// # Returns
    ///
    /// the number of units that may be emitted by finishing state.
    #[inline(always)]
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        self.engine
            .max_finish_output_len()
            .map_err(|_| CapacityError::OutputLengthOverflow)
    }

    /// Runs before-reset cleanup and emits stream-start output.
    #[inline(always)]
    fn reset(
        &mut self,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        self.engine.reset(output, output_index)
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
    /// Returns conversion progress for consumed input and produced output
    /// units.
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
        self.engine
            .transcode(input, input_index, output, output_index)
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
    fn finish(
        &mut self,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        self.engine.finish(output, output_index)
    }
}

impl<C> TranscodeEncoder<C::Value, C::Unit> for CodecTranscodeEncoder<C>
where
    C: Codec,
{
    // empty
}

impl<C> Default for CodecTranscodeEncoder<C>
where
    C: Codec + Default,
{
    /// Creates a default codec-backed buffered encoder.
    ///
    /// # Returns
    ///
    /// Returns an encoder backed by `C::default()`.
    #[inline(always)]
    fn default() -> Self {
        Self::new(C::default())
    }
}
