// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests decode outcomes through the public decoder-engine boundary.

use qubit_codec::{CodecTranscodeDecoder, TranscodeStatus, Transcoder};

use crate::common::IdentityCodec;

#[test]
fn test_decode_outcome_emits_through_transcode_progress() {
    let mut decoder = CodecTranscodeDecoder::new(IdentityCodec);
    let mut reset_output = [];
    decoder
        .reset(&mut reset_output, 0)
        .expect("initialize stream");

    let mut output = [0_u8; 1];

    let progress = decoder
        .transcode(&[7], 0, &mut output, 0)
        .expect("identity decode should succeed");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((1, 1), (progress.read(), progress.written()));
    assert_eq!([7], output);
}
