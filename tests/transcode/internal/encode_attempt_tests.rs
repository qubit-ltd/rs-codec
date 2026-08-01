// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{CodecTranscodeEncoder, Transcoder};

use crate::common::IdentityCodec;

#[test]
fn test_encode_attempt_is_exercised_by_encoder() {
    let mut encoder = CodecTranscodeEncoder::new(IdentityCodec);
    let mut reset_output = [];
    encoder
        .reset(&mut reset_output, 0)
        .expect("initialize stream");

    let mut output = [0_u8; 1];
    let progress = encoder
        .transcode(&[7], 0, &mut output, 0)
        .expect("encode attempt should complete");

    assert!(progress.is_complete());
    assert_eq!(1, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([7], output);
}
