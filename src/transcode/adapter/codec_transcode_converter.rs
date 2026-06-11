// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Buffered converter adapter backed by two low-level codecs.

use core::{
    fmt,
    hash::{
        Hash,
        Hasher,
    },
};

use super::CodecTranscodeConvertHooks;
use crate::{
    CapacityError,
    Codec,
    CodecConvertError,
    TranscodeConvertEngine,
    TranscodeConverter,
    TranscodeError,
    TranscodeProgress,
    TranscodeStatus,
    Transcoder,
};

/// Strict codec-backed converter error type.
type CodecTranscodeConvertError<D, E> =
    CodecConvertError<<D as Codec>::DecodeError, <E as Codec>::EncodeError>;

/// Converts source units to target units through a decoded value by using
/// codecs.
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
pub struct CodecTranscodeConverter<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
{
    /// Common buffered converter engine.
    engine: TranscodeConvertEngine<D, E, CodecTranscodeConvertHooks>,
}

impl<D, E> Clone for CodecTranscodeConverter<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    TranscodeConvertEngine<D, E, CodecTranscodeConvertHooks>: Clone,
{
    /// Clones the wrapped converter engine.
    ///
    /// # Returns
    ///
    /// Returns a cloned converter adapter sharing the same inner engine state.
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            engine: self.engine.clone(),
        }
    }
}

impl<D, E> fmt::Debug for CodecTranscodeConverter<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    TranscodeConvertEngine<D, E, CodecTranscodeConvertHooks>: fmt::Debug,
{
    /// Formats the wrapped converter engine for debugging.
    ///
    /// # Parameters
    ///
    /// - `f`: Destination formatter.
    ///
    /// # Returns
    ///
    /// Returns `fmt::Result` from the formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodecTranscodeConverter")
            .field("engine", &self.engine)
            .finish()
    }
}

impl<D, E> Default for CodecTranscodeConverter<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    TranscodeConvertEngine<D, E, CodecTranscodeConvertHooks>: Default,
{
    /// Creates a default codec-backed buffered converter.
    ///
    /// # Returns
    ///
    /// Returns a converter with default codecs and hooks.
    #[inline(always)]
    fn default() -> Self {
        Self {
            engine: TranscodeConvertEngine::default(),
        }
    }
}

impl<D, E> Eq for CodecTranscodeConverter<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    TranscodeConvertEngine<D, E, CodecTranscodeConvertHooks>: Eq,
{
}

impl<D, E> Hash for CodecTranscodeConverter<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    TranscodeConvertEngine<D, E, CodecTranscodeConvertHooks>: Hash,
{
    /// Hashes the wrapped converter engine.
    ///
    /// # Parameters
    ///
    /// - `state`: Output hash state.
    ///
    /// # Returns
    ///
    /// Returns unit `()`.
    #[inline(always)]
    fn hash<S: Hasher>(&self, state: &mut S) {
        self.engine.hash(state);
    }
}

impl<D, E> PartialEq for CodecTranscodeConverter<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    TranscodeConvertEngine<D, E, CodecTranscodeConvertHooks>: PartialEq,
{
    /// Compares the wrapped converter engine.
    ///
    /// # Parameters
    ///
    /// - `other`: Another converter to compare with.
    ///
    /// # Returns
    ///
    /// Returns `true` when the wrapped engines are equal.
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.engine == other.engine
    }
}

impl<D, E> CodecTranscodeConverter<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
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
    #[inline(always)]
    pub fn new(decoder: D, encoder: E) -> Self {
        Self {
            engine: TranscodeConvertEngine::new(
                decoder,
                encoder,
                CodecTranscodeConvertHooks::new(),
            ),
        }
    }

    /// Returns an upper bound for target units produced from `input_len` units.
    ///
    /// This concrete adapter method is available even when `D::Value` does not
    /// implement [`Default`].
    ///
    /// # Parameters
    ///
    /// - `input_len`: Source units the caller plans to convert.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound for produced target units.
    #[must_use = "capacity planning can fail on overflow"]
    #[inline(always)]
    pub fn max_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        self.engine.max_output_len(input_len)
    }

    /// Returns the maximum target units emitted by finishing internal state.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound for remaining converter-final output.
    #[must_use = "capacity planning can fail on overflow"]
    #[inline(always)]
    pub fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        self.engine.max_finish_output_len()
    }

    /// Returns the maximum target units emitted when resetting stream state.
    #[must_use = "capacity planning can fail on overflow"]
    #[inline(always)]
    pub fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        self.engine.max_reset_output_len()
    }

    /// Clears retained pending output and hook state and emits stream-start
    /// encode output.
    #[inline(always)]
    pub fn reset(
        &mut self,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<usize, CodecTranscodeConvertError<D, E>> {
        self.engine.reset(output, output_index)
    }

    /// Converts source units into target units.
    ///
    /// This is the main streaming operation and does not require `D::Value` to
    /// implement [`Default`].
    ///
    /// # Parameters
    ///
    /// - `input`: Source unit slice.
    /// - `input_index`: Absolute source index where conversion starts.
    /// - `output`: Target unit slice.
    /// - `output_index`: Absolute target index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns conversion progress for consumed/produced counters and stop
    /// reason.
    ///
    /// # Errors
    ///
    /// Returns converter error when source or target indices are invalid, or
    /// when decoding/encoding fails under current policy.
    #[inline(always)]
    pub fn transcode(
        &mut self,
        input: &[D::Unit],
        input_index: usize,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, CodecTranscodeConvertError<D, E>> {
        self.engine
            .transcode(input, input_index, output, output_index)
    }

    /// Finishes internally retained output after EOF.
    ///
    /// The strict codec-backed converter has no hook-owned final output. Finish
    /// drains any retained decoded value through the normal conversion path and
    /// then completes without requiring `D::Value: Default`.
    ///
    /// # Parameters
    ///
    /// - `output`: Target unit slice for finalization output.
    /// - `output_index`: Absolute target output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of target units written by finalization.
    ///
    /// # Errors
    ///
    /// Returns a finish error for pending output that cannot be finalized.
    #[inline(always)]
    pub fn finish(
        &mut self,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<usize, CodecTranscodeConvertError<D, E>> {
        let required = self.max_finish_output_len().unwrap_or(usize::MAX);
        <CodecTranscodeConvertError<D, E> as TranscodeError<()>>::ensure_output_capacity(
            (),
            output.len(),
            output_index,
            required,
        )?;

        let empty_input: &[D::Unit] = &[];
        let progress = self.transcode(empty_input, 0, output, output_index)?;
        match progress.status() {
            TranscodeStatus::Complete => Ok(progress.written()),
            TranscodeStatus::NeedInput { .. } => {
                unreachable!(
                    "codec converter finish uses empty input and strict no-op decode finish hooks"
                )
            }
            TranscodeStatus::NeedOutput { .. } => {
                unreachable!(
                    "codec converter finish reserves the complete pending-output bound before draining"
                )
            }
        }
    }
}

impl<D, E> Transcoder<D::Unit, E::Unit> for CodecTranscodeConverter<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
{
    type Error = CodecConvertError<D::DecodeError, E::EncodeError>;
    type ErrorContext = ();

    /// Returns an upper bound for target units produced from `input_len` units.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Source units the caller plans to convert.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound for produced target units.
    #[inline(always)]
    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        CodecTranscodeConverter::max_output_len(self, input_len)
    }

    /// Returns the maximum target units emitted by finishing internal state.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound for remaining converter-final output.
    #[inline(always)]
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        CodecTranscodeConverter::max_finish_output_len(self)
    }

    /// Returns the maximum target units emitted when resetting stream state.
    #[inline(always)]
    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        CodecTranscodeConverter::max_reset_output_len(self)
    }

    /// Clears retained pending output and emits stream-start encode output.
    #[inline(always)]
    fn reset(
        &mut self,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        CodecTranscodeConverter::reset(self, output, output_index)
    }

    /// Converts source units into target units.
    ///
    /// # Parameters
    ///
    /// - `input`: Source unit slice.
    /// - `input_index`: Absolute source index where conversion starts.
    /// - `output`: Target unit slice.
    /// - `output_index`: Absolute target index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns conversion progress for consumed/produced counters and stop
    /// reason.
    ///
    /// # Errors
    ///
    /// Returns converter error when source or target indices are invalid, or
    /// when decoding/encoding fails under current policy.
    #[inline(always)]
    fn transcode(
        &mut self,
        input: &[D::Unit],
        input_index: usize,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        CodecTranscodeConverter::transcode(
            self,
            input,
            input_index,
            output,
            output_index,
        )
    }

    /// Finishes internally retained output after EOF.
    ///
    /// # Parameters
    ///
    /// - `output`: Target unit slice for finalization output.
    /// - `output_index`: Absolute target output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of target units written by finalization.
    ///
    /// # Errors
    ///
    /// Returns a finish error for pending output that cannot be finalized.
    #[inline(always)]
    fn finish(
        &mut self,
        output: &mut [E::Unit],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        CodecTranscodeConverter::finish(self, output, output_index)
    }
}

impl<D, E> TranscodeConverter<D::Unit, E::Unit>
    for CodecTranscodeConverter<D, E>
where
    D: Codec,
    E: Codec<Value = D::Value>,
{
}
