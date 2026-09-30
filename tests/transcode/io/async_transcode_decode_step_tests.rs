// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests the result type of one asynchronous decode step.

use qubit_codec::AsyncTranscodeDecodeStep;
use qubit_codec::TranscodeProgress;

#[test]
fn test_async_transcode_decode_step_distinguishes_progress_payloads() {
    let step = AsyncTranscodeDecodeStep::Progress(TranscodeProgress::complete(2, 1));
    let same_payload = AsyncTranscodeDecodeStep::Progress(TranscodeProgress::complete(2, 1));
    let other_read = AsyncTranscodeDecodeStep::Progress(TranscodeProgress::complete(3, 1));
    let other_written = AsyncTranscodeDecodeStep::Progress(TranscodeProgress::complete(2, 2));

    assert_eq!(step, same_payload, "separately built equal steps compare equal");
    assert_ne!(step, other_read, "the consumed input count takes part in equality");
    assert_ne!(step, other_written, "the written output count takes part in equality");
}

#[test]
fn test_async_transcode_decode_step_separates_progress_from_end_of_input() {
    let step = AsyncTranscodeDecodeStep::Progress(TranscodeProgress::complete(2, 1));

    assert_ne!(
        step,
        AsyncTranscodeDecodeStep::EndOfInput,
        "a committed progress step is not an end-of-input step"
    );
}

#[test]
fn test_async_transcode_decode_step_progress_keeps_committed_counts() {
    let step = AsyncTranscodeDecodeStep::Progress(TranscodeProgress::complete(2, 1));

    let AsyncTranscodeDecodeStep::Progress(progress) = step else {
        panic!("expected a progress step, got {step:?}");
    };

    assert_eq!(2, progress.read(), "committed input count survives");
    assert_eq!(1, progress.written(), "committed output count survives");
    assert!(progress.is_complete(), "complete(2, 1) marks the step complete");
}

#[test]
fn test_async_transcode_decode_step_debug_renders_variant_and_payload() {
    assert_eq!(
        "EndOfInput",
        format!("{:?}", AsyncTranscodeDecodeStep::EndOfInput),
        "EndOfInput renders without a payload"
    );
    assert!(
        format!(
            "{:?}",
            AsyncTranscodeDecodeStep::Progress(TranscodeProgress::complete(2, 1))
        )
        .starts_with("Progress("),
        "Progress renders its wrapped progress"
    );
}
