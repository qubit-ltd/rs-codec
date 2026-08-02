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
fn test_transcode_state_tracks_nonzero_start_indexes() {
    let mut decoder = CodecTranscodeDecoder::new(IdentityCodec);
    let mut reset_output = [];
    decoder
        .reset(&mut reset_output, 0)
        .expect("initialize stream");

    let mut output = [0_u8; 3];

    let progress = decoder
        .transcode(&[1, 2, 3], 1, &mut output, 1)
        .expect("indexed decoding should succeed");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((2, 2), (progress.read(), progress.written()));
    assert_eq!([0, 2, 3], output);
}
