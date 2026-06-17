// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the low-level codec trait.

use qubit_codec::{
    CapacityError, Codec, CodecDecodeError, CodecEncodeError, CodecValueEncoder, ValueEncoder, nz,
};

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

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((value.wrapping_sub(1), core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index < output.len());

        // SAFETY: The caller guarantees that `index` is writable.
        unsafe {
            *output.as_mut_ptr().add(index) = value.wrapping_add(1);
        }
        Ok(nz!(1))
    }
}

#[derive(Default)]
struct StatefulLifecycleCodec {
    decode_state: usize,
    encode_state: usize,
}

unsafe impl Codec for StatefulLifecycleCodec {
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

    fn max_encode_reset_units(&self) -> usize {
        1
    }

    fn max_decode_flush_values(&self) -> usize {
        1
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        let decoded = input[index].wrapping_sub(self.decode_state as u8);
        self.decode_state += 1;
        Ok((decoded, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[index] = value.wrapping_add(self.encode_state as u8);
        self.encode_state += 1;
        Ok(nz!(1))
    }

    unsafe fn encode_reset(
        &mut self,
        output: &mut [u8],
        index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[index] = 0xfe;
        self.encode_state = 1;
        Ok(1)
    }

    unsafe fn decode_flush(
        &mut self,
        output: &mut [u8],
        index: usize,
    ) -> Result<usize, Self::DecodeError> {
        output[index] = self.decode_state as u8;
        self.decode_state = 0;
        Ok(1)
    }
}

#[derive(Default)]
struct VariableWidthResetCodec;

unsafe impl Codec for VariableWidthResetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::new(2).expect("literal is non-zero")
    }

    fn encode_len(&self, value: &u8) -> core::num::NonZeroUsize {
        if *value < 0x80 {
            core::num::NonZeroUsize::MIN
        } else {
            core::num::NonZeroUsize::new(2).expect("literal is non-zero")
        }
    }

    fn max_encode_reset_units(&self) -> usize {
        1
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        Ok((input[index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        let required = self.encode_len(value).get();
        debug_assert!(
            index
                .checked_add(required)
                .is_some_and(|end| end <= output.len())
        );
        unsafe {
            // SAFETY: The caller guarantees that `required` units are writable
            // from `index`.
            *output.as_mut_ptr().add(index) = *value;
            if required == 2 {
                *output.as_mut_ptr().add(index + 1) = 0;
            }
        }
        Ok(self.encode_len(value))
    }

    unsafe fn encode_reset(
        &mut self,
        output: &mut [u8],
        index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[index] = 0xfe;
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

    unsafe fn decode(
        &mut self,
        _input: &[u8],
        _index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        Ok((0, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        _value: &u8,
        _output: &mut [u8],
        _index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Ok(nz!(1))
    }
}

#[derive(Default)]
struct OverflowEncodeBoundCodec;

unsafe impl Codec for OverflowEncodeBoundCodec {
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

    fn max_encode_reset_units(&self) -> usize {
        usize::MAX
    }

    unsafe fn decode(
        &mut self,
        _input: &[u8],
        _index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        Ok((0, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        _value: &u8,
        _output: &mut [u8],
        _index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Ok(nz!(1))
    }
}

#[test]
fn test_codec_trait_encodes_and_decodes_one_value() {
    let mut codec = ByteIncrementCodec;
    let mut output = [0_u8; 1];

    let written =
        unsafe { codec.encode(&41, &mut output, 0) }.expect("encoding should be infallible");
    let (decoded, consumed) =
        unsafe { Codec::decode(&mut codec, &output, 0) }.expect("decoding should be infallible");

    assert_eq!(1, codec.min_units_per_value().get());
    assert_eq!(1, codec.max_units_per_value().get());
    assert!(codec.can_encode_value(&41));
    assert_eq!(1, written.get());
    assert_eq!(1, consumed.get());
    assert_eq!(41, decoded);
}

#[test]
fn test_codec_trait_exposes_stateful_lifecycle_methods() {
    let mut codec = StatefulLifecycleCodec::default();
    let mut encoded = [0_u8; 2];
    let mut flushed = [0_u8; 1];

    let reset_written =
        unsafe { codec.encode_reset(&mut encoded, 0) }.expect("reset should be infallible");
    let value_written = unsafe { codec.encode(&41, &mut encoded, reset_written) }
        .expect("encoding should be infallible");

    assert_eq!(1, reset_written);
    assert_eq!(1, value_written.get());
    assert_eq!([0xfe, 42], encoded);
    assert_eq!(2, codec.encode_state);

    let (decoded, consumed) =
        unsafe { Codec::decode(&mut codec, &[42], 0) }.expect("decoding should be infallible");
    let flushed_len =
        unsafe { codec.decode_flush(&mut flushed, 0) }.expect("flush should be infallible");

    assert_eq!(42, decoded);
    assert_eq!(1, consumed.get());
    assert_eq!(1, flushed_len);
    assert_eq!([1], flushed);
    assert_eq!(0, codec.decode_state);
}

#[test]
fn test_codec_reports_reset_plus_value_output_bound() {
    let codec = StatefulLifecycleCodec::default();

    assert_eq!(Ok(2), codec.max_encode_value_units());
}

#[test]
fn test_codec_reports_reset_plus_value_output_bound_overflow() {
    let codec = OverflowEncodeBoundCodec;

    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        codec.max_encode_value_units(),
    );
}

#[test]
fn test_codec_encode_value_with_reset_writes_reset_and_value() {
    let mut codec = StatefulLifecycleCodec::default();
    let mut output = [0_u8; 2];

    let written = codec
        .encode_value_with_reset(&41, &mut output, 0)
        .expect("stateful encode should be infallible");

    assert_eq!(2, written);
    assert_eq!([0xfe, 42], output);
}

#[test]
fn test_codec_encode_value_with_reset_uses_exact_value_width() {
    let mut codec = VariableWidthResetCodec;
    let mut output = [0_u8; 2];

    let written = codec
        .encode_value_with_reset(&0x41, &mut output, 0)
        .expect("one-byte value should fit after reset");

    assert_eq!(2, written);
    assert_eq!([0xfe, 0x41], output);
}

#[test]
fn test_codec_encode_value_with_reset_rejects_invalid_output_index() {
    let mut codec = StatefulLifecycleCodec::default();
    let mut output = [];

    let error = codec
        .encode_value_with_reset(&41, &mut output, 1)
        .expect_err("output index beyond the slice should fail");

    assert_eq!(
        CodecEncodeError::InvalidOutputIndex { index: 1, len: 0 },
        error,
    );
}

#[test]
fn test_codec_encode_value_with_reset_rejects_insufficient_output() {
    let mut codec = StatefulLifecycleCodec::default();
    let mut output = [0_u8; 1];

    let error = codec
        .encode_value_with_reset(&41, &mut output, 0)
        .expect_err("output must hold reset bytes and encoded value");

    assert_eq!(
        CodecEncodeError::InsufficientOutput {
            output_index: 0,
            required: 2,
            available: 1,
        },
        error,
    );
}

#[test]
fn test_codec_decode_value_with_flush_returns_value_consumed_and_flushed() {
    let mut codec = StatefulLifecycleCodec::default();
    let mut flushed = [0_u8; 1];

    let (value, consumed, flushed_len) = codec
        .decode_value_with_flush(&[42], 0, &mut flushed, 0)
        .expect("stateful decode should be infallible");

    assert_eq!(42, value);
    assert_eq!(1, consumed.get());
    assert_eq!(1, flushed_len);
    assert_eq!([1], flushed);
}

#[test]
fn test_codec_decode_exact_value_with_flush_returns_value_and_flushed() {
    let mut codec = StatefulLifecycleCodec::default();
    let mut flushed = [0_u8; 1];

    let (value, flushed_len) = codec
        .decode_exact_value_with_flush(&[42], &mut flushed, 0)
        .expect("stateful decode should be infallible");

    assert_eq!(42, value);
    assert_eq!(1, flushed_len);
    assert_eq!([1], flushed);
}

#[test]
fn test_codec_decode_exact_value_with_flush_rejects_trailing_before_flush() {
    let mut codec = StatefulLifecycleCodec::default();
    let mut flushed = [0_u8; 1];

    let error = codec
        .decode_exact_value_with_flush(&[42, 43], &mut flushed, 0)
        .expect_err("exact decode should reject trailing input");

    assert_eq!(
        CodecDecodeError::TrailingInput {
            consumed: 1,
            remaining: 1,
        },
        error,
    );
    assert_eq!(1, codec.decode_state);
    assert_eq!([0], flushed);
}

#[test]
fn test_codec_decode_value_with_flush_rejects_invalid_input_index() {
    let mut codec = StatefulLifecycleCodec::default();
    let mut flushed = [0_u8; 1];

    let error = codec
        .decode_value_with_flush(&[42], 2, &mut flushed, 0)
        .expect_err("input index beyond the slice should fail");

    assert_eq!(
        CodecDecodeError::InvalidInputIndex { index: 2, len: 1 },
        error,
    );
}

#[test]
fn test_codec_decode_value_with_flush_rejects_insufficient_flush_output() {
    let mut codec = StatefulLifecycleCodec::default();
    let mut flushed = [];

    let error = codec
        .decode_value_with_flush(&[42], 0, &mut flushed, 0)
        .expect_err("flush output must reserve the codec flush bound");

    assert_eq!(
        CodecDecodeError::InsufficientOutput {
            output_index: 0,
            required: 1,
            available: 0,
        },
        error,
    );
}

#[test]
#[should_panic(
    expected = "Codec::min_units_per_value() must not exceed Codec::max_units_per_value()"
)]
fn test_codec_unit_bounds_panics_when_min_exceeds_max() {
    let mut encoder = CodecValueEncoder::new(InvalidBoundsCodec);

    let _ = encoder
        .encode(&42)
        .expect("unit-bound assertion should panic before encoding");
}
