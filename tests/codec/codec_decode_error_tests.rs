// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::CodecDecodeError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TestDecodeError {
    Invalid { consumed: usize },
}

impl core::fmt::Display for TestDecodeError {
    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        match self {
            Self::Invalid { consumed } => {
                write!(formatter, "invalid decode consumed {consumed}")
            }
        }
    }
}

#[test]
fn test_codec_decode_error_wraps_codec_error() {
    let error =
        CodecDecodeError::decode(TestDecodeError::Invalid { consumed: 2 }, 7);

    assert_eq!(
        CodecDecodeError::Decode {
            source: TestDecodeError::Invalid { consumed: 2 },
            input_index: 7,
        },
        error,
    );
}

#[test]
fn test_codec_decode_error_wraps_decode_flush_error() {
    let error = CodecDecodeError::decode_flush(TestDecodeError::Invalid {
        consumed: 1,
    });

    assert_eq!(
        CodecDecodeError::DecodeFlush {
            source: TestDecodeError::Invalid { consumed: 1 },
        },
        error,
    );
    assert!(error.to_string().contains("codec decode flush error"));
}

#[test]
fn test_codec_decode_error_wraps_decode_reset_error() {
    let error = CodecDecodeError::decode_reset(TestDecodeError::Invalid {
        consumed: 4,
    });

    assert_eq!(
        CodecDecodeError::DecodeReset {
            source: TestDecodeError::Invalid { consumed: 4 },
        },
        error,
    );
}

#[test]
fn test_codec_decode_error_into_source_extracts_codec_errors() {
    assert_eq!(
        Some(TestDecodeError::Invalid { consumed: 2 }),
        CodecDecodeError::decode(TestDecodeError::Invalid { consumed: 2 }, 7)
            .into_source(),
    );
    assert_eq!(
        Some(TestDecodeError::Invalid { consumed: 4 }),
        CodecDecodeError::decode_reset(TestDecodeError::Invalid {
            consumed: 4
        })
        .into_source(),
    );
    assert_eq!(
        Some(TestDecodeError::Invalid { consumed: 1 }),
        CodecDecodeError::decode_flush(TestDecodeError::Invalid {
            consumed: 1
        })
        .into_source(),
    );
    assert_eq!(
        None,
        CodecDecodeError::<TestDecodeError>::incomplete(0, 2, 1).into_source(),
    );
    assert_eq!(
        None,
        CodecDecodeError::<TestDecodeError>::trailing_input(1, 1).into_source(),
    );
}

#[test]
fn test_codec_decode_error_reports_adapter_incomplete_input() {
    let error = CodecDecodeError::<TestDecodeError>::incomplete(3, 4, 2);

    assert_eq!(
        CodecDecodeError::Incomplete {
            input_index: 3,
            required_total: 4,
            available: 2,
        },
        error,
    );
    assert!(error.is_incomplete());
    assert_eq!(Some(crate::nz(2)), error.needed_additional());
}

#[test]
fn test_codec_decode_error_reports_trailing_input() {
    let error = CodecDecodeError::<TestDecodeError>::trailing_input(1, 3);

    assert_eq!(
        CodecDecodeError::TrailingInput {
            consumed: 1,
            remaining: 3,
        },
        error,
    );
    assert!(!error.is_incomplete());
    assert_eq!(None, error.needed_additional());
}

#[test]
fn test_codec_decode_error_display_formats_domain_variants() {
    assert!(
        CodecDecodeError::<&'static str>::incomplete(0, 2, 1)
            .to_string()
            .contains("incomplete input")
    );
    assert!(
        CodecDecodeError::<&'static str>::trailing_input(1, 1)
            .to_string()
            .contains("trailing input")
    );
    assert!(
        CodecDecodeError::decode("codec failure", 3)
            .to_string()
            .contains("codec decode error")
    );
}

#[test]
fn test_codec_decode_error_ensure_min_input_accepts_available_input() {
    assert_eq!(
        Ok(()),
        CodecDecodeError::<TestDecodeError>::ensure_min_input(5, 2, 3),
    );
}

#[test]
fn test_codec_decode_error_ensure_min_input_rejects_short_input() {
    assert_eq!(
        Err(CodecDecodeError::Incomplete {
            input_index: 2,
            required_total: 4,
            available: 3,
        }),
        CodecDecodeError::<TestDecodeError>::ensure_min_input(5, 2, 4),
    );
}

#[test]
fn test_codec_decode_error_ensure_no_trailing_input_accepts_exact_input() {
    assert_eq!(
        Ok(()),
        CodecDecodeError::<TestDecodeError>::ensure_no_trailing_input(3, 3),
    );
}

#[test]
fn test_codec_decode_error_ensure_no_trailing_input_rejects_extra_input() {
    assert_eq!(
        Err(CodecDecodeError::TrailingInput {
            consumed: 2,
            remaining: 3,
        }),
        CodecDecodeError::<TestDecodeError>::ensure_no_trailing_input(2, 5),
    );
}
