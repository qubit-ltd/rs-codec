// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Value decoder adapter backed by a low-level codec.

use core::fmt;

use super::ValueDecoder;
use crate::{
    Codec,
    TranscodeDecodeErrorOf,
    codec::{
        assert_unit_bounds,
        decode_lifecycle_scratch_len,
    },
    value::codec_value_lifecycle::decode_exact_complete_value,
};

/// Decodes one encoded unit slice into one owned value by using a [`Codec`].
///
/// `CodecValueDecoder` is the default bridge from the low-level unchecked
/// [`Codec`] contract to the convenience-layer [`ValueDecoder`] contract. The
/// supplied input slice must contain exactly one encoded value. After a
/// successful decode, the adapter calls [`Codec::decode_finish`] to reset
/// decode-side stream state for the next call.
///
/// Values emitted by [`Codec::decode_reset`] and [`Codec::decode_finish`] are
/// written into reusable lifecycle scratch storage and discarded. The adapter
/// returns only the value produced by [`Codec::decode`]. Callers for which
/// reset or finish values are semantically observable should use a streaming
/// decoder adapter instead.
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
    /// Reusable storage for values emitted by decode reset and finish hooks.
    decode_lifecycle_scratch: Vec<C::Value>,
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
    /// # Compile-Time Checks
    ///
    /// Fails to compile when the supplied codec declares zero unit bounds or
    /// when [`Codec::MIN_UNITS_PER_VALUE`] exceeds
    /// [`Codec::MAX_UNITS_PER_VALUE`].
    #[inline]
    #[must_use]
    pub fn new(codec: C) -> Self {
        assert_unit_bounds::<C>();
        Self {
            codec,
            decode_lifecycle_scratch: Vec::new(),
        }
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
    /// Values emitted by decode reset or finish are discarded; this method
    /// returns only the main value decoded from `input`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::TranscodeFailure::IncompleteInput`] when fewer than
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
    /// Panics when [`Codec::MAX_DECODE_LIFECYCLE_VALUES`] does not match the
    /// reset and finish bounds, when the wrapped codec reports a consumed unit
    /// count larger than the input slice length, or when finish output exceeds
    /// [`Codec::MAX_DECODE_FINISH_VALUES`].
    pub fn decode(
        &mut self,
        input: &[C::Unit],
    ) -> Result<C::Value, TranscodeDecodeErrorOf<C>>
    where
        C::Value: Default,
    {
        let scratch_cap = decode_lifecycle_scratch_len::<C>();
        if self.decode_lifecycle_scratch.len() < scratch_cap {
            self.decode_lifecycle_scratch
                .resize_with(scratch_cap, C::Value::default);
        }
        decode_exact_complete_value(
            &mut self.codec,
            input,
            &mut self.decode_lifecycle_scratch,
        )
    }
}

impl<C> ValueDecoder<[C::Unit]> for CodecValueDecoder<C>
where
    C: Codec,
    C::Value: Default,
{
    type Output = C::Value;
    type Error = TranscodeDecodeErrorOf<C>;

    #[inline(always)]
    fn decode(
        &mut self,
        input: &[C::Unit],
    ) -> Result<Self::Output, Self::Error> {
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
            .field(
                "decode_lifecycle_scratch_len",
                &self.decode_lifecycle_scratch.len(),
            )
            .field(
                "decode_lifecycle_scratch_capacity",
                &self.decode_lifecycle_scratch.capacity(),
            )
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
