// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::engine::DecodeInvalidAction;

use crate::nonzero;

#[test]
fn test_decode_invalid_action_variants_are_public() {
    assert_eq!(
        DecodeInvalidAction::<u8>::Skip { consumed: nonzero(1) },
        DecodeInvalidAction::Skip { consumed: nonzero(1) },
    );
    assert_eq!(
        DecodeInvalidAction::Emit {
            value: 7_u8,
            consumed: nonzero(1),
        },
        DecodeInvalidAction::Emit {
            value: 7,
            consumed: nonzero(1),
        },
    );
}
