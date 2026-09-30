// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Value decoder adapter backed by a low-level codec.

use core::fmt;

use qubit_utils::try_reserve_vec;

use super::DecodeLifecycleOutput;
use super::DecodeLifecycleProgress;
use super::ValueDecoder;
use crate::Codec;
use crate::TranscodeDecodeErrorOf;
use crate::TranscodeFailure;
use crate::codec::assert_unit_bounds;
use crate::value::codec_value_lifecycle::decode_exact_complete_value;

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
///
/// # Examples
///
/// ```
/// use std::convert::Infallible;
/// use std::num::NonZeroUsize;
///
/// use qubit_codec::Codec;
/// use qubit_codec::CodecValueDecoder;
/// use qubit_codec::DecodeFailure;
///
/// struct Identity;
/// impl Codec for Identity {
///     type Value = u8;
///     type Unit = u8;
///     type DecodeError = Infallible;
///     type EncodeError = Infallible;
///     const MIN_UNITS_PER_VALUE: usize = 1;
///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
///     unsafe fn decode(&mut self, input: &[u8], index: usize)
///         -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
///         Ok((input[index], NonZeroUsize::new(1).unwrap()))
///     }
///     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
///         -> Result<usize, Infallible> {
///         output[index] = *value;
///         Ok(1)
///     }
/// }
/// let mut decoder = CodecValueDecoder::new(Identity);
/// assert_eq!(decoder.decode(&[7]).unwrap(), 7);
/// ```
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
    /// Fails at compile time if codec decode bounds are zero or the minimum
    /// exceeds the maximum.
    #[inline]
    #[must_use]
    pub fn new(codec: C) -> Self {
        assert_unit_bounds::<C>();
        Self { codec }
    }

    /// Returns a shared reference to the wrapped codec.
    ///
    /// # Returns
    ///
    /// The codec borrowed for as long as the adapter is borrowed.
    #[inline]
    #[must_use]
    pub const fn codec(&self) -> &C {
        &self.codec
    }

    /// Returns a mutable reference to the wrapped codec.
    ///
    /// The next value operation starts a fresh codec lifecycle, so mutations
    /// are applied to the following operation.
    ///
    /// # Returns
    ///
    /// An exclusive borrow of the codec without allocating.
    #[inline]
    #[must_use]
    pub fn codec_mut(&mut self) -> &mut C {
        &mut self.codec
    }

    /// Consumes the adapter and returns its wrapped codec.
    ///
    /// # Returns
    ///
    /// The owned codec, preserving its most recent lifecycle state.
    #[inline]
    #[must_use]
    pub fn into_codec(self) -> C {
        self.codec
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
    /// the input slice length or [`Codec::MAX_DECODE_UNITS_PER_VALUE`].
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
    /// fails. Returns an allocation failure when either lifecycle buffer cannot
    /// reserve its declared capacity.
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
        try_reserve_vec(&mut reset, C::MAX_DECODE_RESET_VALUES).map_err(|_| TranscodeFailure::allocation_failed())?;
        reset.resize_with(C::MAX_DECODE_RESET_VALUES, C::Value::default);
        let mut finish = Vec::new();
        try_reserve_vec(&mut finish, C::MAX_DECODE_FINISH_VALUES).map_err(|_| TranscodeFailure::allocation_failed())?;
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
        Ok(DecodeLifecycleProgress::new(value, reset_written, finish_written))
    }
}

impl<C> ValueDecoder<[C::Unit]> for CodecValueDecoder<C>
where
    C: Codec,
{
    /// Owned value returned after the complete decode lifecycle.
    type Output = C::Value;
    /// Framework or phase-aware codec failure from one-value decoding.
    type Error = TranscodeDecodeErrorOf<C>;

    /// Decodes one value through the owned-value adapter.
    ///
    /// # Parameters
    ///
    /// - `input`: Complete encoded representation of exactly one value.
    ///
    /// # Returns
    ///
    /// The owned decoded value after finishing the codec lifecycle.
    ///
    /// # Errors
    ///
    /// Returns the same unsupported-lifecycle, incomplete-input,
    /// trailing-input, and codec errors as [`Self::decode`].
    ///
    /// # Panics
    ///
    /// Panics if the codec reports consumption beyond its bounds or the input.
    fn decode(&mut self, input: &[C::Unit]) -> Result<Self::Output, Self::Error> {
        self.decode(input)
    }
}

impl<C> fmt::Debug for CodecValueDecoder<C>
where
    C: Codec + fmt::Debug,
{
    /// Formats the decoder without requiring finished values to be printable.
    ///
    /// # Parameters
    ///
    /// - `formatter`: Destination and formatting options.
    ///
    /// # Returns
    ///
    /// Unit on successful formatting.
    ///
    /// # Errors
    ///
    /// Returns the formatter error if writing the codec representation fails.
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
    ///
    /// # Returns
    ///
    /// An adapter owning `C::default()`.
    ///
    /// # Panics
    ///
    /// Fails at compile time for invalid codec decode bounds, as [`Self::new`].
    /// May panic if the codec's own default constructor panics.
    fn default() -> Self {
        Self::new(C::default())
    }
}
