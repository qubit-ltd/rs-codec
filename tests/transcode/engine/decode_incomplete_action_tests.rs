// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for public incomplete-decode policy actions.

use qubit_codec::engine::DecodeIncompleteAction;

#[test]
fn test_decode_incomplete_action_variants_are_public() {
    assert_eq!(
        DecodeIncompleteAction::<u8>::Reject,
        DecodeIncompleteAction::Reject,
    );
    assert_eq!(
        DecodeIncompleteAction::<u8>::Skip,
        DecodeIncompleteAction::Skip,
    );
    assert_eq!(
        DecodeIncompleteAction::Emit { value: 7_u8 },
        DecodeIncompleteAction::Emit { value: 7 },
    );
}
