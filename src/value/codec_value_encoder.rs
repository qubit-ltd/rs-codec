/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Value encoder adapter backed by a low-level codec.

use core::marker::PhantomData;

use super::ValueEncoder;
use crate::Codec;

/// Encodes one borrowed value into owned units by using a [`Codec`].
///
/// `CodecValueEncoder` is the default bridge from the low-level unchecked
/// [`Codec`] contract to the convenience-layer [`ValueEncoder`] contract. It
/// allocates `codec.max_units_per_value()` output units, calls
/// [`Codec::encode_unchecked`] with the borrowed value, then truncates the owned
/// output to the number of units actually written.
///
/// # Type Parameters
///
/// - `C`: Low-level codec used to encode one value.
/// - `Value`: Logical value type accepted by the codec.
/// - `Unit`: Encoded unit type produced by the codec.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CodecValueEncoder<C, Value, Unit> {
    /// Low-level codec used for one-value encoding.
    codec: C,
    /// Binds the adapter to one codec value/unit contract.
    marker: PhantomData<fn(Value) -> Unit>,
}

impl<C, Value, Unit> CodecValueEncoder<C, Value, Unit> {
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
    #[inline]
    pub const fn new(codec: C) -> Self {
        Self {
            codec,
            marker: PhantomData,
        }
    }

    /// Returns the wrapped codec.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the wrapped low-level codec.
    #[must_use]
    #[inline]
    pub const fn codec(&self) -> &C {
        &self.codec
    }

    /// Returns a mutable reference to the wrapped codec.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the wrapped low-level codec.
    #[must_use]
    #[inline]
    pub fn codec_mut(&mut self) -> &mut C {
        &mut self.codec
    }

    /// Consumes the adapter and returns the wrapped codec.
    ///
    /// # Returns
    ///
    /// Returns the codec supplied at construction time.
    #[must_use]
    #[inline]
    pub fn into_codec(self) -> C {
        self.codec
    }
}

impl<C, Value, Unit> ValueEncoder<Value> for CodecValueEncoder<C, Value, Unit>
where
    C: Codec<Value, Unit>,
    Unit: Copy + Default,
{
    type Output = Vec<Unit>;
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
    fn encode(&self, input: &Value) -> Result<Self::Output, Self::Error> {
        let mut output = vec![Unit::default(); self.codec.max_units_per_value()];
        // SAFETY: The output buffer is allocated to the codec's declared maximum
        // width, which is the safety precondition for one-value encoding.
        let written = unsafe { self.codec.encode_unchecked(input, &mut output, 0) }?;
        output.truncate(written);
        Ok(output)
    }
}
