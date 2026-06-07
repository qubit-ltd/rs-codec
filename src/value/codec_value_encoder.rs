// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Value encoder adapter backed by a low-level codec.

use super::ValueEncoder;
use crate::{Codec, codec::assert_unit_bounds};

/// Encodes one borrowed value into owned units by using a [`Codec`].
///
/// `CodecValueEncoder` is the default bridge from the low-level unchecked
/// [`Codec`] contract to the convenience-layer [`ValueEncoder`] contract. It
/// allocates `codec.max_units_per_value()` output units, calls
/// [`Codec::encode_unchecked`] with the borrowed value, then truncates the
/// owned output to the number of units actually written.
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
    C::Unit: Default,
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
    /// Returns the units written by the wrapped codec.
    ///
    /// # Errors
    ///
    /// Returns the wrapped codec's encode error when `input` cannot be
    /// represented.
    ///
    /// # Panics
    ///
    /// Panics when the wrapped codec reports more written units than its
    /// declared [`Codec::max_units_per_value`] bound.
    fn encode(&self, input: &C::Value) -> Result<Self::Output, Self::Error> {
        assert_unit_bounds::<C>(&self.codec);
        let mut output = vec![C::Unit::default(); self.codec.max_units_per_value().get()];
        // SAFETY: The output buffer is allocated to the codec's declared
        // maximum width, which is the safety precondition for one-value
        // encoding.
        let written = unsafe { self.codec.encode_unchecked(input, &mut output, 0) }?;
        assert!(
            written <= output.len(),
            "Codec::encode_unchecked wrote beyond allocated output",
        );
        output.truncate(written);
        Ok(output)
    }
}
