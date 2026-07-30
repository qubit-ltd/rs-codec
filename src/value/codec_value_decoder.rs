// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Value decoder adapter backed by a low-level codec.

use core::fmt;

use super::{DecodeLifecycleOutput, DecodeLifecycleProgress, ValueDecoder};
use crate::{
    Codec, TranscodeDecodeErrorOf, TranscodeFailure, codec::assert_decode_lifecycle_bounds,
    value::codec_value_lifecycle::decode_exact_complete_value,
};

/// Decodes one encoded unit slice into one owned value by using a [`Codec`].
///
/// `CodecValueDecoder` is the default bridge from the low-level unchecked
/// [`Codec`] contract to the convenience-layer [`ValueDecoder`] contract. The
/// supplied input slice must contain exactly one encoded value. The adapter
/// runs the complete decode lifecycle, including [`Codec::decode_finish`],
/// before returning the decoded value.
///
/// # Type Parameters
///
/// - `C`: Low-level codec used to decode one value.
pub struct CodecValueDecoder<C>
where
    C: Codec,
{
    /// Low-level codec used for one-value decoding.
    codec: C,
}

impl<C> CodecValueDecoder<C>
where
    C: Codec,
{
    /// Creates a decoder backed by `codec`.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec used to decode one value.
    ///
    /// # Returns
    ///
    /// Returns a value decoder adapter for the supplied codec.
    ///
    /// # Panics
    ///
    /// Panics when the supplied codec declares invalid unit bounds or an
    /// inconsistent decode lifecycle bound.
    #[inline]
    #[must_use]
    pub fn new(codec: C) -> Self {
        assert_decode_lifecycle_bounds::<C>();
        Self { codec }
    }

    /// Decodes exactly one encoded value from `input`.
    ///
    /// # Parameters
    ///
    /// - `input`: Encoded units for exactly one value.
    ///
    /// # Returns
    ///
    /// Returns the decoded value.
    ///
    /// Codecs that may emit decode reset or finish values must use
    /// [`Self::decode_lifecycle`] or [`Self::decode_lifecycle_with_scratch`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::TranscodeFailure::UnsupportedDecodeLifecycleOutput`]
    /// before inspecting `input` or running codec hooks when reset or finish
    /// may emit values. Returns
    /// [`crate::TranscodeFailure::IncompleteInput`] when fewer than
    /// [`Codec::MIN_UNITS_PER_VALUE`] units are available or when
    /// [`crate::DecodeFailure::Incomplete`] is reported by the codec. In this
    /// one-shot API, incomplete input is a terminal error rather than a
    /// resumable streaming status. Returns a domain error when the wrapped
    /// codec rejects or cannot finish the input. Returns
    /// [`crate::TranscodeFailure::TrailingInput`] when a value is decoded but
    /// extra input remains.
    ///
    /// # Panics
    ///
    /// Panics when the wrapped codec reports a consumed unit count larger than
    /// the input slice length.
    pub fn decode(&mut self, input: &[C::Unit]) -> Result<C::Value, TranscodeDecodeErrorOf<C>> {
        TranscodeFailure::ensure_no_decode_lifecycle_output::<C>()?;
        let (value, reset_written, finish_written) =
            decode_exact_complete_value(&mut self.codec, input, &mut [], &mut [])?;
        debug_assert_eq!(0, reset_written);
        debug_assert_eq!(0, finish_written);
        Ok(value)
    }

    /// Decodes exactly one value and preserves all lifecycle output.
    ///
    /// # Parameters
    ///
    /// - `input`: Encoded units for exactly one value.
    ///
    /// # Returns
    ///
    /// Returns reset values, the main decoded value, and finish values in
    /// separate owned buffers.
    ///
    /// # Errors
    ///
    /// Returns a framework error when input is incomplete or has trailing
    /// units. Returns a phase-aware domain error when reset, decode, or finish
    /// fails.
    ///
    /// # Panics
    ///
    /// Panics when the wrapped codec violates its declared reset, decode, or
    /// finish bounds.
    pub fn decode_lifecycle(
        &mut self,
        input: &[C::Unit],
    ) -> Result<DecodeLifecycleOutput<C::Value>, TranscodeDecodeErrorOf<C>>
    where
        C::Value: Default,
    {
        let mut reset = Vec::new();
        reset.resize_with(C::MAX_DECODE_RESET_VALUES, C::Value::default);
        let mut finish = Vec::new();
        finish.resize_with(C::MAX_DECODE_FINISH_VALUES, C::Value::default);
        let (value, reset_written, finish_written) = self
            .decode_lifecycle_with_scratch(input, &mut reset, &mut finish)?
            .into_parts();
        reset.truncate(reset_written);
        finish.truncate(finish_written);
        Ok(DecodeLifecycleOutput::new(reset, value, finish))
    }

    /// Decodes exactly one value into separate lifecycle output buffers.
    ///
    /// # Parameters
    ///
    /// - `input`: Encoded units for exactly one value.
    /// - `reset_output`: Destination for values emitted by decode reset.
    /// - `finish_output`: Destination for values emitted by decode finish.
    ///
    /// # Returns
    ///
    /// Returns the main decoded value and the initialized lengths of both
    /// lifecycle output buffers.
    ///
    /// # Errors
    ///
    /// Returns a framework error before running any lifecycle hook when either
    /// output buffer is shorter than its corresponding codec bound. Also
    /// returns incomplete-input, trailing-input, or phase-aware domain errors.
    ///
    /// # Panics
    ///
    /// Panics when the wrapped codec violates its declared reset, decode, or
    /// finish bounds.
    pub fn decode_lifecycle_with_scratch(
        &mut self,
        input: &[C::Unit],
        reset_output: &mut [C::Value],
        finish_output: &mut [C::Value],
    ) -> Result<DecodeLifecycleProgress<C::Value>, TranscodeDecodeErrorOf<C>> {
        let (value, reset_written, finish_written) =
            decode_exact_complete_value(&mut self.codec, input, reset_output, finish_output)?;
        Ok(DecodeLifecycleProgress::new(
            value,
            reset_written,
            finish_written,
        ))
    }
}

impl<C> ValueDecoder<[C::Unit]> for CodecValueDecoder<C>
where
    C: Codec,
{
    type Output = C::Value;
    type Error = TranscodeDecodeErrorOf<C>;

    #[inline(always)]
    fn decode(&mut self, input: &[C::Unit]) -> Result<Self::Output, Self::Error> {
        self.decode(input)
    }
}

impl<C> fmt::Debug for CodecValueDecoder<C>
where
    C: Codec + fmt::Debug,
{
    /// Formats the decoder without requiring finished values to be printable.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CodecValueDecoder")
            .field("codec", &self.codec)
            .finish()
    }
}

impl<C> Default for CodecValueDecoder<C>
where
    C: Codec + Default,
{
    /// Creates a decoder from the default codec.
    #[inline(always)]
    fn default() -> Self {
        Self::new(C::default())
    }
}
