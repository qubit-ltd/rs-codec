// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::convert::Infallible;
use core::num::NonZeroUsize;
use std::io::Cursor;
use std::io::Error;
use std::io::ErrorKind;

use qubit_codec::Codec;
use qubit_codec::DecodeFailure;
use qubit_codec::TranscodeDecodeInput;
use qubit_io::Input;
use qubit_utils as utils_crate;

use crate::common::IdentityCodec;

struct ShortThenErrorInput {
    calls: usize,
    kind: ErrorKind,
}

impl Input for ShortThenErrorInput {
    type Item = u8;

    unsafe fn read_unchecked(&mut self, output: &mut [u8], index: usize, count: usize) -> std::io::Result<usize> {
        if count == 0 {
            return Ok(0);
        }
        self.calls += 1;
        if self.calls == 1 {
            output[index] = 7;
            Ok(1)
        } else {
            Err(Error::new(self.kind, "next input is unavailable"))
        }
    }
}

struct ShortValueCodec;

impl Codec for ShortValueCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 10;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    unsafe fn decode(&mut self, input: &[u8], index: usize) -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
        Ok((input[index], NonZeroUsize::MIN))
    }

    unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Infallible> {
        output[index] = *value;
        Ok(1)
    }
}

#[test]
fn test_codec_decode_driver_returns_short_value_before_next_io_error() {
    for kind in [ErrorKind::BrokenPipe, ErrorKind::WouldBlock] {
        let source = ShortThenErrorInput { calls: 0, kind };
        let mut input = TranscodeDecodeInput::with_capacity(source, 10);
        let mut codec = ShortValueCodec;
        let value = input
            .read_decoded_with(&mut codec, Error::other)
            .expect("first short value should decode before another input read");
        assert_eq!(7, value);
        assert_eq!(1, input.inner().calls);

        let error = input
            .read_decoded_with(&mut codec, Error::other)
            .expect_err("the next call should report the underlying input error");
        assert_eq!(kind, error.kind());
        assert_eq!(2, input.inner().calls);
    }
}

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
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 3;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index < input.len());
        Err(DecodeFailure::incomplete(utils_crate::nonzero(3)))
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
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        Err(DecodeFailure::incomplete(utils_crate::nonzero(2)))
    }

    unsafe fn decode_eof(
        &mut self,
        _input: &[u8],
        _input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        Err(DecodeFailure::incomplete(utils_crate::nonzero(1)))
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
        .expect_err("EOF codec must not report an already satisfied incomplete hint");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("available window"));
    assert_eq!(&[0xa5], input.unread());
}

#[derive(Clone, Copy, Debug, Default)]
struct AlwaysIncompleteCodec;

impl Codec for AlwaysIncompleteCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 4;

    unsafe fn decode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        Err(DecodeFailure::incomplete(utils_crate::nonzero(4)))
    }

    unsafe fn decode_eof(
        &mut self,
        _input: &[u8],
        _input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        Err(DecodeFailure::incomplete(utils_crate::nonzero(4)))
    }

    unsafe fn encode(
        &mut self,
        _value: &u8,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Ok(0)
    }
}

#[test]
fn test_codec_decode_driver_reports_truncated_input_as_unexpected_eof() {
    let mut input = TranscodeDecodeInput::new(Cursor::new(vec![0xa5]));

    let error = input
        .read_decoded_with(&mut AlwaysIncompleteCodec, Error::other)
        .expect_err("a truncated value must not decode");

    assert_eq!(ErrorKind::UnexpectedEof, error.kind());
    assert!(input.unread().is_empty());
}

#[derive(Clone, Copy, Debug, Default)]
struct RejectingCodec;

impl Codec for RejectingCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Error;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        Err(DecodeFailure::invalid(
            Error::other("rejected input"),
            NonZeroUsize::MIN,
        ))
    }

    unsafe fn encode(
        &mut self,
        _value: &u8,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Ok(0)
    }
}

#[test]
fn test_codec_decode_driver_maps_codec_rejection_to_io_error() {
    let mut input = TranscodeDecodeInput::new(Cursor::new(vec![0xa5]));

    let error = input
        .read_decoded_with(&mut RejectingCodec, Error::other)
        .expect_err("a rejected value must fail");

    assert!(error.to_string().contains("rejected input"));
    assert!(input.unread().is_empty());
}

#[derive(Clone, Copy, Debug, Default)]
struct OverConsumingCodec;

impl Codec for OverConsumingCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        Ok((input[input_index], utils_crate::nonzero(2)))
    }

    unsafe fn encode(
        &mut self,
        _value: &u8,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Ok(0)
    }
}

#[test]
fn test_codec_decode_driver_rejects_consumption_beyond_window() {
    let mut input = TranscodeDecodeInput::new(Cursor::new(vec![0xa5]));

    let error = input
        .read_decoded_with(&mut OverConsumingCodec, Error::other)
        .expect_err("consuming beyond the window must be rejected");

    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert!(error.to_string().contains("exceed unread window"));
    assert_eq!(&[0xa5], input.unread());
}
