// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the codec-backed value decoder adapter.

use qubit_codec::{
    Codec,
    CodecDecodeError,
    CodecValueDecoder,
    ValueDecoder,
};
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct SingleByteCodec;

impl Codec for SingleByteCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
        debug_assert!(input_index < input.len());

        // SAFETY: The caller guarantees that `input_index` is readable.
        let value = unsafe { *input.as_ptr().add(input_index) };
        if value == 0xff {
            Err(qubit_codec::CodecDecodeFailure::invalid(
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
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(output_index < output.len());

        // SAFETY: The caller guarantees that `output_index` is writable.
        unsafe {
            *output.as_mut_ptr().add(output_index) = *value;
        }
        Ok(qubit_io::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FixedPairCodec;

impl Codec for FixedPairCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(2);

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(2);

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
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
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(output_index + 1 < output.len());

        output[output_index] = *value;
        output[output_index + 1] = value.wrapping_add(1);
        Ok(qubit_io::nz!(2))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverconsumingCodec;

impl Codec for OverconsumingCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
        debug_assert!(input_index < input.len());

        Ok((input[input_index], qubit_io::nz!(2)))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(output_index < output.len());

        output[output_index] = *value;
        Ok(qubit_io::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TestDecodeError {
    Invalid { consumed: usize },
    FlushFailed,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FlushFailStatelessCodec;

impl Codec for FlushFailStatelessCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
        Ok((input[input_index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(qubit_io::nz!(1))
    }

    unsafe fn decode_flush(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Err(TestDecodeError::FlushFailed)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FlushFailStatefulCodec;

impl Codec for FlushFailStatefulCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_DECODE_FLUSH_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
        Ok((input[input_index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(qubit_io::nz!(1))
    }

    unsafe fn decode_flush(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Err(TestDecodeError::FlushFailed)
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

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_DECODE_FLUSH_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
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
        output[output_index] = *value;
        Ok(qubit_io::nz!(1))
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

#[derive(Debug, Eq, PartialEq)]
struct CountingFlushValue(u8);

static COUNTING_FLUSH_DEFAULTS: AtomicUsize = AtomicUsize::new(0);

impl Default for CountingFlushValue {
    fn default() -> Self {
        COUNTING_FLUSH_DEFAULTS.fetch_add(1, Ordering::SeqCst);
        Self(0)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CountingFlushCodec;

impl Codec for CountingFlushCodec {
    type Value = CountingFlushValue;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_DECODE_FLUSH_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (CountingFlushValue, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
        Ok((
            CountingFlushValue(input[input_index]),
            core::num::NonZeroUsize::MIN,
        ))
    }

    unsafe fn encode(
        &mut self,
        value: &CountingFlushValue,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[output_index] = value.0;
        Ok(qubit_io::nz!(1))
    }

    unsafe fn decode_flush(
        &mut self,
        output: &mut [CountingFlushValue],
        output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        output[output_index] = CountingFlushValue(0);
        Ok(1)
    }
}

#[test]
fn test_codec_value_decoder_flushes_decode_state_after_success() {
    let mut decoder = CodecValueDecoder::<StatefulLifecycleCodec>::new(
        StatefulLifecycleCodec::default(),
    );

    let first = ValueDecoder::<[u8]>::decode(&mut decoder, &[42])
        .expect("first decode should succeed");
    let second = ValueDecoder::<[u8]>::decode(&mut decoder, &[42])
        .expect("second decode should succeed");

    assert_eq!(42, first);
    assert_eq!(42, second);
}

#[test]
fn test_codec_value_decoder_reuses_flush_scratch() {
    COUNTING_FLUSH_DEFAULTS.store(0, Ordering::SeqCst);
    let mut decoder =
        CodecValueDecoder::<CountingFlushCodec>::new(CountingFlushCodec);

    let first = ValueDecoder::<[u8]>::decode(&mut decoder, &[7])
        .expect("first decode should succeed");
    let second = ValueDecoder::<[u8]>::decode(&mut decoder, &[8])
        .expect("second decode should succeed");

    assert_eq!(CountingFlushValue(7), first);
    assert_eq!(CountingFlushValue(8), second);
    assert_eq!(1, COUNTING_FLUSH_DEFAULTS.load(Ordering::SeqCst));
}

#[test]
fn test_codec_value_decoder_decodes_exactly_one_value() {
    let mut decoder =
        CodecValueDecoder::<SingleByteCodec>::new(SingleByteCodec);

    let output = ValueDecoder::<[u8]>::decode(&mut decoder, &[7])
        .expect("single byte should decode");

    assert_eq!(7, output);
}

#[test]
fn test_codec_value_decoder_default_and_debug_do_not_require_value_debug() {
    let mut decoder = CodecValueDecoder::<SingleByteCodec>::default();

    let output = ValueDecoder::<[u8]>::decode(&mut decoder, &[9])
        .expect("default decoder should decode");
    let debug = format!("{decoder:?}");

    assert_eq!(9, output);
    assert!(debug.contains("CodecValueDecoder"));
    assert!(debug.contains("flush_scratch_len"));
}

#[test]
fn test_codec_value_decoder_reports_too_short_input_before_codec_call() {
    let mut decoder = CodecValueDecoder::<FixedPairCodec>::new(FixedPairCodec);

    let error = ValueDecoder::<[u8]>::decode(&mut decoder, &[7])
        .expect_err("one byte is incomplete");

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
    let mut decoder =
        CodecValueDecoder::<SingleByteCodec>::new(SingleByteCodec);

    let error = ValueDecoder::<[u8]>::decode(&mut decoder, &[7, 8])
        .expect_err("trailing input should fail");

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
    let mut decoder =
        CodecValueDecoder::<SingleByteCodec>::new(SingleByteCodec);

    let error = ValueDecoder::<[u8]>::decode(&mut decoder, &[0xff])
        .expect_err("0xff should fail");

    assert_eq!(
        CodecDecodeError::Decode {
            source: TestDecodeError::Invalid { consumed: 1 },
            input_index: 0,
        },
        error,
    );
}

#[test]
#[should_panic(expected = "Codec::decode consumed beyond available input")]
fn test_codec_value_decoder_panics_when_codec_consumes_beyond_input() {
    let mut decoder =
        CodecValueDecoder::<OverconsumingCodec>::new(OverconsumingCodec);

    let _ = ValueDecoder::<[u8]>::decode(&mut decoder, &[7]);
}

#[test]
fn test_codec_value_decoder_wraps_stateless_decode_flush_error() {
    let mut decoder = CodecValueDecoder::<FlushFailStatelessCodec>::new(
        FlushFailStatelessCodec,
    );

    let error = ValueDecoder::<[u8]>::decode(&mut decoder, &[7])
        .expect_err("stateless flush failure should be wrapped");

    assert_eq!(
        CodecDecodeError::DecodeFlush {
            source: TestDecodeError::FlushFailed,
        },
        error,
    );
}

#[test]
fn test_codec_value_decoder_wraps_stateful_decode_flush_error() {
    let mut decoder = CodecValueDecoder::<FlushFailStatefulCodec>::new(
        FlushFailStatefulCodec,
    );

    let error = ValueDecoder::<[u8]>::decode(&mut decoder, &[7])
        .expect_err("stateful flush failure should be wrapped");

    assert_eq!(
        CodecDecodeError::DecodeFlush {
            source: TestDecodeError::FlushFailed,
        },
        error,
    );
}
