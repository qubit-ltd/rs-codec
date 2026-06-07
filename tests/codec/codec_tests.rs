// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the low-level codec trait.

use qubit_codec::{Codec, CodecValueEncoder, ValueEncoder};

#[derive(Default)]
struct ByteIncrementCodec;

unsafe impl Codec for ByteIncrementCodec {
    type Value = u8;
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
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((value.wrapping_sub(1), core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode_unchecked(
        &self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<usize, Self::EncodeError> {
        debug_assert!(index < output.len());

        // SAFETY: The caller guarantees that `index` is writable.
        unsafe {
            *output.as_mut_ptr().add(index) = value.wrapping_add(1);
        }
        Ok(1)
    }
}

#[derive(Default)]
struct InvalidBoundsCodec;

unsafe impl Codec for InvalidBoundsCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::new(2).expect("literal is non-zero")
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    unsafe fn decode_unchecked(
        &self,
        _input: &[u8],
        _index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        Ok((0, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode_unchecked(
        &self,
        _value: &u8,
        _output: &mut [u8],
        _index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Ok(0)
    }
}

#[test]
fn test_codec_trait_encodes_and_decodes_one_value() {
    let codec = ByteIncrementCodec;
    let mut output = [0_u8; 1];

    let written = unsafe { codec.encode_unchecked(&41, &mut output, 0) }
        .expect("encoding should be infallible");
    let (decoded, consumed) =
        unsafe { codec.decode_unchecked(&output, 0) }.expect("decoding should be infallible");

    assert_eq!(1, codec.min_units_per_value().get());
    assert_eq!(1, codec.max_units_per_value().get());
    assert_eq!(1, written);
    assert_eq!(1, consumed.get());
    assert_eq!(41, decoded);
}

#[test]
#[should_panic(
    expected = "Codec::min_units_per_value() must not exceed Codec::max_units_per_value()"
)]
fn test_codec_unit_bounds_panics_when_min_exceeds_max() {
    let encoder = CodecValueEncoder::new(InvalidBoundsCodec);

    let _ = encoder
        .encode(&42)
        .expect("unit-bound assertion should panic before encoding");
}
