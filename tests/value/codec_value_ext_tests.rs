// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the value-level codec extension trait.

use qubit_codec::{
    CapacityError,
    Codec,
    CodecDecodeError,
    CodecEncodeError,
    CodecValueExt,
};

#[derive(Default)]
struct ResetByteCodec;

unsafe impl Codec for ResetByteCodec {
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
        output[index] = *value;
        Ok(core::num::NonZeroUsize::MIN)
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
        Ok(qubit_io::nz!(1))
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
        Ok(qubit_io::nz!(1))
    }
}

#[derive(Default)]
struct RejectingCodec;

unsafe impl Codec for RejectingCodec {
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

    fn can_encode_value(&self, _value: &u8) -> bool {
        false
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
        Ok(core::num::NonZeroUsize::MIN)
    }
}

#[derive(Default)]
struct FallibleCodec;

unsafe impl Codec for FallibleCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = &'static str;
    type EncodeError = &'static str;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_decode_flush_values(&self) -> usize {
        1
    }

    unsafe fn decode(
        &mut self,
        _input: &[u8],
        _index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        Err("decode failure")
    }

    unsafe fn encode(
        &mut self,
        _value: &u8,
        _output: &mut [u8],
        _index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Err("encode failure")
    }

    unsafe fn decode_flush(
        &mut self,
        _output: &mut [u8],
        _index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Err("flush failure")
    }
}

#[test]
fn test_codec_value_ext_is_available_for_every_codec() {
    fn assert_value_ext<T: CodecValueExt>() {}

    assert_value_ext::<ResetByteCodec>();
}

#[test]
fn test_codec_value_ext_encodes_reset_prefixed_value() {
    let mut codec = ResetByteCodec;
    let mut output = [0_u8; 2];

    let written = codec
        .encode_value_with_reset(&0x41, &mut output, 0)
        .expect("reset-prefixed value should fit");

    assert_eq!(2, written);
    assert_eq!([0xfe, 0x41], output);
    assert_eq!(Ok(2), CodecValueExt::max_encode_value_units(&codec));
}

#[test]
fn test_codec_value_ext_reports_reset_plus_value_output_bound() {
    let codec = StatefulLifecycleCodec::default();

    assert_eq!(Ok(2), codec.max_encode_value_units());
}

#[test]
fn test_codec_value_ext_reports_reset_plus_value_output_bound_overflow() {
    let codec = OverflowEncodeBoundCodec;

    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        codec.max_encode_value_units(),
    );
}

#[test]
fn test_codec_value_ext_encode_value_with_reset_writes_reset_and_value() {
    let mut codec = StatefulLifecycleCodec::default();
    let mut output = [0_u8; 2];

    let written = codec
        .encode_value_with_reset(&41, &mut output, 0)
        .expect("stateful encode should be infallible");

    assert_eq!(2, written);
    assert_eq!([0xfe, 42], output);
}

#[test]
fn test_codec_value_ext_encode_value_with_reset_uses_exact_value_width() {
    let mut codec = VariableWidthResetCodec;
    let mut output = [0_u8; 2];

    let written = codec
        .encode_value_with_reset(&0x41, &mut output, 0)
        .expect("one-byte value should fit after reset");

    assert_eq!(2, written);
    assert_eq!([0xfe, 0x41], output);
}

#[test]
fn test_codec_value_ext_encode_value_with_reset_rejects_invalid_output_index() {
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
fn test_codec_value_ext_encode_value_with_reset_rejects_insufficient_output() {
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
fn test_codec_value_ext_encode_value_with_reset_rejects_unencodable_value() {
    let mut codec = RejectingCodec;
    let mut output = [0_u8; 1];

    let error = codec
        .encode_value_with_reset(&41, &mut output, 0)
        .expect_err("unencodable values should be rejected before encoding");

    assert_eq!(CodecEncodeError::UnencodableValue { input_index: 0 }, error);
}

#[test]
fn test_codec_value_ext_encode_value_with_reset_rejects_output_length_overflow()
{
    let mut codec = OverflowEncodeBoundCodec;
    let mut output = [0_u8; 1];

    let error = codec
        .encode_value_with_reset(&41, &mut output, 0)
        .expect_err("reset plus value bound should overflow");

    assert_eq!(CodecEncodeError::OutputLengthOverflow, error);
}

#[test]
fn test_codec_value_ext_encode_value_with_reset_wraps_encode_error() {
    let mut codec = FallibleCodec;
    let mut output = [0_u8; 1];

    let error = codec
        .encode_value_with_reset(&41, &mut output, 0)
        .expect_err("codec encode errors should be wrapped");

    assert_eq!(
        CodecEncodeError::Encode {
            source: "encode failure",
            input_index: 0,
        },
        error,
    );
}

#[test]
fn test_codec_value_ext_decode_value_with_flush_returns_value_consumed_and_flushed()
 {
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
fn test_codec_value_ext_decode_value_with_flush_rejects_incomplete_input() {
    let mut codec = StatefulLifecycleCodec::default();
    let mut flushed = [0_u8; 1];

    let error = codec
        .decode_value_with_flush(&[], 0, &mut flushed, 0)
        .expect_err(
            "closed input shorter than the codec minimum is incomplete",
        );

    assert_eq!(
        CodecDecodeError::Incomplete {
            input_index: 0,
            required_total: 1,
            available: 0,
        },
        error,
    );
}

#[test]
fn test_codec_value_ext_decode_value_with_flush_wraps_decode_error() {
    let mut codec = FallibleCodec;
    let mut flushed = [0_u8; 1];

    let error = codec
        .decode_value_with_flush(&[42], 0, &mut flushed, 0)
        .expect_err("codec decode errors should be wrapped");

    assert_eq!(
        CodecDecodeError::Decode {
            source: "decode failure",
            input_index: 0,
        },
        error,
    );
}

#[test]
fn test_codec_value_ext_decode_value_with_flush_wraps_flush_error() {
    struct FlushOnlyFallibleCodec;

    unsafe impl Codec for FlushOnlyFallibleCodec {
        type Value = u8;
        type Unit = u8;
        type DecodeError = &'static str;
        type EncodeError = core::convert::Infallible;

        fn min_units_per_value(&self) -> core::num::NonZeroUsize {
            core::num::NonZeroUsize::MIN
        }

        fn max_units_per_value(&self) -> core::num::NonZeroUsize {
            core::num::NonZeroUsize::MIN
        }

        fn max_decode_flush_values(&self) -> usize {
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
            _value: &u8,
            _output: &mut [u8],
            _index: usize,
        ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
            Ok(core::num::NonZeroUsize::MIN)
        }

        unsafe fn decode_flush(
            &mut self,
            _output: &mut [u8],
            _index: usize,
        ) -> Result<usize, Self::DecodeError> {
            Err("flush failure")
        }
    }

    let mut codec = FlushOnlyFallibleCodec;
    let mut flushed = [0_u8; 1];

    let error = codec
        .decode_value_with_flush(&[42], 0, &mut flushed, 0)
        .expect_err("decode flush errors should be wrapped after consumption");

    assert_eq!(
        CodecDecodeError::Decode {
            source: "flush failure",
            input_index: 1,
        },
        error,
    );
}

#[test]
fn test_codec_value_ext_decode_exact_value_with_flush_returns_value_and_flushed()
 {
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
fn test_codec_value_ext_decode_exact_value_with_flush_rejects_insufficient_flush_output()
 {
    let mut codec = StatefulLifecycleCodec::default();
    let mut flushed = [];

    let error = codec
        .decode_exact_value_with_flush(&[42], &mut flushed, 0)
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
fn test_codec_value_ext_decode_exact_value_with_flush_rejects_trailing_before_flush()
 {
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
fn test_codec_value_ext_decode_value_with_flush_rejects_invalid_input_index() {
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
fn test_codec_value_ext_decode_value_with_flush_rejects_insufficient_flush_output()
 {
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
