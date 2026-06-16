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

#[test]
fn test_codec_decode_error_wraps_codec_error() {
    let error = CodecDecodeError::decode(TestDecodeError::Invalid { consumed: 2 }, 7);

    assert_eq!(
        CodecDecodeError::Decode {
            source: TestDecodeError::Invalid { consumed: 2 },
            input_index: 7,
        },
        error,
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
}

#[test]
fn test_codec_decode_error_reports_invalid_input_index() {
    let error = CodecDecodeError::<TestDecodeError>::invalid_input_index(5, 2);

    assert_eq!(
        CodecDecodeError::InvalidInputIndex { index: 5, len: 2 },
        error
    );
}

#[test]
fn test_codec_decode_error_reports_invalid_output_index() {
    let error = CodecDecodeError::<TestDecodeError>::invalid_output_index(5, 2);

    assert_eq!(
        CodecDecodeError::InvalidOutputIndex { index: 5, len: 2 },
        error
    );
}

#[test]
fn test_codec_decode_error_reports_insufficient_output() {
    let error = CodecDecodeError::<TestDecodeError>::insufficient_output(2, 4, 1);

    assert_eq!(
        CodecDecodeError::InsufficientOutput {
            output_index: 2,
            required: 4,
            available: 1,
        },
        error,
    );
    assert!(
        CodecDecodeError::<&'static str>::insufficient_output(2, 4, 1)
            .to_string()
            .contains("insufficient finish output")
    );
}

#[test]
fn test_codec_decode_error_display_formats_framework_variants() {
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
        CodecDecodeError::<&'static str>::invalid_input_index(1, 0)
            .to_string()
            .contains("invalid input index")
    );
    assert!(
        CodecDecodeError::<&'static str>::invalid_output_index(1, 0)
            .to_string()
            .contains("invalid output index")
    );
    assert!(
        CodecDecodeError::decode("codec failure", 3)
            .to_string()
            .contains("codec decode error")
    );
}

#[test]
fn test_codec_decode_error_ensure_min_input_accepts_sufficient_input() {
    CodecDecodeError::<TestDecodeError>::ensure_min_input(4, 1, 2).expect("sufficient input");
}

#[test]
fn test_codec_decode_error_ensure_input_index_accepts_valid_index() {
    CodecDecodeError::<TestDecodeError>::ensure_input_index(4, 2).expect("valid index");
}

#[test]
fn test_codec_decode_error_ensure_input_index_rejects_out_of_range() {
    let error = CodecDecodeError::<TestDecodeError>::ensure_input_index(2, 5)
        .expect_err("out-of-range index");

    assert_eq!(CodecDecodeError::invalid_input_index(5, 2), error);
}

#[test]
fn test_codec_decode_error_ensure_output_index_accepts_valid_index() {
    CodecDecodeError::<TestDecodeError>::ensure_output_index(4, 4).expect("valid index");
}

#[test]
fn test_codec_decode_error_ensure_output_index_rejects_out_of_range() {
    let error = CodecDecodeError::<TestDecodeError>::ensure_output_index(1, 2)
        .expect_err("out-of-range index");

    assert_eq!(CodecDecodeError::invalid_output_index(2, 1), error);
}

#[test]
fn test_codec_decode_error_ensure_output_capacity_accepts_sufficient_capacity() {
    CodecDecodeError::<TestDecodeError>::ensure_output_capacity(4, 1, 2)
        .expect("sufficient capacity");
}

#[test]
fn test_codec_decode_error_ensure_output_capacity_delegates_to_output_index() {
    let error = CodecDecodeError::<TestDecodeError>::ensure_output_capacity(2, 5, 0)
        .expect_err("out-of-range index");

    assert_eq!(CodecDecodeError::invalid_output_index(5, 2), error);
}

#[test]
fn test_codec_decode_error_ensure_output_capacity_rejects_insufficient_capacity() {
    let error = CodecDecodeError::<TestDecodeError>::ensure_output_capacity(4, 2, 3)
        .expect_err("insufficient capacity");

    assert_eq!(
        CodecDecodeError::InsufficientOutput {
            output_index: 2,
            required: 3,
            available: 2,
        },
        error,
    );
}

#[test]
fn test_codec_decode_error_ensure_min_input_rejects_incomplete_input() {
    let error = CodecDecodeError::<TestDecodeError>::ensure_min_input(3, 1, 4)
        .expect_err("incomplete input");

    assert_eq!(
        CodecDecodeError::Incomplete {
            input_index: 1,
            required_total: 4,
            available: 2,
        },
        error,
    );
    assert_eq!(core::num::NonZeroUsize::new(2), error.needed_additional());
}

#[test]
fn test_codec_decode_error_ensure_no_trailing_input_accepts_exact_consumption() {
    CodecDecodeError::<TestDecodeError>::ensure_no_trailing_input(4, 4).expect("exact consumption");
}

#[test]
fn test_codec_decode_error_ensure_no_trailing_input_rejects_remaining_input() {
    let error = CodecDecodeError::<TestDecodeError>::ensure_no_trailing_input(1, 4)
        .expect_err("trailing input");

    assert_eq!(
        CodecDecodeError::TrailingInput {
            consumed: 1,
            remaining: 3,
        },
        error,
    );
}
