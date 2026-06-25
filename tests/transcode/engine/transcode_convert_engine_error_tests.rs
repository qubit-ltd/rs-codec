// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::TranscodeConvertEngineError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("decode side failure")]
struct DecodeFailure;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("encode side failure")]
struct EncodeFailure;

#[test]
fn test_transcode_convert_engine_error_wraps_decode_error() {
    let error =
        TranscodeConvertEngineError::<DecodeFailure, EncodeFailure>::decode(
            DecodeFailure,
        );

    assert_eq!(TranscodeConvertEngineError::Decode(DecodeFailure), error,);
    assert_eq!("decode side failed: decode side failure", error.to_string());
}

#[test]
fn test_transcode_convert_engine_error_wraps_encode_error() {
    let error =
        TranscodeConvertEngineError::<DecodeFailure, EncodeFailure>::encode(
            EncodeFailure,
        );

    assert_eq!(TranscodeConvertEngineError::Encode(EncodeFailure), error,);
    assert_eq!("encode side failed: encode side failure", error.to_string());
}
