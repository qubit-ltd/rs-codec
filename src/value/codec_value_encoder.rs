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
    CodecPhase,
    CodecValueExt,
    TranscodeError,
    codec::assert_unit_bounds,
};

/// Encodes one borrowed value into owned units by using a [`Codec`].
///
/// `CodecValueEncoder` is the default bridge from the low-level unchecked
/// [`Codec`] contract to the convenience-layer [`ValueEncoder`] contract. Each
/// call emits stream-start output through [`Codec::encode_reset`], encodes one
/// value through [`Codec::encode`], flushes encode-side state through
/// [`Codec::encode_flush`], and returns the owned output truncated to the units
/// actually written.
///
/// # Type Parameters
///
/// - `C`: Low-level codec used to encode one value.
#[derive(Debug, Default)]
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
    /// In debug builds, panics when the supplied codec violates the
    /// [`Codec::MIN_UNITS_PER_VALUE`] / [`Codec::MAX_UNITS_PER_VALUE`] ordering
    /// invariant. Release builds skip this check because the invariant is the
    /// responsibility of the [`Codec`] implementation.
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
    /// flushes encode-side state through [`Codec::encode_flush`], appending
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
    /// flush output cannot be represented. Returns a framework error when
    /// output length arithmetic overflows.
    ///
    /// # Panics
    ///
    /// Panics when the wrapped codec reports more reset or flush output than
    /// its declared bounds, or a value width different from
    /// [`Codec::encode_len`].
    pub fn encode_into(
        &mut self,
        input: &C::Value,
        output: &mut Vec<C::Unit>,
    ) -> Result<usize, TranscodeError<C::EncodeError>>
    where
        C::Unit: Default,
    {
        if !self.codec.can_encode_value(input) {
            return Err(TranscodeError::unencodable_value(0));
        }
        let units = C::MAX_ENCODE_RESET_UNITS
            .checked_add(self.codec.encode_len(input).get())
            .and_then(|units| units.checked_add(C::MAX_ENCODE_FLUSH_UNITS))
            .ok_or_else(TranscodeError::output_length_overflow)?;
        let original_len = output.len();
        let target_len = original_len
            .checked_add(units)
            .ok_or(TranscodeError::output_length_overflow())?;
        output.resize_with(target_len, C::Unit::default);

        match self
            .codec
            .encode_value_with_reset(input, output, original_len)
        {
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
    type Error = TranscodeError<C::EncodeError>;
    type DomainError = C::EncodeError;

    /// Maps a codec-domain error from the main encode phase.
    #[inline(always)]
    fn map_error(&self, error: Self::DomainError) -> Self::Error {
        TranscodeError::domain(error, CodecPhase::Main, Some(0))
    }

    /// Encodes one borrowed value into owned units.
    ///
    /// # Parameters
    ///
    /// - `input`: Value to encode.
    ///
    /// # Returns
    ///
    /// Returns stream-start output followed by the units written for `input`
    /// and any encode-flush output.
    ///
    /// # Errors
    ///
    /// Returns the wrapped codec's encode error when reset output, `input`, or
    /// flush output cannot be represented.
    ///
    /// # Panics
    ///
    /// Panics when the wrapped codec reports more reset or flush output than
    /// its declared bounds, or a value width different from
    /// [`Codec::encode_len`].
    fn encode(
        &mut self,
        input: &C::Value,
    ) -> Result<Self::Output, Self::Error> {
        let units = self
            .codec
            .max_encode_value_units()
            .map_err(|_| TranscodeError::output_length_overflow())?;
        let mut output = Vec::with_capacity(units);
        self.encode_into(input, &mut output)?;
        Ok(output)
    }
}
