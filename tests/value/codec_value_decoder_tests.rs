// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the codec-backed value decoder adapter.

use qubit_codec::{Codec, CodecValueDecoder, TranscodeDecodeError, TranscodeFailure, ValueDecoder};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct SingleByteCodec;

impl Codec for SingleByteCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index < input.len());

        // SAFETY: The caller guarantees that `input_index` is readable.
        let value = unsafe { *input.as_ptr().add(input_index) };
        if value == 0xff {
            Err(qubit_codec::DecodeFailure::invalid(
                TestDecodeError::Invalid { consumed: 1 },
                core::num::NonZeroUsize::MIN,
            ))
        } else {
            Ok((value, core::num::NonZeroUsize::MIN))
        }
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        debug_assert!(output_index < output.len());

        // SAFETY: The caller guarantees that `output_index` is writable.
        unsafe {
            *output.as_mut_ptr().add(output_index) = *value;
        }
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FixedPairCodec;

impl Codec for FixedPairCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 2;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index + 1 < input.len());

        Ok((
            input[input_index].wrapping_add(input[input_index + 1]),
            unsafe { core::num::NonZeroUsize::new_unchecked(2) },
        ))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        debug_assert!(output_index + 1 < output.len());

        output[output_index] = *value;
        output[output_index + 1] = value.wrapping_add(1);
        Ok(2)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverconsumingCodec;

impl Codec for OverconsumingCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index < input.len());

        Ok((input[input_index], crate::nz(2)))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        debug_assert!(output_index < output.len());

        output[output_index] = *value;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverreportingDecodeResetCodec;

impl Codec for OverreportingDecodeResetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    unsafe fn decode_reset(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Ok(1)
    }

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
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverreportingDecodeFinishCodec;

impl Codec for OverreportingDecodeFinishCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

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
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(1)
    }

    unsafe fn decode_finish(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TestDecodeError {
    Invalid { consumed: usize },
    ResetFailed,
    FinishFailed,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetFailDecodeCodec {
    fail_reset: bool,
}

impl Codec for ResetFailDecodeCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_RESET_VALUES: usize = 1;

    unsafe fn decode_reset(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        if self.fail_reset {
            Err(TestDecodeError::ResetFailed)
        } else {
            Ok(0)
        }
    }

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
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct IncompleteDecodeCodec;

impl Codec for IncompleteDecodeCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>> {
        Err(qubit_codec::DecodeFailure::incomplete(crate::nz(2)))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FinishFailStatelessCodec {
    fail_finish: bool,
}

impl Codec for FinishFailStatelessCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

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
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(1)
    }

    unsafe fn decode_finish(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        if self.fail_finish {
            Err(TestDecodeError::FinishFailed)
        } else {
            Ok(0)
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FinishFailStatefulCodec {
    fail_finish: bool,
}

impl Codec for FinishFailStatefulCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_FINISH_VALUES: usize = 1;

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
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(1)
    }

    unsafe fn decode_finish(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        if self.fail_finish {
            Err(TestDecodeError::FinishFailed)
        } else {
            Ok(0)
        }
    }
}

#[derive(Default)]
struct StatefulLifecycleCodec {
    decode_state: usize,
}

impl Codec for StatefulLifecycleCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_FINISH_VALUES: usize = 1;

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
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(1)
    }

    unsafe fn decode_finish(
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
pub(super) struct ResetSensitiveLifecycleCodec {
    decode_state: usize,
}

impl Codec for ResetSensitiveLifecycleCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_RESET_VALUES: usize = 1;

    const MAX_DECODE_FINISH_VALUES: usize = 1;

    unsafe fn decode_reset(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        output[output_index] = 0xfe;
        self.decode_state = 1;
        Ok(1)
    }

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
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(1)
    }

    unsafe fn decode_finish(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        output[output_index] = self.decode_state as u8;
        self.decode_state = 0;
        Ok(1)
    }
}

#[derive(Debug, Eq, PartialEq)]
struct CountingFinishValue(u8);

static COUNTING_FINISH_DEFAULTS: AtomicUsize = AtomicUsize::new(0);

impl Default for CountingFinishValue {
    fn default() -> Self {
        COUNTING_FINISH_DEFAULTS.fetch_add(1, Ordering::SeqCst);
        Self(0)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CountingFinishCodec;

impl Codec for CountingFinishCodec {
    type Value = CountingFinishValue;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_FINISH_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (CountingFinishValue, core::num::NonZeroUsize),
        qubit_codec::DecodeFailure<Self::DecodeError>,
    > {
        Ok((
            CountingFinishValue(input[input_index]),
            core::num::NonZeroUsize::MIN,
        ))
    }

    unsafe fn encode(
        &mut self,
        value: &CountingFinishValue,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = value.0;
        Ok(1)
    }

    unsafe fn decode_finish(
        &mut self,
        output: &mut [CountingFinishValue],
        output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        output[output_index] = CountingFinishValue(0);
        Ok(1)
    }
}

#[test]
fn test_codec_value_decoder_finishes_decode_state_after_success() {
    let mut decoder =
        CodecValueDecoder::<StatefulLifecycleCodec>::new(StatefulLifecycleCodec::default());

    let first = decoder
        .decode_lifecycle(&[42])
        .expect("first decode should succeed");
    let second = decoder
        .decode_lifecycle(&[42])
        .expect("second decode should succeed");

    assert_eq!(&42, first.value());
    assert_eq!(&[1], first.finish());
    assert_eq!(&42, second.value());
    assert_eq!(&[1], second.finish());
}

#[test]
fn test_codec_value_decoder_runs_complete_decode_lifecycle() {
    let mut decoder = CodecValueDecoder::<ResetSensitiveLifecycleCodec>::new(
        ResetSensitiveLifecycleCodec::default(),
    );

    let first = decoder
        .decode_lifecycle(&[43])
        .expect("first decode should succeed");
    let second = decoder
        .decode_lifecycle(&[44])
        .expect("second decode should succeed");

    assert_eq!(&42, first.value());
    assert_eq!(&43, second.value());
}

#[test]
fn test_codec_value_decoder_rejects_short_reset_output_before_input_and_hooks() {
    let mut decoder =
        CodecValueDecoder::<ResetFailDecodeCodec>::new(ResetFailDecodeCodec { fail_reset: true });

    let error = decoder
        .decode_lifecycle_with_scratch(&[], &mut [], &mut [])
        .expect_err("short reset output must be rejected first");

    assert_eq!(
        Some(&TranscodeFailure::InsufficientOutput {
            output_index: 0,
            required: 1,
            available: 0,
        }),
        error.failure_ref(),
    );
}

#[test]
fn test_codec_value_decoder_runs_reset_before_input_validation() {
    let mut decoder =
        CodecValueDecoder::<ResetFailDecodeCodec>::new(ResetFailDecodeCodec { fail_reset: true });
    let mut reset_output = [0_u8; 1];

    let error = decoder
        .decode_lifecycle_with_scratch(&[], &mut reset_output, &mut [])
        .expect_err("decode reset failure must precede incomplete input");

    assert_eq!(
        TranscodeDecodeError::domain_reset(TestDecodeError::ResetFailed),
        error,
    );
}

#[test]
fn test_codec_value_decoder_rejects_short_finish_output_before_decode() {
    let mut decoder = CodecValueDecoder::<FinishFailStatefulCodec>::new(FinishFailStatefulCodec {
        fail_finish: true,
    });

    let error = decoder
        .decode_lifecycle_with_scratch(&[7], &mut [], &mut [])
        .expect_err("short finish output must be rejected first");

    assert_eq!(
        Some(&TranscodeFailure::InsufficientOutput {
            output_index: 0,
            required: 1,
            available: 0,
        }),
        error.failure_ref(),
    );
}

#[test]
fn test_codec_value_decoder_rejects_lifecycle_output_before_input_validation() {
    let mut decoder = CodecValueDecoder::<ResetSensitiveLifecycleCodec>::new(
        ResetSensitiveLifecycleCodec::default(),
    );

    let error = decoder
        .decode(&[])
        .expect_err("strict decoding must reject lifecycle output");

    assert_eq!(
        Some(&TranscodeFailure::UnsupportedDecodeLifecycleOutput {
            reset_bound: 1,
            finish_bound: 1,
        }),
        error.failure_ref(),
    );
}

#[test]
fn test_codec_value_decoder_reuses_caller_decode_lifecycle_scratch() {
    let mut decoder = CodecValueDecoder::<CountingFinishCodec>::new(CountingFinishCodec);
    let mut finish_output = [CountingFinishValue(0)];
    COUNTING_FINISH_DEFAULTS.store(0, Ordering::SeqCst);

    let first = decoder
        .decode_lifecycle_with_scratch(&[7], &mut [], &mut finish_output)
        .expect("first decode should succeed");
    let second = decoder
        .decode_lifecycle_with_scratch(&[8], &mut [], &mut finish_output)
        .expect("second decode should succeed");

    assert_eq!(&CountingFinishValue(7), first.value());
    assert_eq!(&CountingFinishValue(8), second.value());
    assert_eq!(1, first.finish_written());
    assert_eq!(1, second.finish_written());
    assert_eq!([CountingFinishValue(0)], finish_output);
    assert_eq!(0, COUNTING_FINISH_DEFAULTS.load(Ordering::SeqCst));
}

#[test]
fn test_codec_value_decoder_decodes_exactly_one_value() {
    let mut decoder = CodecValueDecoder::<SingleByteCodec>::new(SingleByteCodec);

    let output =
        ValueDecoder::<[u8]>::decode(&mut decoder, &[7]).expect("single byte should decode");

    assert_eq!(7, output);
}

#[test]
fn test_codec_value_decoder_default_and_debug_do_not_require_value_debug() {
    let mut decoder = CodecValueDecoder::<SingleByteCodec>::default();

    let output =
        ValueDecoder::<[u8]>::decode(&mut decoder, &[9]).expect("default decoder should decode");
    let debug = format!("{decoder:?}");

    assert_eq!(9, output);
    assert!(debug.contains("CodecValueDecoder"));
    assert!(debug.contains("codec"));
}

#[test]
fn test_codec_value_decoder_reports_too_short_input_before_main_decode_call() {
    let mut decoder = CodecValueDecoder::<FixedPairCodec>::new(FixedPairCodec);

    let error =
        ValueDecoder::<[u8]>::decode(&mut decoder, &[7]).expect_err("one byte is incomplete");

    assert_eq!(TranscodeDecodeError::incomplete_input(0, 2, 1), error,);
}

#[test]
fn test_codec_value_decoder_wraps_codec_incomplete_failure() {
    let mut decoder = CodecValueDecoder::<IncompleteDecodeCodec>::new(IncompleteDecodeCodec);

    let error = ValueDecoder::<[u8]>::decode(&mut decoder, &[7])
        .expect_err("codec-reported incomplete input should fail");

    assert_eq!(TranscodeDecodeError::incomplete_input(0, 2, 1), error,);
}

#[test]
fn test_codec_value_decoder_rejects_trailing_input() {
    let mut decoder = CodecValueDecoder::<SingleByteCodec>::new(SingleByteCodec);

    let error = ValueDecoder::<[u8]>::decode(&mut decoder, &[7, 8])
        .expect_err("trailing input should fail");

    assert_eq!(TranscodeDecodeError::trailing_input(1, 1), error,);
}

#[test]
fn test_codec_value_decoder_wraps_codec_decode_error() {
    let mut decoder = CodecValueDecoder::<SingleByteCodec>::new(SingleByteCodec);

    let error = ValueDecoder::<[u8]>::decode(&mut decoder, &[0xff]).expect_err("0xff should fail");

    assert_eq!(
        TranscodeDecodeError::domain_main_with_consumed(
            TestDecodeError::Invalid { consumed: 1 },
            0,
            Some(crate::nz(1)),
        ),
        error,
    );
}

#[test]
fn test_codec_value_decoder_wraps_decode_reset_error() {
    let mut decoder =
        CodecValueDecoder::<ResetFailDecodeCodec>::new(ResetFailDecodeCodec { fail_reset: true });

    let error = decoder
        .decode_lifecycle(&[7])
        .expect_err("decode reset failure should be wrapped");

    assert_eq!(
        TranscodeDecodeError::domain_reset(TestDecodeError::ResetFailed),
        error,
    );

    let mut decoder =
        CodecValueDecoder::<ResetFailDecodeCodec>::new(ResetFailDecodeCodec { fail_reset: false });
    let value = decoder
        .decode_lifecycle(&[7])
        .expect("successful reset mode should decode");
    assert_eq!(&7, value.value());
}

#[test]
#[should_panic(expected = "Codec::decode consumed beyond available input")]
fn test_codec_value_decoder_panics_when_codec_consumes_beyond_input() {
    let mut decoder = CodecValueDecoder::<OverconsumingCodec>::new(OverconsumingCodec);

    let _ = ValueDecoder::<[u8]>::decode(&mut decoder, &[7]);
}

#[test]
#[should_panic(expected = "Codec::decode consumed beyond Codec::MAX_DECODE_UNITS_PER_VALUE")]
fn test_codec_value_decoder_panics_when_codec_consumes_beyond_decode_maximum() {
    let mut decoder = CodecValueDecoder::<OverconsumingCodec>::new(OverconsumingCodec);

    let _ = ValueDecoder::<[u8]>::decode(&mut decoder, &[7, 8]);
}

#[test]
#[should_panic(expected = "Codec::decode_reset wrote beyond its reset bound")]
fn test_codec_value_decoder_panics_when_decode_reset_overreports() {
    let mut decoder =
        CodecValueDecoder::<OverreportingDecodeResetCodec>::new(OverreportingDecodeResetCodec);

    let _ = ValueDecoder::<[u8]>::decode(&mut decoder, &[7]);
}

#[test]
#[should_panic(expected = "Codec::decode_finish wrote beyond its finish bound")]
fn test_codec_value_decoder_panics_when_decode_finish_overreports() {
    let mut decoder =
        CodecValueDecoder::<OverreportingDecodeFinishCodec>::new(OverreportingDecodeFinishCodec);

    let _ = ValueDecoder::<[u8]>::decode(&mut decoder, &[7]);
}

#[test]
fn test_codec_value_decoder_wraps_stateless_decode_finish_error() {
    let mut decoder =
        CodecValueDecoder::<FinishFailStatelessCodec>::new(FinishFailStatelessCodec {
            fail_finish: true,
        });

    let error = ValueDecoder::<[u8]>::decode(&mut decoder, &[7])
        .expect_err("stateless finish failure should be wrapped");

    assert_eq!(
        TranscodeDecodeError::domain_finish(TestDecodeError::FinishFailed),
        error,
    );

    let mut decoder =
        CodecValueDecoder::<FinishFailStatelessCodec>::new(FinishFailStatelessCodec {
            fail_finish: false,
        });
    let value = ValueDecoder::<[u8]>::decode(&mut decoder, &[7])
        .expect("successful stateless finish mode should decode");
    assert_eq!(7, value);
}

#[test]
fn test_codec_value_decoder_wraps_stateful_decode_finish_error() {
    let mut decoder = CodecValueDecoder::<FinishFailStatefulCodec>::new(FinishFailStatefulCodec {
        fail_finish: true,
    });

    let error = decoder
        .decode_lifecycle(&[7])
        .expect_err("stateful finish failure should be wrapped");

    assert_eq!(
        TranscodeDecodeError::domain_finish(TestDecodeError::FinishFailed),
        error,
    );

    let mut decoder = CodecValueDecoder::<FinishFailStatefulCodec>::new(FinishFailStatefulCodec {
        fail_finish: false,
    });
    let value = decoder
        .decode_lifecycle(&[7])
        .expect("successful stateful finish mode should decode");
    assert_eq!(&7, value.value());
}

#[test]
fn test_codec_value_decoder_default_and_debug() {
    let decoder = CodecValueDecoder::<SingleByteCodec>::default();
    let debug = format!("{decoder:?}");
    assert!(debug.contains("CodecValueDecoder"));
    assert!(debug.contains("codec"));
}
