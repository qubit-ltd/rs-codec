// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for buffered decode context snapshots.

use qubit_codec::engine::DecodeContext;

#[test]
fn test_decode_context_reports_relative_progress() {
    let context = DecodeContext::new(2, 7, 3, 5, 4);

    assert_eq!(2, context.input_start());
    assert_eq!(7, context.input_index());
    assert_eq!(3, context.output_start());
    assert_eq!(5, context.output_index());
    assert_eq!(4, context.available());
    assert_eq!(5, context.input_used());
    assert_eq!(2, context.output_written());
}

#[test]
#[should_panic(expected = "decode context input index must not precede input start")]
fn test_decode_context_panics_when_input_index_precedes_input_start() {
    let _ = DecodeContext::new(7, 2, 3, 5, 4);
}

#[test]
#[should_panic(expected = "decode context output index must not precede output start")]
fn test_decode_context_panics_when_output_index_precedes_output_start() {
    let _ = DecodeContext::new(2, 7, 5, 3, 4);
}
