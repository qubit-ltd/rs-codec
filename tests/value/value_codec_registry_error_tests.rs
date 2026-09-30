// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests the duplicate-ID failure reported while freezing a value-codec
//! registry.

use std::error::Error;

use qubit_codec::ValueCodecRegistrationSource;
use qubit_codec::ValueCodecRegistryError;

fn source(
    crate_name: &'static str,
    module_path: &'static str,
    file: &'static str,
    line: u32,
) -> ValueCodecRegistrationSource {
    ValueCodecRegistrationSource::new(crate_name, module_path, file, line)
}

#[test]
fn test_duplicate_id_message_names_the_conflicting_id_and_every_source() {
    let error = ValueCodecRegistryError::DuplicateId {
        id: "mirror.duplicate",
        sources: vec![
            source("mirror_a", "mirror_a::codecs", "src/a.rs", 10),
            source("mirror_b", "mirror_b::codecs", "src/b.rs", 20),
        ],
    };

    assert_eq!(
        error.to_string(),
        "duplicate value codec ID mirror.duplicate from [ValueCodecRegistrationSource { \
         crate_name: \"mirror_a\", module_path: \"mirror_a::codecs\", file: \"src/a.rs\", line: 10 }, \
         ValueCodecRegistrationSource { crate_name: \"mirror_b\", module_path: \"mirror_b::codecs\", \
         file: \"src/b.rs\", line: 20 }]",
        "the report must name the conflicting ID and every declaration site that claimed it"
    );
}

#[test]
fn test_duplicate_id_preserves_the_supplied_source_order() {
    let error = ValueCodecRegistryError::DuplicateId {
        id: "mirror.duplicate",
        sources: vec![
            source("zeta", "zeta::codecs", "src/z.rs", 5),
            source("alpha", "alpha::codecs", "src/a.rs", 5),
        ],
    };

    match error {
        ValueCodecRegistryError::DuplicateId { id, sources } => {
            assert_eq!(id, "mirror.duplicate");
            assert_eq!(
                sources
                    .iter()
                    .map(ValueCodecRegistrationSource::crate_name)
                    .collect::<Vec<_>>(),
                ["zeta", "alpha"],
                "the error must report sources in the deterministic order supplied by the registry"
            );
        }
    }
}

#[test]
fn test_duplicate_id_with_no_sources_renders_an_empty_source_list() {
    let error = ValueCodecRegistryError::DuplicateId {
        id: "mirror.duplicate",
        sources: Vec::new(),
    };

    assert_eq!(
        error.to_string(),
        "duplicate value codec ID mirror.duplicate from []",
        "an empty source list must still render the conflicting ID"
    );
}

#[test]
fn test_duplicate_id_can_be_cloned_independently() {
    let error = ValueCodecRegistryError::DuplicateId {
        id: "mirror.duplicate",
        sources: vec![source("mirror_a", "mirror_a::codecs", "src/a.rs", 10)],
    };

    let cloned = error.clone();
    let boxed: Box<dyn Error> = Box::new(error);

    assert_eq!(
        cloned.to_string(),
        boxed.to_string(),
        "a cloned registry error must keep the same diagnostic as the original"
    );
}

#[test]
fn test_duplicate_id_has_no_further_source_error() {
    let error = ValueCodecRegistryError::DuplicateId {
        id: "mirror.duplicate",
        sources: vec![source("mirror_a", "mirror_a::codecs", "src/a.rs", 10)],
    };

    assert!(
        error.source().is_none(),
        "a duplicate-ID conflict originates in the registry and has no source error"
    );
}

#[test]
fn test_duplicate_id_debug_output_names_the_conflicting_id() {
    let error = ValueCodecRegistryError::DuplicateId {
        id: "mirror.duplicate",
        sources: Vec::new(),
    };

    assert!(
        format!("{error:?}").contains("mirror.duplicate"),
        "the derived Debug output must identify the conflicting ID"
    );
}
