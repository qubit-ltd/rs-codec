// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::engine::EncodeContext;

#[test]
fn test_encode_context_getters_and_parts() {
    let input = 0x8d_u8;
    let context = EncodeContext::new(&input, 11, 1, 2);

    assert_eq!(0x8d_u8, *context.input_value());
    assert_eq!(11, context.input_index());
    assert_eq!(1, context.output_index());
    assert_eq!(2, context.available_output());
}
