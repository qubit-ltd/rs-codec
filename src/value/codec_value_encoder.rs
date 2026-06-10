// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Value encoder adapter backed by a low-level codec.

use super::ValueEncoder;
use crate::{Codec, core::assert_unit_bounds};

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

impl<C> CodecValueEncoder<C> {
    /// Creates an encoder backed by `codec`.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec used to encode one value.
    ///
    /// # Returns
    ///
    /// Returns a value encoder adapter for the supplied codec.
    #[must_use]
    #[inline(always)]
    pub const fn new(codec: C) -> Self {
        Self { codec }
    }
}

impl<C> ValueEncoder<C::Value> for CodecValueEncoder<C>
where
    C: Codec,
{
    type Output = Vec<C::Unit>;
    type Error = C::EncodeError;

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
    /// Panics when the wrapped codec reports more written units than its
    /// declared [`Codec::max_encode_reset_units`] or
    /// [`Codec::max_units_per_value`] bounds.
    fn encode(&mut self, input: &C::Value) -> Result<Self::Output, Self::Error> {
        assert_unit_bounds::<C>(&self.codec);
        let reset_units = self.codec.max_encode_reset_units();
        let value_units = self.codec.max_units_per_value().get();
        let mut output = vec![C::Unit::default(); reset_units + value_units];

        // SAFETY: The output buffer reserves the codec's declared reset-output
        // bound at index zero.
        let reset_written = unsafe { self.codec.encode_reset(&mut output, 0) }?;
        assert!(
            reset_written <= reset_units,
            "Codec::encode_reset wrote beyond allocated output",
        );

        // SAFETY: The output buffer reserves the codec's declared maximum
        // value width after any reset output.
        let value_written =
            unsafe { self.codec.encode(input, &mut output, reset_written) }?;
        assert!(
            value_written <= value_units,
            "Codec::encode wrote beyond allocated output",
        );

        output.truncate(reset_written + value_written);
        Ok(output)
    }
}
