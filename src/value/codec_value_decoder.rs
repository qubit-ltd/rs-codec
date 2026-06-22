// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Value decoder adapter backed by a low-level codec.

use super::ValueDecoder;
use crate::{
    Codec,
    CodecDecodeError,
    CodecValueExt,
    codec::assert_unit_bounds,
};

/// Decodes one encoded unit slice into one owned value by using a [`Codec`].
///
/// `CodecValueDecoder` is the default bridge from the low-level unchecked
/// [`Codec`] contract to the convenience-layer [`ValueDecoder`] contract. The
/// supplied input slice must contain exactly one encoded value. After a
/// successful decode, the adapter calls [`Codec::decode_flush`] to reset
/// decode-side stream state for the next call.
///
/// # Type Parameters
///
/// - `C`: Low-level codec used to decode one value.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CodecValueDecoder<C> {
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
    /// Panics when the supplied codec violates the
    /// [`Codec::min_units_per_value`] / [`Codec::max_units_per_value`] ordering
    /// invariant. Validating once at construction lets the hot decode path
    /// skip the check.
    #[must_use]
    #[inline]
    pub fn new(codec: C) -> Self {
        assert_unit_bounds::<C>(&codec);
        Self { codec }
    }
}

impl<C> ValueDecoder<[C::Unit]> for CodecValueDecoder<C>
where
    C: Codec,
    C::Value: Default,
{
    type Output = C::Value;
    type Error = CodecDecodeError<C::DecodeError>;

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
    /// # Errors
    ///
    /// Returns [`CodecDecodeError::Incomplete`] when fewer than
    /// [`Codec::min_units_per_value`] units are available. Returns
    /// [`CodecDecodeError::Decode`] when the wrapped codec rejects the input.
    /// Returns [`CodecDecodeError::TrailingInput`] when a value is decoded but
    /// extra input remains.
    ///
    /// # Panics
    ///
    /// Panics when the wrapped codec reports a consumed unit count larger than
    /// the input slice length, or when flush output exceeds
    /// [`Codec::max_decode_flush_values`].
    fn decode(
        &mut self,
        input: &[C::Unit],
    ) -> Result<Self::Output, Self::Error> {
        let flush_cap = self.codec.max_decode_flush_values();
        let (value, _) = if flush_cap == 0 {
            self.codec
                .decode_exact_value_with_flush(input, &mut [], 0)?
        } else {
            let mut scratch = Vec::with_capacity(flush_cap);
            scratch.resize_with(flush_cap, C::Value::default);
            self.codec
                .decode_exact_value_with_flush(input, &mut scratch, 0)?
        };

        Ok(value)
    }
}
