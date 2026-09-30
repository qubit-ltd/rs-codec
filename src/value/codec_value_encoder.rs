// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Value encoder adapter backed by a low-level codec.

use core::fmt;

use qubit_utils::try_reserve_vec;

use super::ValueEncoder;
use crate::CapacityError;
use crate::Codec;
use crate::TranscodeEncodeErrorOf;
use crate::TranscodeFailure;
use crate::codec::assert_unit_bounds;
use crate::value::codec_value_lifecycle::encode_complete_value_into_reserved;
use crate::value::codec_value_lifecycle::max_complete_encode_units;

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
///
/// # Examples
///
/// ```
/// use core::convert::Infallible;
/// use core::num::NonZeroUsize;
///
/// use qubit_codec::Codec;
/// use qubit_codec::CodecValueEncoder;
/// use qubit_codec::DecodeFailure;
/// use qubit_codec::ValueEncoder;
///
/// # struct IdentityCodec;
/// # impl Codec for IdentityCodec {
/// #     type Value = u8;
/// #     type Unit = u8;
/// #     type DecodeError = Infallible;
/// #     type EncodeError = Infallible;
/// #     const MIN_UNITS_PER_VALUE: usize = 1;
/// #     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
/// #     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
/// #     unsafe fn decode(&mut self, input: &[u8], index: usize)
/// #         -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
/// #         Ok((input[index], NonZeroUsize::MIN))
/// #     }
/// #     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
/// #         -> Result<usize, Infallible> {
/// #         output[index] = *value;
/// #         Ok(1)
/// #     }
/// # }
/// let mut encoder = CodecValueEncoder::new(IdentityCodec);
/// assert_eq!(encoder.encode(&7_u8).unwrap(), vec![7]);
/// ```
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
    /// Fails to compile when the supplied codec declares a zero decode unit
    /// bound or when [`Codec::MIN_UNITS_PER_VALUE`] exceeds
    /// [`Codec::MAX_DECODE_UNITS_PER_VALUE`]. The encode bound may be zero for
    /// a fully buffered codec.
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
    /// The codec borrowed for as long as this adapter is shared, without
    /// cloning.
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
    /// An exclusive codec borrow that prevents simultaneous adapter operations.
    #[inline]
    #[must_use]
    pub fn codec_mut(&mut self) -> &mut C {
        &mut self.codec
    }

    /// Consumes the adapter and returns its wrapped codec.
    ///
    /// # Returns
    ///
    /// The owned codec in its current lifecycle state, without resetting it.
    #[inline]
    #[must_use]
    pub fn into_codec(self) -> C {
        self.codec
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
    /// output length arithmetic overflows or the output allocation fails.
    ///
    /// # Panics
    ///
    /// Panics when the wrapped codec reports more reset or finish output than
    /// its declared bounds, when [`Codec::encode_len`] exceeds
    /// [`Codec::MAX_ENCODE_UNITS_PER_VALUE`], or when encoding writes a
    /// different value width than [`Codec::encode_len`].
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
        try_reserve_vec(output, units).map_err(|_| TranscodeFailure::allocation_failed())?;
        output.resize_with(target_len, C::Unit::default);

        match encode_complete_value_into_reserved(&mut self.codec, input, output, original_len, units) {
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
    /// Owned units containing the complete reset/value/finish representation.
    type Output = Vec<C::Unit>;
    /// Codec-domain failures or framework capacity/allocation failures.
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
    /// finish output cannot be represented. Also returns framework capacity
    /// overflow or allocation failures while preparing the output vector.
    ///
    /// # Panics
    ///
    /// Panics when the wrapped codec reports more reset or finish output than
    /// its declared bounds, or a value width different from
    /// [`Codec::encode_len`].
    fn encode(&mut self, input: &C::Value) -> Result<Self::Output, Self::Error> {
        let mut output = Vec::new();
        self.encode_into(input, &mut output)?;
        Ok(output)
    }
}

impl<C> fmt::Debug for CodecValueEncoder<C>
where
    C: Codec + fmt::Debug,
{
    /// Formats the encoder without requiring finished values to be printable.
    ///
    /// # Parameters
    ///
    /// - `formatter`: Destination receiving the wrapped codec description.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the description has been written.
    ///
    /// # Errors
    ///
    /// Propagates errors from the destination formatter.
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
    ///
    /// # Returns
    ///
    /// An adapter owning `C::default()`, with the same compile-time unit-bound
    /// checks as [`Self::new`].
    fn default() -> Self {
        Self::new(C::default())
    }
}
