// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the low-level codec trait.

use qubit_codec::{
    Codec,
    CodecValueEncoder,
    ValueEncoder,
};

#[derive(Default)]
struct ByteIncrementCodec;

impl Codec for ByteIncrementCodec {
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
        index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
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
        Ok(qubit_io::nz!(1))
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

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_ENCODE_RESET_UNITS: usize = 1;

    const MAX_DECODE_FLUSH_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
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
struct InvalidBoundsCodec;

impl Codec for InvalidBoundsCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(2);

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    unsafe fn decode(
        &mut self,
        _input: &[u8],
        _index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
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

#[test]
fn test_codec_trait_encodes_and_decodes_one_value() {
    let mut codec = ByteIncrementCodec;
    let mut output = [0_u8; 1];

    let written = unsafe { codec.encode(&41, &mut output, 0) }
        .expect("encoding should be infallible");
    let (decoded, consumed) = unsafe { Codec::decode(&mut codec, &output, 0) }
        .expect("decoding should be infallible");

    assert_eq!(1, <ByteIncrementCodec as Codec>::MIN_UNITS_PER_VALUE.get(),);
    assert_eq!(1, <ByteIncrementCodec as Codec>::MAX_UNITS_PER_VALUE.get(),);
    assert!(codec.can_encode_value(&41));
    assert_eq!(1, written.get());
    assert_eq!(1, consumed.get());
    assert_eq!(41, decoded);
}

#[test]
fn test_codec_trait_default_lifecycle_methods_are_noop() {
    let mut codec = ByteIncrementCodec;
    let mut reset_output = [0_u8; 1];
    let mut flush_output = [0_u8; 1];

    let reset_written = unsafe { codec.encode_reset(&mut reset_output, 0) }
        .expect("default reset should be infallible");
    let flushed = unsafe { codec.decode_flush(&mut flush_output, 0) }
        .expect("default flush should be infallible");

    assert_eq!(1, codec.encode_len(&41).get());
    assert_eq!(0, <ByteIncrementCodec as Codec>::MAX_ENCODE_RESET_UNITS);
    assert_eq!(0, <ByteIncrementCodec as Codec>::MAX_DECODE_FLUSH_VALUES);
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

    let reset_written = unsafe { codec.encode_reset(&mut encoded, 0) }
        .expect("reset should be infallible");
    let value_written =
        unsafe { codec.encode(&41, &mut encoded, reset_written) }
            .expect("encoding should be infallible");

    assert_eq!(1, reset_written);
    assert_eq!(1, value_written.get());
    assert_eq!([0xfe, 42], encoded);
    assert_eq!(2, codec.encode_state);

    let (decoded, consumed) = unsafe { Codec::decode(&mut codec, &[42], 0) }
        .expect("decoding should be infallible");
    let flushed_len = unsafe { codec.decode_flush(&mut flushed, 0) }
        .expect("flush should be infallible");

    assert_eq!(42, decoded);
    assert_eq!(1, consumed.get());
    assert_eq!(1, flushed_len);
    assert_eq!([1], flushed);
    assert_eq!(0, codec.decode_state);
}

#[test]
#[should_panic(
    expected = "Codec::MIN_UNITS_PER_VALUE must not exceed Codec::MAX_UNITS_PER_VALUE"
)]
fn test_codec_unit_bounds_panics_when_min_exceeds_max() {
    let mut encoder = CodecValueEncoder::new(InvalidBoundsCodec);

    let _ = encoder
        .encode(&42)
        .expect("unit-bound assertion should panic before encoding");
}
