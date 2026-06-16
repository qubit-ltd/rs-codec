// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the codec-backed value decoder adapter.

use qubit_codec::{Codec, CodecDecodeError, CodecValueDecoder, ValueDecoder, nz};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct SingleByteCodec;

unsafe impl Codec for SingleByteCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
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
        if value == 0xff {
            Err(TestDecodeError::Invalid { consumed: 1 })
        } else {
            Ok((value, core::num::NonZeroUsize::MIN))
        }
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
            *output.as_mut_ptr().add(index) = *value;
        }
        Ok(nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FixedPairCodec;

unsafe impl Codec for FixedPairCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::new(2).expect("literal is non-zero")
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        unsafe { core::num::NonZeroUsize::new_unchecked(2) }
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index + 1 < input.len());

        Ok((input[index].wrapping_add(input[index + 1]), unsafe {
            core::num::NonZeroUsize::new_unchecked(2)
        }))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index + 1 < output.len());

        output[index] = *value;
        output[index + 1] = value.wrapping_add(1);
        Ok(nz!(2))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverconsumingCodec;

unsafe impl Codec for OverconsumingCodec {
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

        Ok((
            input[index],
            core::num::NonZeroUsize::new(2).expect("literal is non-zero"),
        ))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index < output.len());

        output[index] = *value;
        Ok(nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TestDecodeError {
    Invalid { consumed: usize },
    FlushFailed,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FlushFailStatelessCodec;

unsafe impl Codec for FlushFailStatelessCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
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
        Ok((input[index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[index] = *value;
        Ok(nz!(1))
    }

    unsafe fn decode_flush(
        &mut self,
        _output: &mut [u8],
        _index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Err(TestDecodeError::FlushFailed)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FlushFailStatefulCodec;

unsafe impl Codec for FlushFailStatefulCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = TestDecodeError;
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
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[index] = *value;
        Ok(nz!(1))
    }

    unsafe fn decode_flush(
        &mut self,
        _output: &mut [u8],
        _index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Err(TestDecodeError::FlushFailed)
    }
}

#[derive(Default)]
struct StatefulLifecycleCodec {
    decode_state: usize,
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
        output[index] = *value;
        Ok(nz!(1))
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

#[test]
fn test_codec_value_decoder_flushes_decode_state_after_success() {
    let mut decoder =
        CodecValueDecoder::<StatefulLifecycleCodec>::new(StatefulLifecycleCodec::default());

    let first =
        ValueDecoder::<[u8]>::decode(&mut decoder, &[42]).expect("first decode should succeed");
    let second =
        ValueDecoder::<[u8]>::decode(&mut decoder, &[42]).expect("second decode should succeed");

    assert_eq!(42, first);
    assert_eq!(42, second);
}

#[test]
fn test_codec_value_decoder_decodes_exactly_one_value() {
    let mut decoder = CodecValueDecoder::<SingleByteCodec>::new(SingleByteCodec);

    let output =
        ValueDecoder::<[u8]>::decode(&mut decoder, &[7]).expect("single byte should decode");

    assert_eq!(7, output);
}

#[test]
fn test_codec_value_decoder_reports_too_short_input_before_codec_call() {
    let mut decoder = CodecValueDecoder::<FixedPairCodec>::new(FixedPairCodec);

    let error =
        ValueDecoder::<[u8]>::decode(&mut decoder, &[7]).expect_err("one byte is incomplete");

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
    let mut decoder = CodecValueDecoder::<SingleByteCodec>::new(SingleByteCodec);

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
    let mut decoder = CodecValueDecoder::<SingleByteCodec>::new(SingleByteCodec);

    let error = ValueDecoder::<[u8]>::decode(&mut decoder, &[0xff]).expect_err("0xff should fail");

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
    let mut decoder = CodecValueDecoder::<OverconsumingCodec>::new(OverconsumingCodec);

    let _ = ValueDecoder::<[u8]>::decode(&mut decoder, &[7]);
}

#[test]
fn test_codec_value_decoder_wraps_stateless_decode_flush_error() {
    let mut decoder = CodecValueDecoder::<FlushFailStatelessCodec>::new(FlushFailStatelessCodec);

    let error = ValueDecoder::<[u8]>::decode(&mut decoder, &[7])
        .expect_err("stateless flush failure should be wrapped");

    assert_eq!(
        CodecDecodeError::Decode {
            source: TestDecodeError::FlushFailed,
            input_index: 1,
        },
        error,
    );
}

#[test]
fn test_codec_value_decoder_wraps_stateful_decode_flush_error() {
    let mut decoder = CodecValueDecoder::<FlushFailStatefulCodec>::new(FlushFailStatefulCodec);

    let error = ValueDecoder::<[u8]>::decode(&mut decoder, &[7])
        .expect_err("stateful flush failure should be wrapped");

    assert_eq!(
        CodecDecodeError::Decode {
            source: TestDecodeError::FlushFailed,
            input_index: 1,
        },
        error,
    );
}
