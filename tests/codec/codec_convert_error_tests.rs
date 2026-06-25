// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{
    CodecConvertError,
    CodecDecodeError,
    CodecEncodeError,
};

#[test]
fn test_codec_convert_error_wraps_decode_error_explicitly() {
    let decode = CodecDecodeError::decode("decode failed", 4);
    let error = CodecConvertError::<&'static str, &'static str>::decode(decode);

    assert_eq!(
        CodecConvertError::Decode(CodecDecodeError::Decode {
            source: "decode failed",
            input_index: 4,
        }),
        error,
    );
}

#[test]
fn test_codec_convert_error_wraps_encode_error_explicitly() {
    let encode = CodecEncodeError::encode("encode failed", 7);
    let error = CodecConvertError::<&'static str, &'static str>::encode(encode);

    assert_eq!(
        CodecConvertError::Encode(CodecEncodeError::Encode {
            source: "encode failed",
            input_index: 7,
        }),
        error,
    );
}
