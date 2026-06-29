// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::EncodeContext;

#[test]
fn test_encode_context_getters_and_parts() {
    let input = 0x8d_u8;
    let mut output = [0_u8; 3];
    let mut context = EncodeContext::new(&input, 11, &mut output, 1);

    assert_eq!(0x8d_u8, *context.input_value());
    assert_eq!(11, context.input_index());
    assert_eq!(1, context.output_index());
    assert_eq!(2, context.available_output());
    assert_eq!(3, context.output().len());
    assert_eq!(&mut [0_u8; 3], context.output());
    let output = context.output();
    output[0] = 1;
    output[1] = 2;
    output[2] = 3;
    let _ = output;

    let (context_input_value, context_input_index, context_output, context_output_index) =
        context.into_parts();

    assert_eq!(0x8d_u8, *context_input_value);
    assert_eq!(11, context_input_index);
    assert_eq!(1, context_output_index);
    assert_eq!([1, 2, 3], context_output);
}

#[test]
fn test_module_compiles() {}
