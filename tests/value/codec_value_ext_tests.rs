// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the value-level codec extension trait.

use qubit_codec::{CapacityError, Codec, CodecPhase, CodecValueExt, TranscodeError};

#[derive(Default)]
struct ResetByteCodec;

impl Codec for ResetByteCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_ENCODE_RESET_UNITS: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        Ok((input[input_index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(core::num::NonZeroUsize::MIN)
    }

    unsafe fn encode_reset(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = 0xfe;
        Ok(1)
    }
}

#[derive(Default)]
struct StatefulLifecycleCodec {
    decode_state: usize,
    encode_state: usize,
}

impl Codec for StatefulLifecycleCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_ENCODE_RESET_UNITS: usize = 1;

    const MAX_ENCODE_FLUSH_UNITS: usize = 1;

    const MAX_DECODE_FLUSH_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        let decoded = input[input_index].wrapping_sub(self.decode_state as u8);
        self.decode_state += 1;
        Ok((decoded, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[output_index] = value.wrapping_add(self.encode_state as u8);
        self.encode_state += 1;
        Ok(qubit_io::nz!(1))
    }

    unsafe fn encode_reset(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = 0xfe;
        self.encode_state = 1;
        Ok(1)
    }

    unsafe fn encode_flush(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = self.encode_state as u8;
        self.encode_state = 0;
        Ok(1)
    }

    unsafe fn decode_flush(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        output[output_index] = self.decode_state as u8;
        self.decode_state = 0;
        Ok(1)
    }
}

#[derive(Default)]
struct VariableWidthResetCodec;

impl Codec for VariableWidthResetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(2);

    fn encode_len(&self, value: &u8) -> core::num::NonZeroUsize {
        if *value < 0x80 {
            core::num::NonZeroUsize::MIN
        } else {
            qubit_io::nz!(2)
        }
    }

    const MAX_ENCODE_RESET_UNITS: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        Ok((input[input_index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        let required = self.encode_len(value).get();
        debug_assert!(
            output_index
                .checked_add(required)
                .is_some_and(|end| end <= output.len())
        );
        unsafe {
            // SAFETY: The caller guarantees that `required` units are writable
            // from `output_index`.
            *output.as_mut_ptr().add(output_index) = *value;
            if required == 2 {
                *output.as_mut_ptr().add(output_index + 1) = 0;
            }
        }
        Ok(self.encode_len(value))
    }

    unsafe fn encode_reset(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = 0xfe;
        Ok(1)
    }
}

#[derive(Default)]
struct OverflowEncodeBoundCodec;

impl Codec for OverflowEncodeBoundCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_ENCODE_RESET_UNITS: usize = usize::MAX;

    unsafe fn decode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        Ok((0, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        _value: &u8,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Ok(qubit_io::nz!(1))
    }
}

#[derive(Default)]
struct RejectingCodec;

impl Codec for RejectingCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    fn can_encode_value(&self, _value: &u8) -> bool {
        false
    }

    unsafe fn decode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        Ok((0, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        _value: &u8,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Ok(core::num::NonZeroUsize::MIN)
    }
}

#[derive(Default)]
struct FallibleCodec;

impl Codec for FallibleCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = &'static str;
    type EncodeError = &'static str;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_DECODE_FLUSH_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        Err(qubit_codec::DecodeFailure::invalid_without_consumed(
            "decode failure",
        ))
    }

    unsafe fn encode(
        &mut self,
        _value: &u8,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        Err("encode failure")
    }

    unsafe fn decode_flush(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Err("flush failure")
    }
}

#[derive(Default)]
struct EncodeFlushFallibleCodec;

impl Codec for EncodeFlushFallibleCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = &'static str;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_ENCODE_FLUSH_UNITS: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        Ok((input[input_index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(core::num::NonZeroUsize::MIN)
    }

    unsafe fn encode_flush(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Err("encode flush failure")
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
        .expect("complete value encode should fit");

    assert_eq!(2, written);
    assert_eq!([0xfe, 0x41], output);
    assert_eq!(Ok(2), CodecValueExt::max_encode_value_units(&codec));
}

#[test]
fn test_codec_value_ext_reports_complete_encode_output_bound() {
    let codec = StatefulLifecycleCodec::default();

    assert_eq!(Ok(3), codec.max_encode_value_units());
}

#[test]
fn test_codec_value_ext_reports_complete_encode_output_bound_overflow() {
    let codec = OverflowEncodeBoundCodec;

    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        codec.max_encode_value_units(),
    );
}

#[test]
fn test_codec_value_ext_encode_value_with_reset_runs_complete_lifecycle() {
    let mut codec = StatefulLifecycleCodec::default();
    let mut output = [0_u8; 3];

    let written = codec
        .encode_value_with_reset(&41, &mut output, 0)
        .expect("stateful encode should be infallible");

    assert_eq!(3, written);
    assert_eq!([0xfe, 42, 2], output);
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
        TranscodeError::InvalidOutputIndex { index: 1, len: 0 },
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
        TranscodeError::InsufficientOutput {
            output_index: 0,
            required: 3,
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

    assert_eq!(TranscodeError::UnencodableValue { input_index: 0 }, error,);
}

#[test]
fn test_codec_value_ext_encode_value_with_reset_rejects_output_length_overflow() {
    let mut codec = OverflowEncodeBoundCodec;
    let mut output = [0_u8; 1];

    let error = codec
        .encode_value_with_reset(&41, &mut output, 0)
        .expect_err("reset plus value bound should overflow");

    assert_eq!(TranscodeError::OutputLengthOverflow, error);
}

#[test]
fn test_codec_value_ext_encode_value_with_reset_wraps_encode_error() {
    let mut codec = FallibleCodec;
    let mut output = [0_u8; 1];

    let error = codec
        .encode_value_with_reset(&41, &mut output, 0)
        .expect_err("codec encode errors should be wrapped");

    assert_eq!(
        TranscodeError::domain("encode failure", CodecPhase::Main, Some(0)),
        error,
    );
}

#[test]
fn test_codec_value_ext_encode_value_with_reset_wraps_encode_flush_error() {
    let mut codec = EncodeFlushFallibleCodec;
    let mut output = [0_u8; 2];

    let error = codec
        .encode_value_with_reset(&41, &mut output, 0)
        .expect_err("codec encode flush errors should be wrapped");

    assert_eq!(
        TranscodeError::domain("encode flush failure", CodecPhase::Flush, None),
        error,
    );
}

#[test]
fn test_codec_value_ext_decode_value_with_flush_returns_value_consumed_and_flushed() {
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
        .expect_err("closed input shorter than the codec minimum is incomplete");

    assert_eq!(
        TranscodeError::IncompleteInput {
            input_index: 0,
            required: 1,
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
        TranscodeError::domain("decode failure", CodecPhase::Main, Some(0)),
        error,
    );
}

#[test]
fn test_codec_value_ext_decode_value_with_flush_maps_incomplete_failure() {
    struct IncompleteCodec;

    impl Codec for IncompleteCodec {
        type Value = u8;
        type Unit = u8;
        type DecodeError = &'static str;
        type EncodeError = core::convert::Infallible;

        const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

        const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(2);

        unsafe fn decode(
            &mut self,
            _input: &[u8],
            _input_index: usize,
        ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
        {
            Err(qubit_codec::DecodeFailure::incomplete(qubit_io::nz!(2)))
        }

        unsafe fn encode(
            &mut self,
            _value: &u8,
            _output: &mut [u8],
            _output_index: usize,
        ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
            Ok(core::num::NonZeroUsize::MIN)
        }
    }

    let mut codec = IncompleteCodec;
    let mut flushed = [0_u8; 1];
    let error = codec
        .decode_value_with_flush(&[0xaa], 0, &mut flushed, 0)
        .expect_err("codec-level incomplete failure should be mapped");

    assert_eq!(
        TranscodeError::IncompleteInput {
            input_index: 0,
            required: 2,
            available: 1,
        },
        error,
    );
}

#[test]
fn test_codec_value_ext_decode_value_with_flush_wraps_flush_error() {
    struct FlushOnlyFallibleCodec;

    impl Codec for FlushOnlyFallibleCodec {
        type Value = u8;
        type Unit = u8;
        type DecodeError = &'static str;
        type EncodeError = core::convert::Infallible;

        const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

        const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

        const MAX_DECODE_FLUSH_VALUES: usize = 1;

        unsafe fn decode(
            &mut self,
            input: &[u8],
            input_index: usize,
        ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
        {
            Ok((input[input_index], core::num::NonZeroUsize::MIN))
        }

        unsafe fn encode(
            &mut self,
            _value: &u8,
            _output: &mut [u8],
            _output_index: usize,
        ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
            Ok(core::num::NonZeroUsize::MIN)
        }

        unsafe fn decode_flush(
            &mut self,
            _output: &mut [u8],
            _output_index: usize,
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
        TranscodeError::domain("flush failure", CodecPhase::Flush, None),
        error,
    );
}

#[test]
fn test_codec_value_ext_decode_exact_value_with_flush_returns_value_and_flushed() {
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
fn test_codec_value_ext_decode_exact_value_with_flush_rejects_insufficient_flush_output() {
    let mut codec = StatefulLifecycleCodec::default();
    let mut flushed = [];

    let error = codec
        .decode_exact_value_with_flush(&[42], &mut flushed, 0)
        .expect_err("flush output must reserve the codec flush bound");

    assert_eq!(
        TranscodeError::InsufficientOutput {
            output_index: 0,
            required: 1,
            available: 0,
        },
        error,
    );
}

#[test]
fn test_codec_value_ext_decode_exact_value_with_flush_rejects_trailing_before_flush() {
    let mut codec = StatefulLifecycleCodec::default();
    let mut flushed = [0_u8; 1];

    let error = codec
        .decode_exact_value_with_flush(&[42, 43], &mut flushed, 0)
        .expect_err("exact decode should reject trailing input");

    assert_eq!(
        TranscodeError::TrailingInput {
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
        TranscodeError::InvalidInputIndex { index: 2, len: 1 },
        error,
    );
}

#[test]
fn test_codec_value_ext_decode_value_with_flush_rejects_insufficient_flush_output() {
    let mut codec = StatefulLifecycleCodec::default();
    let mut flushed = [];

    let error = codec
        .decode_value_with_flush(&[42], 0, &mut flushed, 0)
        .expect_err("flush output must reserve the codec flush bound");

    assert_eq!(
        TranscodeError::InsufficientOutput {
            output_index: 0,
            required: 1,
            available: 0,
        },
        error,
    );
}
