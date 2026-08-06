// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::TranscodeStatus;

#[test]
fn test_transcoder_status_variants_are_distinct() {
    assert_ne!(
        TranscodeStatus::Complete,
        TranscodeStatus::NeedInput {
            required: crate::nonzero(1),
        }
    );
    assert_ne!(
        TranscodeStatus::NeedInput {
            required: crate::nonzero(1),
        },
        TranscodeStatus::NeedOutput {
            required: crate::nonzero(1),
        }
    );
}

#[test]
fn test_transcoder_status_constructors_create_expected_variants() {
    assert_eq!(
        TranscodeStatus::NeedInput {
            required: crate::nonzero(2),
        },
        TranscodeStatus::need_input(crate::nonzero(2)),
    );
    assert_eq!(
        TranscodeStatus::NeedOutput {
            required: crate::nonzero(3),
        },
        TranscodeStatus::need_output(crate::nonzero(3)),
    );
}
