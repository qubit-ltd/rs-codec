// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{CodecTranscodeConverter, TranscodeStatus};

use crate::common::IdentityCodec;

#[test]
fn test_pending_value_slot_drains_before_reading_more_input() {
    let mut converter = CodecTranscodeConverter::new(IdentityCodec, IdentityCodec);
    let first = converter
        .transcode(&[13], 0, &mut [], 0)
        .expect("short output should retain the decoded value");
    assert_eq!(1, first.read());

    let mut output = [0_u8; 1];
    let resumed = converter
        .transcode(&[], 0, &mut output, 0)
        .expect("retained value should drain on the next call");

    assert_eq!(TranscodeStatus::Complete, resumed.status());
    assert_eq!((0, 1), (resumed.read(), resumed.written()));
    assert_eq!([13], output);
}
