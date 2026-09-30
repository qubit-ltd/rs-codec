// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests the validated stable value-codec identifier protocol.

use std::collections::HashMap;

use qubit_codec::ValueCodecId;
use qubit_codec::ValueCodecIdError;

const CONST_ID: ValueCodecId = ValueCodecId::new("mirror.const.static");

#[test]
fn test_try_new_accepts_minimal_single_letter_segment() {
    let id = ValueCodecId::try_new("a").expect("single ASCII letter is a valid ID");

    assert_eq!(id.as_str(), "a", "the accepted input must be preserved verbatim");
}

#[test]
fn test_try_new_accepts_multi_segment_id_with_digits_and_underscores() {
    let id = ValueCodecId::try_new("mirror.c2pa.Payload_1")
        .expect("dot-separated ASCII segments with digits and underscores are valid");

    assert_eq!(
        id.as_str(),
        "mirror.c2pa.Payload_1",
        "a valid ID must round-trip the exact validated text"
    );
}

#[test]
fn test_try_new_rejects_empty_id() {
    assert_eq!(
        ValueCodecId::try_new(""),
        Err(ValueCodecIdError::Empty),
        "an ID without any bytes must be rejected as Empty, not EmptySegment"
    );
}

#[test]
fn test_try_new_rejects_empty_segments_at_every_boundary() {
    for candidate in [".", ".leading", "trailing.", "double..dot"] {
        assert_eq!(
            ValueCodecId::try_new(candidate),
            Err(ValueCodecIdError::EmptySegment),
            "{candidate:?} must be rejected as an empty dot-delimited segment"
        );
    }
}

#[test]
fn test_try_new_rejects_segments_with_a_disallowed_initial_byte() {
    for candidate in ["9digit", "_underscore", "-dash"] {
        assert_eq!(
            ValueCodecId::try_new(candidate),
            Err(ValueCodecIdError::InvalidSegment),
            "{candidate:?} must be rejected because a segment must start with a letter"
        );
    }
}

#[test]
fn test_try_new_rejects_segments_with_a_disallowed_continuation_byte() {
    for candidate in ["mirror.bad-id", "mirror.has space", "mirror.c2pa/1"] {
        assert_eq!(
            ValueCodecId::try_new(candidate),
            Err(ValueCodecIdError::InvalidSegment),
            "{candidate:?} must be rejected because only letters, digits, and underscores may continue a segment"
        );
    }
}

#[test]
fn test_try_new_rejects_non_ascii_segments() {
    assert_eq!(
        ValueCodecId::try_new("café"),
        Err(ValueCodecIdError::InvalidSegment),
        "the protocol is byte-oriented, so multi-byte UTF-8 bytes must not pass as ASCII"
    );
}

#[test]
fn test_try_new_rejects_a_leading_invalid_segment_before_a_valid_one() {
    assert_eq!(
        ValueCodecId::try_new("9first.valid"),
        Err(ValueCodecIdError::InvalidSegment),
        "validation must reject the leading segment instead of accepting a later valid one"
    );
}

#[test]
#[should_panic(expected = "invalid value codec ID")]
fn test_new_panics_on_an_invalid_id() {
    let _ = ValueCodecId::new("mirror.bad id");
}

#[test]
fn test_new_accepts_the_same_inputs_as_try_new() {
    let constructed = ValueCodecId::new("mirror.new.valid");
    let validated = ValueCodecId::try_new("mirror.new.valid").expect("the same ID must validate");

    assert_eq!(
        constructed, validated,
        "`new` must agree with `try_new` for accepted inputs"
    );
}

#[test]
fn test_const_constructor_is_usable_in_const_context() {
    assert_eq!(
        CONST_ID.as_str(),
        "mirror.const.static",
        "a `const` ID must retain its validated text without runtime validation"
    );
}

#[test]
fn test_as_str_borrows_the_original_static_string_without_copying() {
    let id = ValueCodecId::new("mirror.borrowed.static");

    assert!(
        std::ptr::eq(id.as_str().as_ptr(), "mirror.borrowed.static".as_ptr()),
        "the ID must borrow the caller's static string instead of allocating a copy"
    );
}

#[test]
fn test_copying_an_id_leaves_both_copies_usable() {
    let original = ValueCodecId::new("mirror.copied");
    let copied = original;

    assert_eq!(
        original.as_str(),
        copied.as_str(),
        "copying a `Copy` ID must not invalidate either handle"
    );
}

#[test]
fn test_ids_order_lexicographically_by_validated_text() {
    let mut ids = [
        ValueCodecId::new("ab"),
        ValueCodecId::new("a"),
        ValueCodecId::new("a.b"),
    ];

    ids.sort();

    assert_eq!(
        ids.map(ValueCodecId::as_str),
        ["a", "a.b", "ab"],
        "IDs must order by the byte sequence of their validated text"
    );
}

#[test]
fn test_borrow_lets_a_str_key_address_a_hash_map_of_ids() {
    let mut by_id = HashMap::new();
    by_id.insert(ValueCodecId::new("mirror.map.entry"), 7_u32);

    assert_eq!(
        by_id.get("mirror.map.entry"),
        Some(&7),
        "`Borrow<str>` must allow lookup by the raw ID text without building a `ValueCodecId`"
    );
}

#[test]
fn test_distinct_texts_produce_distinct_ids() {
    assert_ne!(
        ValueCodecId::new("mirror.same"),
        ValueCodecId::new("mirror.Same"),
        "identifier comparison must be case sensitive"
    );
}
