// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests the messages and error contracts of value-codec ID protocol failures.

use std::error::Error;

use qubit_codec::ValueCodecIdError;

#[test]
fn test_each_variant_renders_its_documented_message() {
    let messages = [
        (ValueCodecIdError::Empty, "value codec ID cannot be empty"),
        (
            ValueCodecIdError::EmptySegment,
            "value codec ID contains an empty segment",
        ),
        (
            ValueCodecIdError::InvalidSegment,
            "value codec ID contains an invalid segment",
        ),
    ];

    for (error, expected) in messages {
        assert_eq!(
            error.to_string(),
            expected,
            "the Display message of {error:?} must identify the violated protocol rule"
        );
    }
}

#[test]
fn test_variant_messages_are_distinguishable() {
    assert_ne!(
        ValueCodecIdError::Empty.to_string(),
        ValueCodecIdError::EmptySegment.to_string(),
        "callers must be able to tell an empty ID from an empty segment"
    );
    assert_ne!(
        ValueCodecIdError::EmptySegment.to_string(),
        ValueCodecIdError::InvalidSegment.to_string(),
        "callers must be able to tell an empty segment from an invalid character"
    );
}

#[test]
fn test_debug_names_the_variant_for_diagnostics() {
    assert_eq!(
        format!("{:?}", ValueCodecIdError::EmptySegment),
        "EmptySegment",
        "the derived Debug output must name the violated protocol rule"
    );
}

#[test]
fn test_variants_are_comparable_for_equality_checks() {
    assert_eq!(
        ValueCodecIdError::InvalidSegment,
        ValueCodecIdError::InvalidSegment,
        "identical protocol violations must compare equal"
    );
    assert_ne!(
        ValueCodecIdError::Empty,
        ValueCodecIdError::InvalidSegment,
        "different protocol violations must not compare equal"
    );
}

#[test]
fn test_copying_an_error_keeps_both_copies_usable() {
    let original = ValueCodecIdError::EmptySegment;
    let copied = original;

    assert_eq!(
        original, copied,
        "copying a `Copy` error must not invalidate either handle"
    );
}

#[test]
fn test_error_has_no_further_source_error() {
    let error: &dyn Error = &ValueCodecIdError::Empty;

    assert!(
        error.source().is_none(),
        "a bare protocol violation must not report a nested source error"
    );
}

#[test]
fn test_error_can_be_raised_through_the_standard_error_trait() {
    fn failing() -> Result<(), ValueCodecIdError> {
        Err(ValueCodecIdError::InvalidSegment)
    }

    let boxed: Box<dyn Error> = Box::new(failing().expect_err("the error must be reported"));

    assert_eq!(
        boxed.to_string(),
        "value codec ID contains an invalid segment",
        "the protocol violation must survive being boxed as a trait object"
    );
}
