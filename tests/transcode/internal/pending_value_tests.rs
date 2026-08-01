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
fn test_pending_value_retains_source_position_when_output_is_short() {
    let mut converter = CodecTranscodeConverter::new(IdentityCodec, IdentityCodec);
    let mut reset_output = [];
    converter
        .reset(&mut reset_output, 0)
        .expect("initialize stream");

    let progress = converter
        .transcode(&[11], 0, &mut [], 0)
        .expect("short output should retain the decoded value");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            required: crate::nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!((1, 0), (progress.read(), progress.written()));
}
