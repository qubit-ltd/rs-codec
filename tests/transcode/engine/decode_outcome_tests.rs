// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for decode-side one-step outcomes.

use qubit_codec::engine::DecodeOutcome;

#[test]
fn test_emitted_creates_emitted_outcome() {
    assert_eq!(
        DecodeOutcome::Emitted {
            read: crate::nz(2),
            emitted: crate::nz(1),
        },
        DecodeOutcome::emitted(crate::nz(2), crate::nz(1)),
    );
}

#[test]
fn test_skipped_creates_skipped_outcome() {
    assert_eq!(
        DecodeOutcome::Skipped { read: crate::nz(3) },
        DecodeOutcome::skipped(crate::nz(3)),
    );
}

#[test]
fn test_need_input_creates_need_input_outcome() {
    assert_eq!(
        DecodeOutcome::NeedInput {
            required: crate::nz(4),
        },
        DecodeOutcome::need_input(crate::nz(4)),
    );
}
