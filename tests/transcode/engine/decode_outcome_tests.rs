// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for decode-side one-step outcomes.

use qubit_codec::DecodeOutcome;

#[test]
fn test_emitted_creates_emitted_outcome() {
    assert_eq!(
        DecodeOutcome::Emitted {
            read: qubit_io::nz!(2),
            emitted: qubit_io::nz!(1),
        },
        DecodeOutcome::emitted(qubit_io::nz!(2), qubit_io::nz!(1)),
    );
}

#[test]
fn test_skipped_creates_skipped_outcome() {
    assert_eq!(
        DecodeOutcome::Skipped {
            read: qubit_io::nz!(3),
        },
        DecodeOutcome::skipped(qubit_io::nz!(3)),
    );
}

#[test]
fn test_need_input_creates_need_input_outcome() {
    assert_eq!(
        DecodeOutcome::NeedInput {
            required: qubit_io::nz!(4),
        },
        DecodeOutcome::need_input(qubit_io::nz!(4)),
    );
}
