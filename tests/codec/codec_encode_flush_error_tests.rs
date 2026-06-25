// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use qubit_codec::{
    CodecEncodeError,
    CodecEncodeFlushError,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TestEncodeFlushError {
    Failed,
}

impl core::fmt::Display for TestEncodeFlushError {
    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        match self {
            Self::Failed => formatter.write_str("encode flush failed"),
        }
    }
}

#[test]
fn test_codec_encode_flush_error_wraps_codec_flush_error() {
    let lifecycle = CodecEncodeFlushError::new(TestEncodeFlushError::Failed);
    assert_eq!(TestEncodeFlushError::Failed, *lifecycle.source());

    let error: CodecEncodeError<TestEncodeFlushError> = lifecycle.into();

    assert_eq!(
        CodecEncodeError::EncodeFlush {
            source: TestEncodeFlushError::Failed,
        },
        error,
    );
    assert!(error.to_string().contains("codec encode flush error"));

    let lifecycle: CodecEncodeFlushError<TestEncodeFlushError> =
        TestEncodeFlushError::Failed.into();
    assert_eq!(TestEncodeFlushError::Failed, lifecycle.into_source());
}
