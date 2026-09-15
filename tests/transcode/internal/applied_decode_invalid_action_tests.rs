// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::engine::DecodeInvalidAction;

#[test]
fn test_decode_invalid_actions_preserve_consumption_and_replacement() {
    let consumed = crate::nonzero(2);
    let skip = DecodeInvalidAction::<u8>::Skip { consumed };
    let replacement = DecodeInvalidAction::Emit {
        value: b'?',
        consumed,
    };

    assert!(matches!(
        skip,
        DecodeInvalidAction::Skip { consumed: width } if width == consumed
    ));
    assert!(matches!(
        replacement,
        DecodeInvalidAction::Emit {
            value: b'?',
            consumed: width,
        } if width == consumed
    ));
}
