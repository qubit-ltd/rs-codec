// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::CodecTranscodeConverter;
use qubit_codec::TranscodeStatus;

use crate::common::IdentityCodec;

#[test]
fn test_convert_state_reports_completed_cursor_progress() {
    let mut converter =
        CodecTranscodeConverter::new(IdentityCodec, IdentityCodec);
    let mut reset_output = [];
    converter
        .reset(&mut reset_output, 0)
        .expect("initialize stream");

    let mut output = [0_u8; 2];

    let progress = converter
        .transcode(&[3, 4], 0, &mut output, 0)
        .expect("identity conversion should succeed");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((2, 2), (progress.read(), progress.written()));
    assert_eq!([3, 4], output);
}
