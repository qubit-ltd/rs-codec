// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::{
    io::Cursor,
    io::{
        Error,
        ErrorKind,
    },
};

use core::num::NonZeroUsize;

use qubit_codec::{
    Codec,
    DecodeFailure,
    TranscodeDecodeInput,
};

use crate::common::IdentityCodec;

#[test]
fn test_codec_decode_driver_consumes_one_buffered_value_per_call() {
    let mut input = TranscodeDecodeInput::new(Cursor::new(vec![21, 22]));
    let mut codec = IdentityCodec;

    let first = input
        .read_decoded_with(&mut codec, Error::other)
        .expect("first value should decode");
    let second = input
        .read_decoded_with(&mut codec, Error::other)
        .expect("second value should decode");

    assert_eq!((21, 22), (first, second));
}

#[derive(Clone, Copy, Debug, Default)]
struct EofAwareShortCodec;

impl Codec for EofAwareShortCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = std::convert::Infallible;
    type EncodeError = std::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 3;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index < input.len());
        Err(DecodeFailure::incomplete(qubit_utils::nonzero!(3)))
    }

    unsafe fn decode_eof(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index < input.len());
        Ok((input[input_index], NonZeroUsize::MIN))
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

#[test]
fn test_codec_decode_driver_uses_eof_decode_for_a_short_final_value() {
    let mut input = TranscodeDecodeInput::new(Cursor::new(vec![0xa5]));
    let mut codec = EofAwareShortCodec;

    let value = input
        .read_decoded_with(&mut codec, Error::other)
        .expect("EOF-aware codec should decode its final short value");

    assert_eq!(0xa5, value);
}

#[derive(Clone, Copy, Debug, Default)]
struct InvalidEofIncompleteHintCodec;

impl Codec for InvalidEofIncompleteHintCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = std::convert::Infallible;
    type EncodeError = std::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        Err(DecodeFailure::incomplete(qubit_utils::nonzero!(2)))
    }

    unsafe fn decode_eof(
        &mut self,
        _input: &[u8],
        _input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        Err(DecodeFailure::incomplete(qubit_utils::nonzero!(1)))
    }

    unsafe fn encode(
        &mut self,
        _value: &u8,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Ok(1)
    }
}

#[test]
fn test_codec_decode_driver_rejects_satisfied_eof_incomplete_hint() {
    let mut input = TranscodeDecodeInput::new(Cursor::new(vec![0xa5]));
    let mut codec = InvalidEofIncompleteHintCodec;

    let error = input
        .read_decoded_with(&mut codec, Error::other)
        .expect_err(
            "EOF codec must not report an already satisfied incomplete hint",
        );

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("available window"));
    assert_eq!(&[0xa5], input.unread());
}
