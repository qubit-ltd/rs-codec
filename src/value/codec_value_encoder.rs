// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Value encoder adapter backed by a low-level codec.

use super::ValueEncoder;
use crate::{
    Codec,
    CodecEncodeError,
    codec::assert_unit_bounds,
};

/// Encodes one borrowed value into owned units by using a [`Codec`].
///
/// `CodecValueEncoder` is the default bridge from the low-level unchecked
/// [`Codec`] contract to the convenience-layer [`ValueEncoder`] contract. Each
/// call emits stream-start output through [`Codec::encode_reset`], then encodes
/// one value through [`Codec::encode`], and returns the owned output truncated
/// to the units actually written.
///
/// # Type Parameters
///
/// - `C`: Low-level codec used to encode one value.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CodecValueEncoder<C> {
    /// Low-level codec used for one-value encoding.
    codec: C,
}

impl<C> CodecValueEncoder<C>
where
    C: Codec,
{
    /// Creates an encoder backed by `codec`.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec used to encode one value.
    ///
    /// # Returns
    ///
    /// Returns a value encoder adapter for the supplied codec.
    ///
    /// # Panics
    ///
    /// Panics when the supplied codec violates the
    /// [`Codec::min_units_per_value`] / [`Codec::max_units_per_value`] ordering
    /// invariant. Validating once at construction lets the hot encode path
    /// skip the check.
    #[must_use]
    #[inline]
    pub fn new(codec: C) -> Self {
        assert_unit_bounds::<C>(&codec);
        Self { codec }
    }
}

impl<C> ValueEncoder<C::Value> for CodecValueEncoder<C>
where
    C: Codec,
{
    type Output = Vec<C::Unit>;
    type Error = CodecEncodeError<C::EncodeError>;

    /// Encodes one borrowed value into owned units.
    ///
    /// # Parameters
    ///
    /// - `input`: Value to encode.
    ///
    /// # Returns
    ///
    /// Returns stream-start output followed by the units written for `input`.
    ///
    /// # Errors
    ///
    /// Returns the wrapped codec's encode error when reset output or `input`
    /// cannot be represented.
    ///
    /// # Panics
    ///
    /// Panics when the wrapped codec reports more reset output than
    /// [`Codec::max_encode_reset_units`] or a value width different from
    /// [`Codec::encode_len`].
    fn encode(
        &mut self,
        input: &C::Value,
    ) -> Result<Self::Output, Self::Error> {
        if !self.codec.can_encode_value(input) {
            return Err(CodecEncodeError::unencodable_value(0));
        }
        let units = self
            .codec
            .max_encode_reset_units()
            .checked_add(self.codec.encode_len(input).get())
            .ok_or_else(CodecEncodeError::output_length_overflow)?;
        let mut output = vec![C::Unit::default(); units];
        let written =
            self.codec.encode_value_with_reset(input, &mut output, 0)?;
        output.truncate(written);
        Ok(output)
    }
}
