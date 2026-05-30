/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for generic decode failure metadata.

use qubit_codec::DecodeFailure;

#[test]
fn test_decode_failure_reports_incomplete_input() {
    let failure = DecodeFailure::Incomplete {
        required_total: 2,
        available: 1,
    };

    assert_eq!(Some((2, 1)), failure.incomplete());
    assert_eq!(None, failure.invalid_consumed());
}

#[test]
fn test_decode_failure_reports_invalid_consumption() {
    let failure = DecodeFailure::Invalid { consumed: 3 };

    assert_eq!(None, failure.incomplete());
    assert_eq!(Some(3), failure.invalid_consumed());
}
