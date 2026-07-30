// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Value encoder adapter backed by a low-level codec.

use core::fmt;

use super::ValueEncoder;
use crate::{
    CapacityError,
    Codec,
    TranscodeEncodeErrorOf,
    codec::assert_unit_bounds,
    value::codec_value_lifecycle::{
        encode_complete_value_into_reserved,
        max_complete_encode_units,
    },
};

/// Encodes one borrowed value into owned units by using a [`Codec`].
///
/// `CodecValueEncoder` is the default bridge from the low-level unchecked
/// [`Codec`] contract to the convenience-layer [`ValueEncoder`] contract. Each
/// call emits stream-start output through [`Codec::encode_reset`], encodes one
/// value through [`Codec::encode`], finishes encode-side state through
/// [`Codec::encode_finish`], and returns the owned output truncated to the
/// units actually written.
///
/// # Type Parameters
///
/// - `C`: Low-level codec used to encode one value.
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
    /// # Compile-Time Checks
    ///
    /// Fails to compile when the supplied codec declares zero unit bounds or
    /// when [`Codec::MIN_UNITS_PER_VALUE`] exceeds
    /// [`Codec::MAX_UNITS_PER_VALUE`].
    #[inline]
    #[must_use]
    pub fn new(codec: C) -> Self {
        assert_unit_bounds::<C>();
        Self { codec }
    }

    /// Encodes one borrowed value and appends the emitted units to `output`.
    ///
    /// This method is the reusable-buffer counterpart of
    /// [`ValueEncoder::encode`]. It emits stream-start output through
    /// [`Codec::encode_reset`], encodes `input` through [`Codec::encode`], and
    /// finishes encode-side state through [`Codec::encode_finish`], appending
    /// only the units actually written. When encoding fails, the vector length
    /// is restored to its original value.
    ///
    /// # Parameters
    ///
    /// - `input`: Value to encode.
    /// - `output`: Destination vector receiving appended encoded units.
    ///
    /// # Returns
    ///
    /// Returns the number of units appended to `output`.
    ///
    /// # Errors
    ///
    /// Returns the wrapped codec's encode error when reset output, `input`, or
    /// finish output cannot be represented. Returns a framework error when
    /// output length arithmetic overflows.
    ///
    /// # Panics
    ///
    /// Panics when the wrapped codec reports more reset or finish output than
    /// its declared bounds, when [`Codec::encode_len`] exceeds
    /// [`Codec::MAX_UNITS_PER_VALUE`], or when encoding writes a different
    /// value width than [`Codec::encode_len`].
    pub fn encode_into(
        &mut self,
        input: &C::Value,
        output: &mut Vec<C::Unit>,
    ) -> Result<usize, TranscodeEncodeErrorOf<C>>
    where
        C::Unit: Default,
    {
        let units = max_complete_encode_units::<C>()?;
        let original_len = output.len();
        let target_len = original_len
            .checked_add(units)
            .ok_or(CapacityError::OutputLengthOverflow)?;
        output.resize_with(target_len, C::Unit::default);

        match encode_complete_value_into_reserved(
            &mut self.codec,
            input,
            output,
            original_len,
            units,
        ) {
            Ok(written) => {
                output.truncate(original_len + written);
                Ok(written)
            }
            Err(error) => {
                output.truncate(original_len);
                Err(error)
            }
        }
    }
}

impl<C> ValueEncoder<C::Value> for CodecValueEncoder<C>
where
    C: Codec,
    C::Unit: Default,
{
    type Output = Vec<C::Unit>;
    type Error = TranscodeEncodeErrorOf<C>;

    /// Encodes one borrowed value into owned units.
    ///
    /// # Parameters
    ///
    /// - `input`: Value to encode.
    ///
    /// # Returns
    ///
    /// Returns stream-start output followed by the units written for `input`
    /// and any encode-finish output.
    ///
    /// # Errors
    ///
    /// Returns the wrapped codec's encode error when reset output, `input`, or
    /// finish output cannot be represented.
    ///
    /// # Panics
    ///
    /// Panics when the wrapped codec reports more reset or finish output than
    /// its declared bounds, or a value width different from
    /// [`Codec::encode_len`].
    fn encode(
        &mut self,
        input: &C::Value,
    ) -> Result<Self::Output, Self::Error> {
        let units = max_complete_encode_units::<C>()?;
        let mut output = Vec::with_capacity(units);
        self.encode_into(input, &mut output)?;
        Ok(output)
    }
}

impl<C> fmt::Debug for CodecValueEncoder<C>
where
    C: Codec + fmt::Debug,
{
    /// Formats the encoder without requiring finished values to be printable.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CodecValueEncoder")
            .field("codec", &self.codec)
            .finish()
    }
}

impl<C> Default for CodecValueEncoder<C>
where
    C: Codec + Default,
{
    /// Creates an encoder from the default codec.
    #[inline(always)]
    fn default() -> Self {
        Self::new(C::default())
    }
}
