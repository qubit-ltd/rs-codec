// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests encode outcomes through the public encoder-engine boundary.

use qubit_codec::{
    CodecTranscodeEncoder,
    TranscodeStatus,
    Transcoder,
};

use crate::common::IdentityCodec;

#[test]
fn test_encode_outcome_reports_output_pressure_through_progress() {
    let mut encoder = CodecTranscodeEncoder::new(IdentityCodec);
    let mut reset_output = [];
    encoder
        .reset(&mut reset_output, 0)
        .expect("initialize stream");

    let mut output = [];

    let progress = encoder
        .transcode(&[7], 0, &mut output, 0)
        .expect("output pressure should not be a domain error");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            required: crate::nz(1),
        },
        progress.status(),
    );
    assert_eq!((0, 0), (progress.read(), progress.written()));
}
