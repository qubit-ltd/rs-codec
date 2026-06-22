// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the codec-backed value encoder adapter.

use qubit_codec::{Codec, CodecEncodeError, CodecValueEncoder, ValueEncoder};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct PairByteCodec;

impl Codec for PairByteCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(2);

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::CodecDecodeFailure<Self::DecodeError>>
    {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((value, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index + 2 <= output.len());

        // SAFETY: The caller guarantees that two bytes are writable from
        // `index`.
        unsafe {
            *output.as_mut_ptr().add(index) = *value;
            *output.as_mut_ptr().add(index + 1) = value.wrapping_add(1);
        }
        Ok(qubit_io::nz!(2))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RejectOddCodec;

impl Codec for RejectOddCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = &'static str;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    fn can_encode_value(&self, value: &u8) -> bool {
        value.is_multiple_of(2)
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::CodecDecodeFailure<Self::DecodeError>>
    {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((value, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(self.can_encode_value(value));
        debug_assert!(index < output.len());

        // SAFETY: The caller guarantees that `index` is writable.
        unsafe {
            *output.as_mut_ptr().add(index) = *value;
        }
        Ok(qubit_io::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverreportingEncodeCodec;

impl Codec for OverreportingEncodeCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::CodecDecodeFailure<Self::DecodeError>>
    {
        debug_assert!(index < input.len());

        Ok((input[index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index < output.len());

        output[index] = *value;
        Ok(qubit_io::nz!(2))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct NonCloneValue {
    value: u8,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct NonCloneValueCodec;

impl Codec for NonCloneValueCodec {
    type Value = NonCloneValue;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<
        (NonCloneValue, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((NonCloneValue { value }, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &NonCloneValue,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index < output.len());

        // SAFETY: The caller guarantees that `index` is writable.
        unsafe {
            *output.as_mut_ptr().add(index) = value.value;
        }
        Ok(qubit_io::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetFailLifecycleCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("reset failed")]
struct ResetFailError;

impl Codec for ResetFailLifecycleCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = ResetFailError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = core::num::NonZeroUsize::MIN;

    const MAX_ENCODE_RESET_UNITS: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::CodecDecodeFailure<Self::DecodeError>>
    {
        Ok((input[index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[index] = *value;
        Ok(qubit_io::nz!(1))
    }

    unsafe fn encode_reset(
        &mut self,
        _output: &mut [u8],
        _index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Err(ResetFailError)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::CodecDecodeFailure<Self::DecodeError>>
    {
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
}

#[derive(Default)]
struct StatefulLifecycleCodec {
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

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), qubit_codec::CodecDecodeFailure<Self::DecodeError>>
    {
        Ok((input[index], core::num::NonZeroUsize::MIN))
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
}

#[test]
fn test_codec_value_encoder_emits_reset_output_before_value() {
    let mut encoder =
        CodecValueEncoder::<StatefulLifecycleCodec>::new(StatefulLifecycleCodec::default());

    let output =
        ValueEncoder::<u8>::encode(&mut encoder, &41).expect("encoding should be infallible");

    assert_eq!(vec![0xfe, 42], output);
}

#[test]
fn test_codec_value_encoder_resets_stream_state_on_each_call() {
    let mut encoder =
        CodecValueEncoder::<StatefulLifecycleCodec>::new(StatefulLifecycleCodec::default());

    let first =
        ValueEncoder::<u8>::encode(&mut encoder, &41).expect("first encoding should be infallible");
    let second = ValueEncoder::<u8>::encode(&mut encoder, &41)
        .expect("second encoding should be infallible");

    assert_eq!(vec![0xfe, 42], first);
    assert_eq!(vec![0xfe, 42], second);
}

#[test]
fn test_codec_value_encoder_encodes_one_value_to_owned_units() {
    let mut encoder = CodecValueEncoder::<PairByteCodec>::new(PairByteCodec);

    let output =
        ValueEncoder::<u8>::encode(&mut encoder, &7).expect("encoding should be infallible");

    assert_eq!(vec![7, 8], output);
}

#[test]
fn test_codec_value_encoder_accepts_non_clone_values() {
    let mut encoder = CodecValueEncoder::<NonCloneValueCodec>::new(NonCloneValueCodec);

    let output = ValueEncoder::<NonCloneValue>::encode(&mut encoder, &NonCloneValue { value: 11 })
        .expect("encoding should not require cloning the value");

    assert_eq!(vec![11], output);
}

#[test]
fn test_codec_value_encoder_propagates_encode_error() {
    let mut encoder = CodecValueEncoder::<RejectOddCodec>::new(RejectOddCodec);

    let error =
        ValueEncoder::<u8>::encode(&mut encoder, &7).expect_err("odd value should be rejected");

    assert_eq!(CodecEncodeError::UnencodableValue { input_index: 0 }, error);
}

#[test]
fn test_codec_value_encoder_rejects_output_length_overflow() {
    let mut encoder = CodecValueEncoder::<OverflowEncodeBoundCodec>::new(OverflowEncodeBoundCodec);

    let error = ValueEncoder::<u8>::encode(&mut encoder, &7)
        .expect_err("reset plus value bound should overflow");

    assert_eq!(CodecEncodeError::OutputLengthOverflow, error);
}

#[test]
#[should_panic(expected = "Codec::encode wrote a different length than Codec::encode_len")]
fn test_codec_value_encoder_panics_when_codec_reports_wrong_value_width() {
    let mut encoder = CodecValueEncoder::<OverreportingEncodeCodec>::new(OverreportingEncodeCodec);

    let _ = ValueEncoder::<u8>::encode(&mut encoder, &7);
}

#[test]
fn test_codec_value_encoder_propagates_encode_reset_error() {
    let mut encoder = CodecValueEncoder::<ResetFailLifecycleCodec>::new(ResetFailLifecycleCodec);

    let error = ValueEncoder::<u8>::encode(&mut encoder, &7)
        .expect_err("encode reset failure should propagate");

    assert_eq!(
        CodecEncodeError::Encode {
            source: ResetFailError,
            input_index: 0,
        },
        error,
    );
}
