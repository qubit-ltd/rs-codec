// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{
    CodecTranscodeDecoder, TranscodeDecodeError, TranscodeFailure, TranscodeStatus, Transcoder,
};

use crate::common::IdentityCodec;

#[test]
fn test_lifecycle_reset_reopens_finished_stream() {
    let mut decoder = CodecTranscodeDecoder::new(IdentityCodec);

    assert_eq!(
        Err(TranscodeDecodeError::Failure(
            TranscodeFailure::FinishBeforeReset
        )),
        decoder.finish(&mut [], 0),
    );

    let mut output = [0_u8; 1];
    assert_eq!(
        Err(TranscodeDecodeError::Failure(
            TranscodeFailure::TranscodeBeforeReset,
        )),
        decoder.transcode(&[5], 0, &mut output, 0),
    );

    decoder
        .reset(&mut [], 0)
        .expect("reset should initialize the stream");
    decoder
        .finish(&mut [], 0)
        .expect("empty stream should finish after reset");
    decoder
        .reset(&mut [], 0)
        .expect("reset should reopen stream");

    let progress = decoder
        .transcode(&[5], 0, &mut output, 0)
        .expect("transcode should resume after reset");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!([5], output);
}
