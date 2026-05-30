/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for the codec-backed value decoder adapter.

use qubit_codec::{
    Codec,
    CodecDecodeError,
    CodecValueDecoder,
    DecodeErrorInfo,
    DecodeFailure,
    ValueDecoder,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct SingleByteCodec;

unsafe impl Codec<u8, u8> for SingleByteCodec {
    type DecodeError = TestDecodeError;
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
        if value == 0xff {
            Err(TestDecodeError::Invalid { consumed: 1 })
        } else {
            Ok((value, core::num::NonZeroUsize::MIN))
        }
    }

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        debug_assert!(index < output.len());

        // SAFETY: The caller guarantees that `index` is writable.
        unsafe {
            *output.as_mut_ptr().add(index) = *value;
        }
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FixedPairCodec;

unsafe impl Codec<u8, u8> for FixedPairCodec {
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::new(2).expect("literal is non-zero")
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        unsafe { core::num::NonZeroUsize::new_unchecked(2) }
    }

    unsafe fn decode_unchecked(
        &self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index + 1 < input.len());

        Ok((input[index].wrapping_add(input[index + 1]), unsafe {
            core::num::NonZeroUsize::new_unchecked(2)
        }))
    }

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        debug_assert!(index + 1 < output.len());

        output[index] = *value;
        output[index + 1] = value.wrapping_add(1);
        Ok(2)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TestDecodeError {
    Invalid { consumed: usize },
}

impl DecodeErrorInfo for TestDecodeError {
    fn failure(&self) -> DecodeFailure {
        match self {
            Self::Invalid { consumed } => DecodeFailure::Invalid { consumed: *consumed },
        }
    }
}

#[test]
fn test_codec_value_decoder_decodes_exactly_one_value() {
    let decoder = CodecValueDecoder::<SingleByteCodec, u8, u8>::new(SingleByteCodec);

    let output = ValueDecoder::<[u8]>::decode(&decoder, &[7]).expect("single byte should decode");

    assert_eq!(7, output);
    assert_eq!(&SingleByteCodec, decoder.codec());
}

#[test]
fn test_codec_value_decoder_reports_too_short_input_before_codec_call() {
    let decoder = CodecValueDecoder::<FixedPairCodec, u8, u8>::new(FixedPairCodec);

    let error = ValueDecoder::<[u8]>::decode(&decoder, &[7]).expect_err("one byte is incomplete");

    assert_eq!(
        CodecDecodeError::Incomplete {
            input_index: 0,
            required_total: 2,
            available: 1,
        },
        error,
    );
}

#[test]
fn test_codec_value_decoder_rejects_trailing_input() {
    let decoder = CodecValueDecoder::<SingleByteCodec, u8, u8>::new(SingleByteCodec);

    let error = ValueDecoder::<[u8]>::decode(&decoder, &[7, 8]).expect_err("trailing input should fail");

    assert_eq!(
        CodecDecodeError::TrailingInput {
            consumed: 1,
            remaining: 1,
        },
        error,
    );
}

#[test]
fn test_codec_value_decoder_wraps_codec_decode_error() {
    let decoder = CodecValueDecoder::<SingleByteCodec, u8, u8>::new(SingleByteCodec);

    let error = ValueDecoder::<[u8]>::decode(&decoder, &[0xff]).expect_err("0xff should fail");

    assert_eq!(
        CodecDecodeError::Decode {
            source: TestDecodeError::Invalid { consumed: 1 },
            input_index: 0,
        },
        error,
    );
}

#[test]
fn test_codec_value_decoder_exposes_wrapped_codec_accessors() {
    let mut decoder = CodecValueDecoder::<SingleByteCodec, u8, u8>::new(SingleByteCodec);

    assert_eq!(&SingleByteCodec, decoder.codec());
    assert_eq!(&mut SingleByteCodec, decoder.codec_mut());
    assert_eq!(SingleByteCodec, decoder.into_codec());
}
