// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::CodecEncodeError;

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
    let error = CodecEncodeError::encode_reset(TestEncodeError);

    assert_eq!(
        CodecEncodeError::EncodeReset {
            source: TestEncodeError,
        },
        error,
    );
    assert!(error.to_string().contains("codec encode reset error"));
}

#[test]
fn test_codec_encode_error_wraps_encode_flush_error() {
    let error = CodecEncodeError::encode_flush(TestEncodeError);

    assert_eq!(
        CodecEncodeError::EncodeFlush {
            source: TestEncodeError,
        },
        error,
    );
    assert!(error.to_string().contains("codec encode flush error"));
}

#[test]
fn test_codec_encode_error_into_source_extracts_codec_errors() {
    assert_eq!(
        Some(TestEncodeError),
        CodecEncodeError::encode(TestEncodeError, 7).into_source(),
    );
    assert_eq!(
        Some(TestEncodeError),
        CodecEncodeError::encode_reset(TestEncodeError).into_source(),
    );
    assert_eq!(
        Some(TestEncodeError),
        CodecEncodeError::encode_flush(TestEncodeError).into_source(),
    );
    assert_eq!(
        None,
        CodecEncodeError::<TestEncodeError>::unencodable_value(7).into_source(),
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
fn test_codec_encode_error_display_formats_domain_variants() {
    assert!(
        CodecEncodeError::encode("codec failure", 2)
            .to_string()
            .contains("codec encode error")
    );
    assert!(
        CodecEncodeError::<&'static str>::unencodable_value(7)
            .to_string()
            .contains("unencodable value")
    );
}
