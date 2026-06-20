// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for codec decode error signals.

use core::num::NonZeroUsize;

use qubit_codec::CodecDecodeErrorSignal;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OpaqueDecodeError;

impl CodecDecodeErrorSignal for OpaqueDecodeError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SignaledDecodeError {
    Incomplete { required: usize },
    Invalid { consumed: NonZeroUsize },
}

impl CodecDecodeErrorSignal for SignaledDecodeError {
    fn required_total(&self) -> Option<usize> {
        match *self {
            Self::Incomplete { required } => Some(required),
            Self::Invalid { .. } => None,
        }
    }

    fn consumed_units(&self) -> Option<NonZeroUsize> {
        match *self {
            Self::Incomplete { .. } => None,
            Self::Invalid { consumed } => Some(consumed),
        }
    }
}

#[test]
fn test_codec_decode_error_signal_defaults_to_no_stream_context() {
    let error = OpaqueDecodeError;

    assert_eq!(None, error.required_total());
    assert_eq!(None, error.consumed_units());
}

#[test]
fn test_codec_decode_error_signal_reports_incomplete_requirement() {
    let error = SignaledDecodeError::Incomplete { required: 4 };

    assert_eq!(Some(4), error.required_total());
    assert_eq!(None, error.consumed_units());
}

#[test]
fn test_codec_decode_error_signal_reports_invalid_consumption() {
    let consumed = NonZeroUsize::new(2).expect("literal is non-zero");
    let error = SignaledDecodeError::Invalid { consumed };

    assert_eq!(None, error.required_total());
    assert_eq!(Some(consumed), error.consumed_units());
}
