// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::engine::EncodeUnencodableAction;

#[test]
fn test_encode_unencodable_action_constructors() {
    assert_eq!(EncodeUnencodableAction::<u8>::reject(), EncodeUnencodableAction::Reject,);
    assert_eq!(
        EncodeUnencodableAction::replace(7_u8),
        EncodeUnencodableAction::Replace { value: 7 },
    );
}

#[test]
fn test_encode_unencodable_action_skip_variant_is_public() {
    assert_eq!(EncodeUnencodableAction::<u8>::Skip, EncodeUnencodableAction::Skip,);
}
