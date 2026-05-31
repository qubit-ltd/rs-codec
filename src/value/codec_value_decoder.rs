/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Value decoder adapter backed by a low-level codec.

use core::marker::PhantomData;

use super::ValueDecoder;
use crate::{
    Codec,
    CodecDecodeError,
    codec::debug_assert_unit_bounds,
};

/// Decodes one encoded unit slice into one owned value by using a [`Codec`].
///
/// `CodecValueDecoder` is the default bridge from the low-level unchecked
/// [`Codec`] contract to the convenience-layer [`ValueDecoder`] contract. The
/// supplied input slice must contain exactly one encoded value. Successfully
/// decoded prefixes followed by extra units are reported as trailing input.
///
/// # Type Parameters
///
/// - `C`: Low-level codec used to decode one value.
/// - `Value`: Logical value type produced by the codec.
/// - `Unit`: Encoded unit type accepted by the codec.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CodecValueDecoder<C, Value, Unit> {
    /// Low-level codec used for one-value decoding.
    codec: C,
    /// Binds the adapter to one codec value/unit contract.
    marker: PhantomData<fn(Unit) -> Value>,
}

impl<C, Value, Unit> CodecValueDecoder<C, Value, Unit> {
    /// Creates a decoder backed by `codec`.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec used to decode one value.
    ///
    /// # Returns
    ///
    /// Returns a value decoder adapter for the supplied codec.
    #[must_use]
    pub const fn new(codec: C) -> Self {
        Self {
            codec,
            marker: PhantomData,
        }
    }
}

impl<C, Value, Unit> ValueDecoder<[Unit]> for CodecValueDecoder<C, Value, Unit>
where
    C: Codec<Value, Unit>,
    Unit: Copy,
{
    type Output = Value;
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
    fn decode(&self, input: &[Unit]) -> Result<Self::Output, Self::Error> {
        debug_assert_unit_bounds::<C, Value, Unit>(&self.codec);
        let min_units = self.codec.min_units_per_value().get();
        if input.len() < min_units {
            return Err(CodecDecodeError::incomplete(0, min_units, input.len()));
        }

        // SAFETY: The input slice has at least the codec's declared minimum
        // number of readable units from index zero.
        let (value, consumed) =
            unsafe { self.codec.decode_unchecked(input, 0) }.map_err(|error| CodecDecodeError::decode(error, 0))?;
        let consumed = consumed.get();
        debug_assert!(
            consumed <= input.len(),
            "Codec::decode_unchecked consumed beyond available input",
        );

        let remaining = input.len().saturating_sub(consumed);
        if remaining == 0 {
            Ok(value)
        } else {
            Err(CodecDecodeError::trailing_input(consumed, remaining))
        }
    }
}
