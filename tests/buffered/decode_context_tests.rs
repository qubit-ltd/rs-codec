/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for buffered decode context snapshots.

use qubit_codec::DecodeContext;

#[test]
fn test_decode_context_reports_relative_progress() {
    let context = DecodeContext::new(2, 7, 3, 5, 4);

    assert_eq!(2, context.input_start);
    assert_eq!(7, context.input_index);
    assert_eq!(3, context.output_start);
    assert_eq!(5, context.output_index);
    assert_eq!(4, context.available);
    assert_eq!(5, context.input_used());
    assert_eq!(2, context.output_written());
}
