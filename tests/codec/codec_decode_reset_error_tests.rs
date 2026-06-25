// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use qubit_codec::{
    CodecDecodeError,
    CodecDecodeResetError,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TestDecodeResetError {
    Failed,
}

impl core::fmt::Display for TestDecodeResetError {
    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        match self {
            Self::Failed => formatter.write_str("decode reset failed"),
        }
    }
}

#[test]
fn test_codec_decode_reset_error_wraps_codec_reset_error() {
    let lifecycle = CodecDecodeResetError::new(TestDecodeResetError::Failed);
    assert_eq!(TestDecodeResetError::Failed, *lifecycle.source());

    let error: CodecDecodeError<TestDecodeResetError> = lifecycle.into();

    assert_eq!(
        CodecDecodeError::DecodeReset {
            source: TestDecodeResetError::Failed,
        },
        error,
    );
    assert!(error.to_string().contains("codec decode reset error"));

    let lifecycle: CodecDecodeResetError<TestDecodeResetError> =
        TestDecodeResetError::Failed.into();
    assert_eq!(TestDecodeResetError::Failed, lifecycle.into_source());
}
