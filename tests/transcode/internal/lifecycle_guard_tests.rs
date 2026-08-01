// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{CodecTranscodeDecoder, TranscodeDecodeError, TranscodeFailure, Transcoder};

use crate::common::IdentityCodec;

#[test]
fn test_lifecycle_guard_rejects_double_finish() {
    let mut decoder = CodecTranscodeDecoder::new(IdentityCodec);

    assert_eq!(Ok(0), decoder.finish(&mut [], 0));
    assert_eq!(
        Err(TranscodeDecodeError::Failure(
            TranscodeFailure::FinishAfterFinish,
        )),
        decoder.finish(&mut [], 0),
    );
}
