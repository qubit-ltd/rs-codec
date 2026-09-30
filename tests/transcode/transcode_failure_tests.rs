// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests framework-level failures reported by safe transcode APIs.

use qubit_codec::TranscodeFailure;

#[test]
fn test_transcode_failure_output_range_validation() {
    assert_eq!(Ok(()), TranscodeFailure::ensure_output_range(4, 1, 2, 2));
    assert_eq!(
        Err(TranscodeFailure::InvalidOutputIndex {
            index: 5,
            output_len: 4,
        }),
        TranscodeFailure::ensure_output_range(4, 5, 0, 0),
    );
    assert_eq!(
        Err(TranscodeFailure::InvalidOutputRange {
            output_index: 3,
            range_len: 2,
            output_len: 4,
        }),
        TranscodeFailure::ensure_output_range(4, 3, 2, 1),
    );
    assert_eq!(
        Err(TranscodeFailure::InsufficientOutput {
            output_index: 1,
            required: 3,
            available: 2,
        }),
        TranscodeFailure::ensure_output_range(4, 1, 2, 3),
    );
}

#[test]
fn test_transcode_failure_output_range_accepts_exhausting_range() {
    assert_eq!(
        Ok(()),
        TranscodeFailure::ensure_output_range(4, 2, 2, 2),
        "a range may reach the end of the output slice"
    );
    assert_eq!(
        Err(TranscodeFailure::InsufficientOutput {
            output_index: 2,
            required: 3,
            available: 2,
        }),
        TranscodeFailure::ensure_output_range(4, 2, 2, 3),
        "the writable range bounds the required capacity"
    );
}

#[test]
fn test_transcode_failure_input_index_validation() {
    assert_eq!(
        Ok(()),
        TranscodeFailure::ensure_input_index(3, 3),
        "the end boundary is a valid start index"
    );
    assert_eq!(
        Err(TranscodeFailure::InvalidInputIndex { index: 4, input_len: 3 }),
        TranscodeFailure::ensure_input_index(3, 4),
    );
    assert_eq!(
        "invalid input index 4 for input length 3",
        TranscodeFailure::invalid_input_index(4, 3).to_string()
    );
}

#[test]
fn test_transcode_failure_min_input_validation() {
    assert_eq!(
        Ok(()),
        TranscodeFailure::ensure_min_input(5, 2, 3),
        "exactly the required number of remaining units is enough"
    );
    assert_eq!(
        Err(TranscodeFailure::IncompleteInput {
            input_index: 2,
            required: 4,
            available: 3,
        }),
        TranscodeFailure::ensure_min_input(5, 2, 4),
    );
    assert_eq!(
        Err(TranscodeFailure::InvalidInputIndex { index: 6, input_len: 5 }),
        TranscodeFailure::ensure_min_input(5, 6, 1),
        "an invalid start index is reported before the shortfall"
    );
    assert_eq!(
        "incomplete input at index 2: at least 4 units required to retry, 3 available",
        TranscodeFailure::incomplete_input(2, 4, 3).to_string()
    );
}

#[test]
fn test_transcode_failure_trailing_input_validation() {
    assert_eq!(
        Ok(()),
        TranscodeFailure::ensure_no_trailing_input(2, 2),
        "a fully consumed input has no trailing units"
    );
    assert_eq!(
        Err(TranscodeFailure::TrailingInput {
            consumed: 2,
            remaining: 3,
        }),
        TranscodeFailure::ensure_no_trailing_input(2, 5),
    );
    assert_eq!(
        Err(TranscodeFailure::InvalidInputIndex { index: 5, input_len: 2 }),
        TranscodeFailure::ensure_no_trailing_input(5, 2),
        "an over-consumed count is an index error, not trailing input"
    );
    assert_eq!(
        "trailing input after value: consumed 2 units, remaining 3",
        TranscodeFailure::trailing_input(2, 3).to_string()
    );
}

#[test]
fn test_transcode_failure_output_index_validation() {
    assert_eq!(
        Ok(()),
        TranscodeFailure::ensure_output_index(4, 4),
        "the end boundary is a valid start index"
    );
    assert_eq!(
        Err(TranscodeFailure::InvalidOutputIndex {
            index: 5,
            output_len: 4,
        }),
        TranscodeFailure::ensure_output_index(4, 5),
    );
    assert_eq!(
        "invalid output index 5 for output length 4",
        TranscodeFailure::invalid_output_index(5, 4).to_string()
    );
}

#[test]
fn test_transcode_failure_transcode_indices_report_input_first() {
    assert_eq!(
        Ok(()),
        TranscodeFailure::ensure_transcode_indices(3, 3, 4, 4),
        "both indices may sit exactly on their end boundaries"
    );
    assert_eq!(
        Err(TranscodeFailure::InvalidInputIndex { index: 4, input_len: 3 }),
        TranscodeFailure::ensure_transcode_indices(3, 4, 5, 5),
        "an invalid input index wins over an invalid output index"
    );
    assert_eq!(
        Err(TranscodeFailure::InvalidOutputIndex {
            index: 5,
            output_len: 4,
        }),
        TranscodeFailure::ensure_transcode_indices(3, 3, 4, 5),
    );
}

#[test]
fn test_transcode_failure_output_capacity_validation() {
    assert_eq!(
        Ok(()),
        TranscodeFailure::ensure_output_capacity(4, 1, 3),
        "the remaining capacity exactly satisfies the requirement"
    );
    assert_eq!(
        Err(TranscodeFailure::InsufficientOutput {
            output_index: 1,
            required: 4,
            available: 3,
        }),
        TranscodeFailure::ensure_output_capacity(4, 1, 4),
    );
    assert_eq!(
        Err(TranscodeFailure::InvalidOutputIndex {
            index: 5,
            output_len: 4,
        }),
        TranscodeFailure::ensure_output_capacity(4, 5, 0),
        "an invalid start index is reported before the shortfall"
    );
    assert_eq!(
        "insufficient output at index 1: required 4 units, available 3",
        TranscodeFailure::insufficient_output(1, 4, 3).to_string()
    );
}

#[test]
fn test_transcode_failure_reports_allocation_failure() {
    let error = TranscodeFailure::allocation_failed();

    assert_eq!(TranscodeFailure::AllocationFailed, error);
    assert_eq!("output allocation failed", error.to_string());
}

#[test]
fn test_transcode_failure_reports_output_length_overflow() {
    let error = TranscodeFailure::output_length_overflow();

    assert_eq!(TranscodeFailure::OutputLengthOverflow, error);
    assert_eq!("output length arithmetic overflow", error.to_string());
}

#[test]
fn test_transcode_failure_reports_lifecycle_output_bounds() {
    let error = TranscodeFailure::unsupported_decode_lifecycle_output(2, 3);

    assert_eq!(
        TranscodeFailure::UnsupportedDecodeLifecycleOutput {
            reset_bound: 2,
            finish_bound: 3,
        },
        error
    );
    assert_eq!(
        "strict single-value decoding does not support lifecycle output: reset bound 2, finish bound 3",
        error.to_string()
    );
}

#[test]
fn test_transcode_failure_lifecycle_misuse_messages() {
    assert_eq!(
        "transcode called after finish without an intervening reset",
        TranscodeFailure::TranscodeAfterFinish.to_string()
    );
    assert_eq!(
        "transcode called before reset",
        TranscodeFailure::TranscodeBeforeReset.to_string()
    );
    assert_eq!(
        "finish called twice without an intervening reset",
        TranscodeFailure::FinishAfterFinish.to_string()
    );
    assert_eq!(
        "finish called before reset",
        TranscodeFailure::FinishBeforeReset.to_string()
    );
    assert_eq!(
        "transcoder lifecycle is poisoned; a successful reset is required",
        TranscodeFailure::LifecyclePoisoned.to_string()
    );
}
