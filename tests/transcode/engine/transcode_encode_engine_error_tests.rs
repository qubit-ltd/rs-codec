// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{
    CodecEncodeError,
    TranscodeEncodeEngineError,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("codec encode failure")]
struct CodecFailure;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("hook encode failure")]
struct HookFailure;

#[test]
fn test_transcode_encode_engine_error_wraps_codec_error() {
    let error = TranscodeEncodeEngineError::<CodecFailure, HookFailure>::codec(
        CodecEncodeError::encode_reset(CodecFailure),
    );

    assert_eq!(
        TranscodeEncodeEngineError::Codec(CodecEncodeError::EncodeReset {
            source: CodecFailure,
        }),
        error,
    );
    assert!(error.to_string().contains("codec encode reset error"));
}

#[test]
fn test_transcode_encode_engine_error_wraps_hook_error() {
    let error = TranscodeEncodeEngineError::<CodecFailure, HookFailure>::hook(
        HookFailure,
    );

    assert_eq!(TranscodeEncodeEngineError::Hook(HookFailure), error);
    assert_eq!("hook encode failure", error.to_string());
}
