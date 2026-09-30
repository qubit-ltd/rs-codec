// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests public incomplete-decode policy actions.

use qubit_codec::engine::DecodeIncompleteAction;
use qubit_codec::engine::DecodeIncompleteActionOf;

use crate::common::IdentityCodec;

#[test]
fn test_decode_incomplete_action_debug_distinguishes_every_variant() {
    assert_eq!(
        "Reject",
        format!("{:?}", DecodeIncompleteAction::<u8>::Reject),
        "Reject renders without a payload"
    );
    assert_eq!(
        "Skip",
        format!("{:?}", DecodeIncompleteAction::<u8>::Skip),
        "Skip renders without a payload"
    );
    assert_eq!(
        "Emit { value: 7 }",
        format!("{:?}", DecodeIncompleteAction::<u8>::Emit { value: 7 }),
        "Emit renders its replacement value"
    );
}

#[test]
fn test_decode_incomplete_action_emit_compares_payload_value() {
    let emitted = DecodeIncompleteAction::<u8>::Emit { value: 7 };

    assert_ne!(
        emitted,
        DecodeIncompleteAction::<u8>::Emit { value: 8 },
        "the replacement value takes part in equality"
    );
}

/// `DecodeIncompleteAction` derives `Copy`, so copying is a compile-time
/// contract; the observable half is that the copy still matches a freshly
/// constructed action carrying the same payload.
#[test]
fn test_decode_incomplete_action_copy_matches_fresh_construction() {
    let original = DecodeIncompleteAction::<u8>::Emit { value: 7 };
    let copied = original;

    assert_eq!(
        copied,
        DecodeIncompleteAction::<u8>::Emit { value: 7 },
        "a copied action still carries the payload it was copied from"
    );
}

/// The alias contract is compile-time; the observable half is that an action
/// built through the alias renders exactly like the equivalent action built for
/// the codec's resolved value type.
#[test]
fn test_decode_incomplete_action_of_alias_resolves_codec_value_type() {
    let action: DecodeIncompleteActionOf<IdentityCodec> = DecodeIncompleteAction::Emit { value: 7 };

    assert_eq!(
        format!("{action:?}"),
        format!("{:?}", DecodeIncompleteAction::<u8>::Emit { value: 7 }),
        "DecodeIncompleteActionOf<IdentityCodec> resolves to DecodeIncompleteAction<u8>"
    );
}
