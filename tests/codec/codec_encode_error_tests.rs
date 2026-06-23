// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{
    CapacityError,
    CodecEncodeError,
    CodecEncodeResetError,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TestEncodeError;

impl core::fmt::Display for TestEncodeError {
    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        formatter.write_str("test encode error")
    }
}

#[test]
fn test_codec_encode_error_wraps_codec_error() {
    let error = CodecEncodeError::encode(TestEncodeError, 7);

    assert_eq!(
        CodecEncodeError::Encode {
            source: TestEncodeError,
            input_index: 7,
        },
        error,
    );
}

#[test]
fn test_codec_encode_error_wraps_encode_reset_error() {
    let lifecycle = CodecEncodeResetError::new(TestEncodeError);
    assert_eq!(TestEncodeError, *lifecycle.source());

    let error: CodecEncodeError<TestEncodeError> = lifecycle.into();

    assert_eq!(
        CodecEncodeError::EncodeReset {
            source: TestEncodeError,
        },
        error,
    );
    assert!(error.to_string().contains("codec encode reset error"));

    let lifecycle: CodecEncodeResetError<TestEncodeError> =
        TestEncodeError.into();
    assert_eq!(TestEncodeError, lifecycle.into_source());
}

#[test]
fn test_codec_encode_error_reports_invalid_input_index() {
    let error = CodecEncodeError::<TestEncodeError>::invalid_input_index(5, 2);

    assert_eq!(
        CodecEncodeError::InvalidInputIndex { index: 5, len: 2 },
        error
    );
}

#[test]
fn test_codec_encode_error_reports_unencodable_value() {
    let error = CodecEncodeError::<TestEncodeError>::unencodable_value(7);

    assert_eq!(CodecEncodeError::UnencodableValue { input_index: 7 }, error);
    assert!(
        CodecEncodeError::<&'static str>::unencodable_value(7)
            .to_string()
            .contains("unencodable value")
    );
}

#[test]
fn test_codec_encode_error_reports_invalid_output_index() {
    let error = CodecEncodeError::<TestEncodeError>::invalid_output_index(5, 2);

    assert_eq!(
        CodecEncodeError::InvalidOutputIndex { index: 5, len: 2 },
        error
    );
}

#[test]
fn test_codec_encode_error_reports_insufficient_output() {
    let error =
        CodecEncodeError::<TestEncodeError>::insufficient_output(2, 4, 1);

    assert_eq!(
        CodecEncodeError::InsufficientOutput {
            output_index: 2,
            required: 4,
            available: 1,
        },
        error,
    );
    assert!(
        CodecEncodeError::<&'static str>::insufficient_output(2, 4, 1)
            .to_string()
            .contains("insufficient output")
    );
}

#[test]
fn test_codec_encode_error_reports_output_length_overflow() {
    let error = CodecEncodeError::<TestEncodeError>::output_length_overflow();

    assert_eq!(CodecEncodeError::OutputLengthOverflow, error);
    assert!(
        CodecEncodeError::<&'static str>::output_length_overflow()
            .to_string()
            .contains("output length arithmetic overflow")
    );
}

#[test]
fn test_codec_encode_error_converts_capacity_error() {
    let error: CodecEncodeError<TestEncodeError> =
        CapacityError::OutputLengthOverflow.into();

    assert_eq!(CodecEncodeError::OutputLengthOverflow, error);
}

#[test]
fn test_codec_encode_error_display_formats_framework_variants() {
    assert!(
        CodecEncodeError::<&'static str>::invalid_input_index(1, 0)
            .to_string()
            .contains("invalid input index")
    );
    assert!(
        CodecEncodeError::<&'static str>::invalid_output_index(1, 0)
            .to_string()
            .contains("invalid output index")
    );
    assert!(
        CodecEncodeError::encode("codec failure", 2)
            .to_string()
            .contains("codec encode error")
    );
}

#[test]
fn test_codec_encode_error_ensure_input_index_accepts_valid_index() {
    CodecEncodeError::<TestEncodeError>::ensure_input_index(4, 2)
        .expect("valid index");
}

#[test]
fn test_codec_encode_error_ensure_input_index_rejects_out_of_range() {
    let error = CodecEncodeError::<TestEncodeError>::ensure_input_index(2, 5)
        .expect_err("out-of-range index");

    assert_eq!(CodecEncodeError::invalid_input_index(5, 2), error);
}

#[test]
fn test_codec_encode_error_ensure_output_index_accepts_valid_index() {
    CodecEncodeError::<TestEncodeError>::ensure_output_index(4, 4)
        .expect("valid index");
}

#[test]
fn test_codec_encode_error_ensure_output_index_rejects_out_of_range() {
    let error = CodecEncodeError::<TestEncodeError>::ensure_output_index(1, 2)
        .expect_err("out-of-range index");

    assert_eq!(CodecEncodeError::invalid_output_index(2, 1), error);
}

#[test]
fn test_codec_encode_error_ensure_output_capacity_accepts_sufficient_capacity()
{
    CodecEncodeError::<TestEncodeError>::ensure_output_capacity(4, 1, 2)
        .expect("sufficient capacity");
}

#[test]
fn test_codec_encode_error_ensure_output_capacity_delegates_to_output_index() {
    let error =
        CodecEncodeError::<TestEncodeError>::ensure_output_capacity(2, 5, 0)
            .expect_err("out-of-range index");

    assert_eq!(CodecEncodeError::invalid_output_index(5, 2), error);
}

#[test]
fn test_codec_encode_error_ensure_output_capacity_rejects_insufficient_capacity()
 {
    let error =
        CodecEncodeError::<TestEncodeError>::ensure_output_capacity(4, 2, 3)
            .expect_err("insufficient capacity");

    assert_eq!(
        CodecEncodeError::InsufficientOutput {
            output_index: 2,
            required: 3,
            available: 2,
        },
        error,
    );
}
