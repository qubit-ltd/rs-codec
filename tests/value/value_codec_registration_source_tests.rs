// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests the link-time source location carried by value-codec registrations.

use qubit_codec::ValueCodecRegistrationSource;

const MACRO_SITE: ValueCodecRegistrationSource = ValueCodecRegistrationSource::new(
    "qubit-codec",
    "value::registration",
    "src/value/value_codec_registration.rs",
    1,
);

#[test]
fn test_each_getter_returns_its_captured_part() {
    let source = ValueCodecRegistrationSource::new(
        "mirror-crate",
        "mirror_crate::codecs::inner",
        "src/codecs/inner.rs",
        4_242,
    );

    assert_eq!(source.crate_name(), "mirror-crate", "crate name must survive");
    assert_eq!(
        source.module_path(),
        "mirror_crate::codecs::inner",
        "module path must survive verbatim, including every separator"
    );
    assert_eq!(source.file(), "src/codecs/inner.rs", "file path must survive verbatim");
    assert_eq!(source.line(), 4_242, "the one-based line must survive");
}

#[test]
fn test_constructor_is_usable_in_const_context() {
    assert_eq!(MACRO_SITE.crate_name(), "qubit-codec");
    assert_eq!(MACRO_SITE.module_path(), "value::registration");
    assert_eq!(MACRO_SITE.file(), "src/value/value_codec_registration.rs");
    assert_eq!(MACRO_SITE.line(), 1);
}

#[test]
fn test_copying_a_source_keeps_both_copies_usable() {
    let original = ValueCodecRegistrationSource::new("a", "m", "f.rs", 7);
    let copied = original;

    assert_eq!(
        original.file(),
        copied.file(),
        "copying a `Copy` source must not invalidate either handle"
    );
    assert_eq!(
        original, copied,
        "both copies must remain equal to the original location"
    );
}

#[test]
fn test_locations_with_identical_prefixes_order_by_line() {
    let earlier = ValueCodecRegistrationSource::new("crate", "module", "codec.rs", 10);
    let later = ValueCodecRegistrationSource::new("crate", "module", "codec.rs", 11);

    assert!(
        earlier < later,
        "two declarations in the same file must order by their declaration line"
    );
}

#[test]
fn test_locations_order_by_crate_then_module_then_file_then_line() {
    let mut locations = [
        ValueCodecRegistrationSource::new("crate_b", "module", "codec.rs", 1),
        ValueCodecRegistrationSource::new("crate_a", "module::late", "codec.rs", 1),
        ValueCodecRegistrationSource::new("crate_a", "module::early", "codec.rs", 9),
    ];

    locations.sort();

    assert_eq!(
        locations.map(|source| (source.crate_name(), source.module_path(), source.line())),
        [
            ("crate_a", "module::early", 9),
            ("crate_a", "module::late", 1),
            ("crate_b", "module", 1),
        ],
        "declaration order must be decided by crate, module, file, then line"
    );
}

#[test]
fn test_identical_locations_compare_equal() {
    let first = ValueCodecRegistrationSource::new("crate", "module", "codec.rs", 12);
    let second = ValueCodecRegistrationSource::new("crate", "module", "codec.rs", 12);

    assert_eq!(
        first, second,
        "the same declaration site must compare equal so duplicate reports stay stable"
    );
}

#[test]
fn test_debug_exposes_all_four_captured_parts() {
    let source = ValueCodecRegistrationSource::new("mirror-crate", "mirror::codecs", "src/codecs.rs", 88);
    let rendered = format!("{source:?}");

    for part in ["mirror-crate", "mirror::codecs", "src/codecs.rs", "88"] {
        assert!(
            rendered.contains(part),
            "the derived Debug output must contain {part:?}, but rendered {rendered:?}"
        );
    }
}
