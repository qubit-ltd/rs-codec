// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

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
fn test_transcode_failure_reports_allocation_failure() {
    let error = TranscodeFailure::allocation_failed();

    assert_eq!(TranscodeFailure::AllocationFailed, error);
    assert_eq!("output allocation failed", error.to_string());
}
