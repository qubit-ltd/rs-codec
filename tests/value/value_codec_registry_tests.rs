// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::any::TypeId;
use std::any::type_name;
use std::io::Error;
use std::ptr;
use std::sync::Arc;
use std::sync::Barrier;
use std::thread;

use qubit_codec::ValueBytesCodecDescriptor;
use qubit_codec::ValueBytesCodecRegistration;
use qubit_codec::ValueBytesCodecRegistry;
use qubit_codec::ValueCodecExecutionError;
use qubit_codec::ValueCodecId;
use qubit_codec::ValueCodecIdError;
use qubit_codec::ValueCodecRegistrationSource;
use qubit_codec::ValueCodecRegistryError;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_codec::ValueStringCodecDescriptor;
use qubit_codec::ValueStringCodecRegistration;
use qubit_codec::ValueStringCodecRegistry;
use qubit_codec::register_value_bytes_codec;
use qubit_codec::register_value_string_codec;

#[derive(Default)]
struct U32StringCodec;

impl ValueEncoder<u32> for U32StringCodec {
    type Output = String;
    type Error = Error;

    fn encode(&mut self, input: &u32) -> Result<Self::Output, Self::Error> {
        if *input == u32::MAX {
            Err(Error::other("encode fixture failure"))
        } else {
            Ok(input.to_string())
        }
    }
}

impl ValueDecoder<str> for U32StringCodec {
    type Output = u32;
    type Error = Error;

    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        input.parse().map_err(Error::other)
    }
}

#[derive(Default)]
struct U32BeBytesCodec;

impl ValueEncoder<u32> for U32BeBytesCodec {
    type Output = Vec<u8>;
    type Error = Error;

    fn encode(&mut self, input: &u32) -> Result<Self::Output, Self::Error> {
        Ok(input.to_be_bytes().to_vec())
    }
}

impl ValueDecoder<[u8]> for U32BeBytesCodec {
    type Output = u32;
    type Error = Error;

    fn decode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        let bytes: [u8; 4] = input.try_into().map_err(|_| Error::other("expected four bytes"))?;
        Ok(u32::from_be_bytes(bytes))
    }
}

register_value_string_codec!(id = "example.u32", codec = U32StringCodec, value = u32,);
register_value_bytes_codec!(id = "example.u32", codec = U32BeBytesCodec, value = u32,);

static STRING_DESCRIPTOR: ValueStringCodecDescriptor = ValueStringCodecDescriptor::of::<U32StringCodec, u32>();
static BYTES_DESCRIPTOR: ValueBytesCodecDescriptor = ValueBytesCodecDescriptor::of::<U32BeBytesCodec, u32>();
static FIRST: ValueStringCodecRegistration = ValueStringCodecRegistration::new(
    ValueCodecId::new("example.local"),
    &STRING_DESCRIPTOR,
    ValueCodecRegistrationSource::new("fixture", "first", "first.rs", 1),
);
static SECOND: ValueStringCodecRegistration = ValueStringCodecRegistration::new(
    ValueCodecId::new("example.local"),
    &STRING_DESCRIPTOR,
    ValueCodecRegistrationSource::new("fixture", "second", "second.rs", 2),
);
static THIRD: ValueStringCodecRegistration = ValueStringCodecRegistration::new(
    ValueCodecId::new("example.local"),
    &STRING_DESCRIPTOR,
    ValueCodecRegistrationSource::new("fixture", "third", "third.rs", 3),
);
static SAME_SOURCE: ValueStringCodecRegistration = ValueStringCodecRegistration::new(
    ValueCodecId::new("example.local"),
    &STRING_DESCRIPTOR,
    ValueCodecRegistrationSource::new("fixture", "first", "first.rs", 1),
);
static SMALLER_ID: ValueStringCodecRegistration = ValueStringCodecRegistration::new(
    ValueCodecId::new("example.aaa"),
    &STRING_DESCRIPTOR,
    ValueCodecRegistrationSource::new("fixture", "small", "small.rs", 4),
);
static SMALLER_ID_DUPLICATE: ValueStringCodecRegistration = ValueStringCodecRegistration::new(
    ValueCodecId::new("example.aaa"),
    &STRING_DESCRIPTOR,
    ValueCodecRegistrationSource::new("fixture", "small_duplicate", "small_duplicate.rs", 5),
);
static BYTES_FIRST: ValueBytesCodecRegistration = ValueBytesCodecRegistration::new(
    ValueCodecId::new("example.local"),
    &BYTES_DESCRIPTOR,
    ValueCodecRegistrationSource::new("fixture", "first", "first.rs", 1),
);
static BYTES_SECOND: ValueBytesCodecRegistration = ValueBytesCodecRegistration::new(
    ValueCodecId::new("example.local"),
    &BYTES_DESCRIPTOR,
    ValueCodecRegistrationSource::new("fixture", "second", "second.rs", 2),
);
static BYTES_THIRD: ValueBytesCodecRegistration = ValueBytesCodecRegistration::new(
    ValueCodecId::new("example.local"),
    &BYTES_DESCRIPTOR,
    ValueCodecRegistrationSource::new("fixture", "third", "third.rs", 3),
);

#[test]
fn test_value_string_codec_descriptor_executes_both_directions() {
    assert_eq!(
        STRING_DESCRIPTOR.codec_type_id(),
        TypeId::of::<U32StringCodec>(),
        "descriptor records its codec type"
    );
    assert_eq!(
        STRING_DESCRIPTOR.codec_type_name(),
        type_name::<U32StringCodec>(),
        "descriptor exposes the codec type name"
    );
    assert_eq!(
        STRING_DESCRIPTOR.value_type_id(),
        TypeId::of::<u32>(),
        "descriptor records its value type"
    );
    assert_eq!(
        STRING_DESCRIPTOR.value_type_name(),
        "u32",
        "descriptor exposes the value type name"
    );
    assert!(
        format!("{STRING_DESCRIPTOR:?}").contains("U32StringCodec"),
        "debug output identifies the codec type"
    );
    assert_eq!(
        STRING_DESCRIPTOR.encode(&42_u32).expect("encode"),
        "42",
        "descriptor encodes the value as text"
    );
    let decoded = STRING_DESCRIPTOR.decode("42").expect("decode");
    assert_eq!(
        decoded.downcast_ref::<u32>(),
        Some(&42),
        "descriptor decodes the value back to u32"
    );
}

#[test]
fn test_value_string_codec_descriptor_reports_type_and_domain_errors() {
    let mismatch = STRING_DESCRIPTOR.encode(&42_u64).expect_err("wrong type");
    assert!(
        matches!(mismatch, ValueCodecExecutionError::TypeMismatch { .. }),
        "wrong input types report TypeMismatch"
    );
    assert!(
        mismatch.to_string().contains("u32"),
        "type mismatch describes the expected u32 value"
    );

    let encode = STRING_DESCRIPTOR.encode(&u32::MAX).expect_err("fixture encode error");
    assert!(
        matches!(encode, ValueCodecExecutionError::EncodeFailed { .. }),
        "codec failures report EncodeFailed"
    );
    assert!(
        encode.to_string().contains("encode fixture failure"),
        "encode errors retain their source message"
    );

    let decode = STRING_DESCRIPTOR
        .decode("not-a-number")
        .expect_err("fixture decode error");
    assert!(
        matches!(decode, ValueCodecExecutionError::DecodeFailed { .. }),
        "codec failures report DecodeFailed"
    );
}

#[test]
fn test_value_bytes_codec_descriptor_executes_both_directions() {
    assert_eq!(
        BYTES_DESCRIPTOR.codec_type_id(),
        TypeId::of::<U32BeBytesCodec>(),
        "descriptor records its codec type"
    );
    assert_eq!(
        BYTES_DESCRIPTOR.encode(&42_u32).expect("encode"),
        vec![0, 0, 0, 42],
        "descriptor encodes u32 in big-endian bytes"
    );
    let decoded = BYTES_DESCRIPTOR.decode(&[0, 0, 0, 42]).expect("decode");
    assert_eq!(
        decoded.downcast_ref::<u32>(),
        Some(&42),
        "descriptor decodes bytes back to u32"
    );
}

#[test]
fn test_value_codec_id_protocol() {
    assert_eq!(
        ValueCodecId::new("example.Codec_1").as_str(),
        "example.Codec_1",
        "valid IDs preserve their spelling"
    );
    assert_eq!(
        ValueCodecId::try_new(""),
        Err(ValueCodecIdError::Empty),
        "empty IDs are rejected"
    );
    assert_eq!(
        ValueCodecId::try_new("example."),
        Err(ValueCodecIdError::EmptySegment),
        "trailing separators create an empty segment"
    );
    assert_eq!(
        ValueCodecId::try_new("example..codec"),
        Err(ValueCodecIdError::EmptySegment),
        "consecutive separators create an empty segment"
    );
    assert_eq!(
        ValueCodecId::try_new("9example.codec"),
        Err(ValueCodecIdError::InvalidSegment),
        "segments must start with a valid identifier character"
    );
    assert_eq!(
        ValueCodecId::try_new("example.bad-id"),
        Err(ValueCodecIdError::InvalidSegment),
        "hyphens are rejected in ID segments"
    );
}

#[test]
fn test_local_value_string_codec_registry_owns_and_queries_entries() {
    let registry = ValueStringCodecRegistry::from_registrations([&FIRST]).expect("valid registry");
    let registration = registry.get("example.local").expect("local registration");
    assert_eq!(
        registration.id(),
        FIRST.id(),
        "lookup returns the requested registration ID"
    );
    assert_eq!(
        registration.descriptor().value_type_id(),
        TypeId::of::<u32>(),
        "registration describes u32 values"
    );
    let source = registration.source();
    assert_eq!(
        source.crate_name(),
        "fixture",
        "registration preserves its source crate"
    );
    assert_eq!(
        source.module_path(),
        "first",
        "registration preserves its source module"
    );
    assert_eq!(source.file(), "first.rs", "registration preserves its source file");
    assert_eq!(source.line(), 1, "registration preserves its source line");
    assert_eq!(
        registry.registrations().len(),
        1,
        "the registry owns one registered entry"
    );
    assert!(
        registry.get("missing").is_none(),
        "lookup returns none for an unknown ID"
    );
    assert!(
        ValueStringCodecRegistry::empty().registrations().is_empty(),
        "an empty registry has no registrations"
    );
}

#[test]
fn test_value_string_codec_registry_rejects_duplicate_ids() {
    let error = ValueStringCodecRegistry::from_registrations([&FIRST, &SECOND]).expect_err("duplicate ID");

    let ValueCodecRegistryError::DuplicateId { id, sources } = &error;

    assert_eq!(*id, "example.local", "the error reports the conflicting stable ID");
    assert_eq!(
        sources.as_slice(),
        &[FIRST.source(), SECOND.source()],
        "the error reports every declaration site that claimed the ID, in sorted order"
    );
    assert_eq!(
        error.to_string(),
        format!(
            "duplicate value codec ID example.local from {:?}",
            [FIRST.source(), SECOND.source()]
        ),
        "the Display message carries the conflicting ID and both sources"
    );
}

#[test]
fn test_value_string_codec_registry_reports_all_duplicate_sources_in_any_input_order() {
    let expected_sources = [FIRST.source(), SECOND.source(), THIRD.source()];
    let permutations = [
        [&FIRST, &SECOND, &THIRD],
        [&FIRST, &THIRD, &SECOND],
        [&SECOND, &FIRST, &THIRD],
        [&SECOND, &THIRD, &FIRST],
        [&THIRD, &FIRST, &SECOND],
        [&THIRD, &SECOND, &FIRST],
    ];
    let mut expected_display = None;

    for permutation in permutations {
        let error =
            ValueStringCodecRegistry::from_registrations(permutation).expect_err("all three entries claim the same ID");
        let ValueCodecRegistryError::DuplicateId { id, sources } = &error;
        assert_eq!(*id, "example.local", "every permutation reports the duplicate ID");
        assert_eq!(
            sources.as_slice(),
            &expected_sources,
            "duplicate sources are sorted consistently"
        );
        let display = error.to_string();
        assert_eq!(
            expected_display.get_or_insert(display.clone()),
            &display,
            "duplicate display output is input-order independent"
        );
    }
}

#[test]
fn test_value_bytes_codec_registry_reports_all_duplicate_sources_in_any_input_order() {
    let expected_sources = [BYTES_FIRST.source(), BYTES_SECOND.source(), BYTES_THIRD.source()];
    let permutations = [
        [&BYTES_FIRST, &BYTES_SECOND, &BYTES_THIRD],
        [&BYTES_FIRST, &BYTES_THIRD, &BYTES_SECOND],
        [&BYTES_SECOND, &BYTES_FIRST, &BYTES_THIRD],
        [&BYTES_SECOND, &BYTES_THIRD, &BYTES_FIRST],
        [&BYTES_THIRD, &BYTES_FIRST, &BYTES_SECOND],
        [&BYTES_THIRD, &BYTES_SECOND, &BYTES_FIRST],
    ];

    for permutation in permutations {
        let error =
            ValueBytesCodecRegistry::from_registrations(permutation).expect_err("all three entries claim the same ID");
        let ValueCodecRegistryError::DuplicateId { id, sources } = &error;
        assert_eq!(
            *id, "example.local",
            "every permutation reports the duplicate byte codec ID"
        );
        assert_eq!(
            sources.as_slice(),
            &expected_sources,
            "byte codec duplicate sources are sorted consistently"
        );
    }
}

#[test]
fn test_value_string_codec_registry_keeps_duplicate_sources_and_selects_smallest_id() {
    let error =
        ValueStringCodecRegistry::from_registrations([&FIRST, &SAME_SOURCE, &SMALLER_ID, &SMALLER_ID_DUPLICATE])
            .expect_err("both IDs have duplicates");
    let ValueCodecRegistryError::DuplicateId { id, sources } = &error;

    assert_eq!(*id, "example.aaa", "the lexically smallest conflicting ID wins");
    assert_eq!(
        sources.as_slice(),
        &[SMALLER_ID.source(), SMALLER_ID_DUPLICATE.source()],
        "the selected duplicate ID contains its complete sorted source list"
    );

    let error = ValueStringCodecRegistry::from_registrations([&FIRST, &SAME_SOURCE])
        .expect_err("the same source still represents two registrations");
    let ValueCodecRegistryError::DuplicateId { sources, .. } = &error;
    assert_eq!(
        sources.as_slice(),
        &[FIRST.source(), SAME_SOURCE.source()],
        "identical source metadata remains duplicated"
    );
}

#[test]
fn test_global_value_string_codec_registry_collects_macro_registration() {
    let registry = ValueStringCodecRegistry::try_global().expect("valid global registry");
    assert!(
        ptr::eq(registry, ValueStringCodecRegistry::global()),
        "global accessors return the same string registry"
    );
    let registration = registry.get("example.u32").expect("linked registration");
    assert_eq!(
        registration.descriptor().codec_type_id(),
        TypeId::of::<U32StringCodec>(),
        "the global registration points to the string codec"
    );
    assert_eq!(
        registration.source().crate_name(),
        env!("CARGO_PKG_NAME"),
        "macro registration records the current crate"
    );
}

#[test]
fn test_global_value_bytes_codec_registry_collects_macro_registration() {
    let registry = ValueBytesCodecRegistry::try_global().expect("valid global registry");
    assert!(
        ptr::eq(registry, ValueBytesCodecRegistry::global()),
        "global accessors return the same bytes registry"
    );
    let registration = registry.get("example.u32").expect("linked registration");
    assert_eq!(
        registration.descriptor().codec_type_id(),
        TypeId::of::<U32BeBytesCodec>(),
        "the global registration points to the bytes codec"
    );
}

#[test]
fn test_same_id_may_exist_in_string_and_bytes_registries() {
    let string = ValueStringCodecRegistry::try_global()
        .expect("valid string registry")
        .get("example.u32")
        .expect("string registration");
    let bytes = ValueBytesCodecRegistry::try_global()
        .expect("valid bytes registry")
        .get("example.u32")
        .expect("bytes registration");
    assert_eq!(
        string.id(),
        bytes.id(),
        "separate registries may use the same stable ID"
    );
    assert_ne!(
        string.descriptor().codec_type_id(),
        bytes.descriptor().codec_type_id(),
        "the registries retain distinct codec types"
    );
}

#[test]
fn test_global_value_string_codec_registry_access_is_unique_across_concurrent_threads() {
    let barrier = Arc::new(Barrier::new(8));
    let threads = (0..8)
        .map(|_| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                ValueStringCodecRegistry::try_global().expect("valid global registry")
                    as *const ValueStringCodecRegistry as usize
            })
        })
        .collect::<Vec<_>>();
    let addresses = threads
        .into_iter()
        .map(|thread| thread.join().expect("registry thread must complete"))
        .collect::<Vec<_>>();

    assert!(
        addresses.iter().all(|address| *address == addresses[0]),
        "concurrent global access returns one registry instance"
    );
}
