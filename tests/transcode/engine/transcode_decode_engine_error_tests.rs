// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{
    CodecDecodeError,
    TranscodeDecodeEngineError,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("codec decode failure")]
struct CodecFailure;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("hook decode failure")]
struct HookFailure;

#[test]
fn test_transcode_decode_engine_error_wraps_codec_error() {
    let error = TranscodeDecodeEngineError::<CodecFailure, HookFailure>::codec(
        CodecDecodeError::decode_flush(CodecFailure),
    );

    assert_eq!(
        TranscodeDecodeEngineError::Codec(CodecDecodeError::DecodeFlush {
            source: CodecFailure,
        }),
        error,
    );
    assert!(error.to_string().contains("codec decode flush error"));
}

#[test]
fn test_transcode_decode_engine_error_wraps_hook_error() {
    let error = TranscodeDecodeEngineError::<CodecFailure, HookFailure>::hook(
        HookFailure,
    );

    assert_eq!(TranscodeDecodeEngineError::Hook(HookFailure), error);
    assert_eq!("hook decode failure", error.to_string());
}
