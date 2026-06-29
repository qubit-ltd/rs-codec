// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{TranscodeContractError, TranscodeProgress, TranscodeStatus};

#[test]
fn test_transcoder_progress_exposes_status_and_counts() {
    let complete = TranscodeProgress::complete(2, 3);
    assert_eq!(TranscodeStatus::Complete, complete.status());
    assert_eq!(2, complete.read());
    assert_eq!(3, complete.written());

    let status = TranscodeStatus::NeedInput {
        input_index: 0,
        required: crate::nz(1),
        available: 0,
    };
    assert!(matches!(
        TranscodeProgress::new(status, 1, 1).status(),
        TranscodeStatus::NeedInput { .. },
    ));
    let status = TranscodeStatus::NeedInput {
        input_index: 4,
        required: crate::nz(3),
        available: 1,
    };
    let need_input = TranscodeProgress::new(status, 1, 2);
    assert_eq!(
        TranscodeStatus::need_input(4, crate::nz(3), 1),
        need_input.status()
    );
    assert_eq!(1, need_input.read());
    assert_eq!(2, need_input.written());

    let status = TranscodeStatus::NeedOutput {
        output_index: 0,
        required: crate::nz(1),
        available: 0,
    };
    assert!(matches!(
        TranscodeProgress::new(status, 1, 0).status(),
        TranscodeStatus::NeedOutput { .. },
    ));
    let status = TranscodeStatus::NeedOutput {
        output_index: 7,
        required: crate::nz(8),
        available: 9,
    };
    let need_output = TranscodeProgress::new(status, 5, 6);
    assert_eq!(
        TranscodeStatus::need_output(7, crate::nz(8), 9),
        need_output.status()
    );
    assert_eq!(5, need_output.read());
    assert_eq!(6, need_output.written());
}

#[test]
fn test_transcoder_progress_constructors_create_expected_progress() {
    let need_input = TranscodeProgress::need_input(4, crate::nz(2), 1, 5, 6);
    assert_eq!(
        TranscodeStatus::need_input(4, crate::nz(2), 1),
        need_input.status()
    );
    assert_eq!(5, need_input.read());
    assert_eq!(6, need_input.written());

    let need_output = TranscodeProgress::need_output(7, crate::nz(3), 0, 8, 9);
    assert_eq!(
        TranscodeStatus::need_output(7, crate::nz(3), 0),
        need_output.status()
    );
    assert_eq!(8, need_output.read());
    assert_eq!(9, need_output.written());
}

#[test]
fn test_transcoder_progress_predicates_match_status() {
    let complete = TranscodeProgress::complete(2, 3);
    assert!(complete.is_complete());
    assert!(!complete.is_need_input());
    assert!(!complete.is_need_output());

    let need_input = TranscodeProgress::need_input(4, crate::nz(2), 1, 5, 6);
    assert!(!need_input.is_complete());
    assert!(need_input.is_need_input());
    assert!(!need_input.is_need_output());

    let need_output = TranscodeProgress::need_output(7, crate::nz(3), 0, 8, 9);
    assert!(!need_output.is_complete());
    assert!(!need_output.is_need_input());
    assert!(need_output.is_need_output());
}

#[test]
fn test_transcoder_progress_validate_accepts_consistent_progress() {
    let complete = TranscodeProgress::complete(2, 3);
    assert_eq!(Ok(()), complete.validate(10, 2, 20, 3));

    let need_input = TranscodeProgress::need_input(12, crate::nz(4), 1, 2, 3);
    assert_eq!(Ok(()), need_input.validate(10, 3, 20, 5));

    let need_output = TranscodeProgress::need_output(23, crate::nz(4), 1, 2, 3);
    assert_eq!(Ok(()), need_output.validate(10, 4, 20, 4));
}

#[test]
fn test_transcoder_progress_validate_rejects_counter_bounds() {
    let progress = TranscodeProgress::complete(3, 1);
    assert_eq!(
        Err(TranscodeContractError::OverRead {
            read: 3,
            available: 2,
        }),
        progress.validate(0, 2, 0, 1),
    );

    let progress = TranscodeProgress::complete(1, 3);
    assert_eq!(
        Err(TranscodeContractError::OverWritten {
            written: 3,
            available: 2,
        }),
        progress.validate(0, 1, 0, 2),
    );
}

#[test]
fn test_transcoder_progress_validate_rejects_status_index_mismatch() {
    let need_input = TranscodeProgress::need_input(11, crate::nz(3), 1, 2, 0);
    assert_eq!(
        Err(TranscodeContractError::StatusIndexMismatch {
            reported: 11,
            expected: 12,
        }),
        need_input.validate(10, 3, 20, 0),
    );

    let need_output = TranscodeProgress::need_output(22, crate::nz(3), 1, 0, 2);
    assert_eq!(
        Err(TranscodeContractError::StatusIndexMismatch {
            reported: 22,
            expected: 23,
        }),
        need_output.validate(10, 0, 21, 3),
    );
}

#[test]
fn test_transcoder_progress_validate_rejects_satisfied_requirements() {
    let need_input = TranscodeProgress::need_input(12, crate::nz(1), 1, 2, 0);
    assert_eq!(
        Err(TranscodeContractError::SatisfiedNeed {
            required: 1,
            available: 1,
        }),
        need_input.validate(10, 3, 20, 0),
    );

    let need_output = TranscodeProgress::need_output(23, crate::nz(1), 1, 0, 3);
    assert_eq!(
        Err(TranscodeContractError::SatisfiedNeed {
            required: 1,
            available: 1,
        }),
        need_output.validate(10, 0, 20, 4),
    );
}

#[test]
fn test_transcoder_progress_validate_rejects_status_available_mismatch() {
    let need_input = TranscodeProgress::need_input(12, crate::nz(4), 0, 2, 0);
    assert_eq!(
        Err(TranscodeContractError::StatusAvailableMismatch {
            reported: 0,
            expected: 1,
        }),
        need_input.validate(10, 3, 20, 0),
    );

    let need_output = TranscodeProgress::need_output(23, crate::nz(4), 0, 0, 3);
    assert_eq!(
        Err(TranscodeContractError::StatusAvailableMismatch {
            reported: 0,
            expected: 1,
        }),
        need_output.validate(10, 0, 20, 4),
    );
}

#[test]
fn test_transcoder_progress_validate_rejects_index_overflow() {
    let need_input = TranscodeProgress::need_input(usize::MAX, crate::nz(2), 0, 1, 0);
    assert_eq!(
        Err(TranscodeContractError::ProgressIndexOverflow {
            index: usize::MAX,
            advanced: 1,
        }),
        need_input.validate(usize::MAX, 1, 0, 0),
    );

    let need_output = TranscodeProgress::need_output(usize::MAX, crate::nz(2), 0, 0, 1);
    assert_eq!(
        Err(TranscodeContractError::ProgressIndexOverflow {
            index: usize::MAX,
            advanced: 1,
        }),
        need_output.validate(0, 0, usize::MAX, 1),
    );
}
