// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{
    CodecTranscodeDecoder,
    TranscodeStatus,
    Transcoder,
};

use crate::common::IdentityCodec;

#[test]
fn test_decode_state_advances_input_and_output_together() {
    let mut decoder = CodecTranscodeDecoder::new(IdentityCodec);
    let mut reset_output = [];
    decoder
        .reset(&mut reset_output, 0)
        .expect("initialize stream");

    let mut output = [0_u8; 2];

    let progress = decoder
        .transcode(&[7, 8], 0, &mut output, 0)
        .expect("identity decoding should succeed");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((2, 2), (progress.read(), progress.written()));
    assert_eq!([7, 8], output);
}
