// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::AsyncTranscodeDecodeStep;
use qubit_codec::TranscodeProgress;

/// Verifies the decode-step result preserves its committed progress.
#[test]
fn test_async_transcode_decode_step_preserves_progress() {
    let step =
        AsyncTranscodeDecodeStep::Progress(TranscodeProgress::complete(2, 1));

    assert_eq!(
        AsyncTranscodeDecodeStep::Progress(TranscodeProgress::complete(2, 1)),
        step,
    );
    assert_ne!(AsyncTranscodeDecodeStep::EndOfInput, step);
}
