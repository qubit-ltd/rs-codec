// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{CodecTranscodeEncoder, TranscodeStatus, Transcoder};

use crate::common::IdentityCodec;

#[test]
fn test_encode_state_stops_before_input_when_output_is_full() {
    let mut encoder = CodecTranscodeEncoder::new(IdentityCodec);
    let mut output = [];

    let progress = encoder
        .transcode(&[9], 0, &mut output, 0)
        .expect("short output should be reported as progress");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            required: crate::nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!((0, 0), (progress.read(), progress.written()));
}
