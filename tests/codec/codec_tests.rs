// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the low-level codec trait.

use qubit_codec as codec;
use qubit_codec::Codec;

#[derive(Default)]
struct ByteIncrementCodec;

impl Codec for ByteIncrementCodec {
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
    ) -> Result<(u8, core::num::NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index < input.len());

        // SAFETY: The caller guarantees that `input_index` is readable.
        let value = unsafe { *input.as_ptr().add(input_index) };
        Ok((value.wrapping_sub(1), core::num::NonZeroUsize::MIN))
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
            *output.as_mut_ptr().add(output_index) = value.wrapping_add(1);
        }
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

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_RESET_UNITS: usize = 1;

    const MAX_DECODE_RESET_VALUES: usize = 2;

    const MAX_DECODE_FINISH_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
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
        output[output_index] = value.wrapping_add(self.encode_state as u8);
        self.encode_state += 1;
        Ok(1)
    }

    unsafe fn encode_reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::EncodeError> {
        output[output_index] = 0xfe;
        self.encode_state = 1;
        Ok(1)
    }

    unsafe fn decode_finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::DecodeError> {
        output[output_index] = self.decode_state as u8;
        self.decode_state = 0;
        Ok(1)
    }
}

#[test]
fn test_codec_trait_encodes_and_decodes_one_value() {
    let mut codec = ByteIncrementCodec;
    let mut output = [0_u8; 1];

    let written = unsafe { codec.encode(&41, &mut output, 0) }.expect("encoding should be infallible");
    let (decoded, consumed) = unsafe { Codec::decode(&mut codec, &output, 0) }.expect("decoding should be infallible");

    assert_eq!(1, <ByteIncrementCodec as Codec>::MIN_UNITS_PER_VALUE,);
    assert_eq!(1, <ByteIncrementCodec as Codec>::MAX_ENCODE_UNITS_PER_VALUE,);
    assert_eq!(1, <ByteIncrementCodec as Codec>::MAX_DECODE_UNITS_PER_VALUE,);
    assert!(codec.can_encode_value(&41));
    assert_eq!(1, written);
    assert_eq!(1, consumed.get());
    assert_eq!(41, decoded);
}

#[test]
fn test_codec_trait_default_lifecycle_methods_are_noop() {
    let mut codec = ByteIncrementCodec;
    let mut reset_output = [0_u8; 1];
    let mut flush_output = [0_u8; 1];

    let reset_written =
        unsafe { codec.encode_reset(&mut reset_output, 0) }.expect("default reset should be infallible");
    let flushed = unsafe { codec.decode_finish(&mut flush_output, 0) }.expect("default flush should be infallible");

    assert_eq!(1, codec.encode_len(&41));
    assert_eq!(0, <ByteIncrementCodec as Codec>::MAX_ENCODE_RESET_UNITS);
    assert_eq!(0, <ByteIncrementCodec as Codec>::MAX_DECODE_FINISH_VALUES);
    assert_eq!(0, reset_written);
    assert_eq!(0, flushed);
    assert_eq!([0], reset_output);
    assert_eq!([0], flush_output);
}

#[test]
fn test_codec_trait_exposes_stateful_lifecycle_methods() {
    let mut codec = StatefulLifecycleCodec::default();
    let mut encoded = [0_u8; 2];
    let mut flushed = [0_u8; 1];

    let reset_written = unsafe { codec.encode_reset(&mut encoded, 0) }.expect("reset should be infallible");
    let value_written =
        unsafe { codec.encode(&41, &mut encoded, reset_written) }.expect("encoding should be infallible");

    assert_eq!(1, reset_written);
    assert_eq!(1, value_written);
    assert_eq!([0xfe, 42], encoded);
    assert_eq!(2, codec.encode_state);

    let (decoded, consumed) = unsafe { Codec::decode(&mut codec, &[42], 0) }.expect("decoding should be infallible");
    let flushed_len = unsafe { codec.decode_finish(&mut flushed, 0) }.expect("flush should be infallible");

    assert_eq!(42, decoded);
    assert_eq!(1, consumed.get());
    assert_eq!(1, flushed_len);
    assert_eq!([1], flushed);
    assert_eq!(0, codec.decode_state);
}

#[derive(Default)]
struct BufferedEncodeCodec {
    buffered: usize,
}

impl Codec for BufferedEncodeCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    fn encode_len(&self, value: &u8) -> usize {
        usize::from(*value == 0)
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        Ok((input[input_index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        if *value != 0 {
            self.buffered += 1;
            return Ok(0);
        }
        output[output_index] = self.buffered as u8;
        self.buffered = 0;
        Ok(1)
    }
}

#[test]
fn test_codec_trait_encode_allows_buffered_zero_output() {
    let mut codec = BufferedEncodeCodec::default();
    let mut output = [0_u8; 1];

    let first = unsafe { codec.encode(&7, &mut output, 0) }.expect("buffering should not fail");
    let second = unsafe { codec.encode(&0, &mut output, 0) }.expect("flushing buffered value should not fail");

    assert_eq!(0, first);
    assert_eq!(1, second);
    assert_eq!([1], output);
}
