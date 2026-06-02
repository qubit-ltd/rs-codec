/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for the codec-backed value encoder adapter.

use qubit_codec::{
    Codec,
    CodecValueEncoder,
    ValueEncoder,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct PairByteCodec;

unsafe impl Codec for PairByteCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        unsafe { core::num::NonZeroUsize::new_unchecked(2) }
    }

    unsafe fn decode_unchecked(
        &self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((value, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        debug_assert!(index + 2 <= output.len());

        // SAFETY: The caller guarantees that two bytes are writable from `index`.
        unsafe {
            *output.as_mut_ptr().add(index) = *value;
            *output.as_mut_ptr().add(index + 1) = value.wrapping_add(1);
        }
        Ok(2)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RejectOddCodec;

unsafe impl Codec for RejectOddCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = &'static str;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    unsafe fn decode_unchecked(
        &self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((value, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        if !value.is_multiple_of(2) {
            return Err("odd value");
        }
        debug_assert!(index < output.len());

        // SAFETY: The caller guarantees that `index` is writable.
        unsafe {
            *output.as_mut_ptr().add(index) = *value;
        }
        Ok(1)
    }
}

#[derive(Debug, Eq, PartialEq)]
struct NonCloneValue {
    value: u8,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct NonCloneValueCodec;

unsafe impl Codec for NonCloneValueCodec {
    type Value = NonCloneValue;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    unsafe fn decode_unchecked(
        &self,
        input: &[u8],
        index: usize,
    ) -> Result<(NonCloneValue, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((NonCloneValue { value }, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode_unchecked(
        &self,
        value: &NonCloneValue,
        output: &mut [u8],
        index: usize,
    ) -> Result<usize, Self::EncodeError> {
        debug_assert!(index < output.len());

        // SAFETY: The caller guarantees that `index` is writable.
        unsafe {
            *output.as_mut_ptr().add(index) = value.value;
        }
        Ok(1)
    }
}

#[test]
fn test_codec_value_encoder_encodes_one_value_to_owned_units() {
    let encoder = CodecValueEncoder::<PairByteCodec>::new(PairByteCodec);

    let output = ValueEncoder::<u8>::encode(&encoder, &7).expect("encoding should be infallible");

    assert_eq!(vec![7, 8], output);
}

#[test]
fn test_codec_value_encoder_accepts_non_clone_values() {
    let encoder = CodecValueEncoder::<NonCloneValueCodec>::new(NonCloneValueCodec);

    let output = ValueEncoder::<NonCloneValue>::encode(&encoder, &NonCloneValue { value: 11 })
        .expect("encoding should not require cloning the value");

    assert_eq!(vec![11], output);
}

#[test]
fn test_codec_value_encoder_propagates_encode_error() {
    let encoder = CodecValueEncoder::<RejectOddCodec>::new(RejectOddCodec);

    let error = ValueEncoder::<u8>::encode(&encoder, &7).expect_err("odd value should be rejected");

    assert_eq!("odd value", error);
}
