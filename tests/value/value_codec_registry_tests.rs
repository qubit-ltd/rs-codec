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
use std::thread;

use qubit_codec::ValueBytesCodecDescriptor;
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

#[test]
fn test_value_string_codec_descriptor_executes_both_directions() {
    assert_eq!(STRING_DESCRIPTOR.codec_type_id(), TypeId::of::<U32StringCodec>());
    assert_eq!(STRING_DESCRIPTOR.codec_type_name(), type_name::<U32StringCodec>());
    assert_eq!(STRING_DESCRIPTOR.value_type_id(), TypeId::of::<u32>());
    assert_eq!(STRING_DESCRIPTOR.value_type_name(), "u32");
    assert!(format!("{STRING_DESCRIPTOR:?}").contains("U32StringCodec"));
    assert_eq!(STRING_DESCRIPTOR.encode(&42_u32).expect("encode"), "42");
    let decoded = STRING_DESCRIPTOR.decode("42").expect("decode");
    assert_eq!(decoded.downcast_ref::<u32>(), Some(&42));
}

#[test]
fn test_value_string_codec_descriptor_reports_type_and_domain_errors() {
    let mismatch = STRING_DESCRIPTOR.encode(&42_u64).expect_err("wrong type");
    assert!(matches!(mismatch, ValueCodecExecutionError::TypeMismatch { .. }));
    assert!(mismatch.to_string().contains("u32"));

    let encode = STRING_DESCRIPTOR.encode(&u32::MAX).expect_err("fixture encode error");
    assert!(matches!(encode, ValueCodecExecutionError::EncodeFailed { .. }));
    assert!(encode.to_string().contains("encode fixture failure"));

    let decode = STRING_DESCRIPTOR
        .decode("not-a-number")
        .expect_err("fixture decode error");
    assert!(matches!(decode, ValueCodecExecutionError::DecodeFailed { .. }));
}

#[test]
fn test_value_bytes_codec_descriptor_executes_both_directions() {
    assert_eq!(BYTES_DESCRIPTOR.codec_type_id(), TypeId::of::<U32BeBytesCodec>());
    assert_eq!(BYTES_DESCRIPTOR.encode(&42_u32).expect("encode"), vec![0, 0, 0, 42]);
    let decoded = BYTES_DESCRIPTOR.decode(&[0, 0, 0, 42]).expect("decode");
    assert_eq!(decoded.downcast_ref::<u32>(), Some(&42));
}

#[test]
fn test_value_codec_id_protocol() {
    assert_eq!(ValueCodecId::new("example.Codec_1").as_str(), "example.Codec_1");
    assert_eq!(ValueCodecId::try_new(""), Err(ValueCodecIdError::Empty));
    assert_eq!(ValueCodecId::try_new("example."), Err(ValueCodecIdError::EmptySegment));
    assert_eq!(
        ValueCodecId::try_new("example..codec"),
        Err(ValueCodecIdError::EmptySegment)
    );
    assert_eq!(
        ValueCodecId::try_new("9example.codec"),
        Err(ValueCodecIdError::InvalidSegment)
    );
    assert_eq!(
        ValueCodecId::try_new("example.bad-id"),
        Err(ValueCodecIdError::InvalidSegment)
    );
}

#[test]
fn test_local_value_string_codec_registry_owns_and_queries_entries() {
    let registry = ValueStringCodecRegistry::from_registrations([&FIRST]).expect("valid registry");
    let registration = registry.get("example.local").expect("local registration");
    assert_eq!(registration.id(), FIRST.id());
    assert_eq!(registration.descriptor().value_type_id(), TypeId::of::<u32>());
    let source = registration.source();
    assert_eq!(source.crate_name(), "fixture");
    assert_eq!(source.module_path(), "first");
    assert_eq!(source.file(), "first.rs");
    assert_eq!(source.line(), 1);
    assert_eq!(registry.registrations().len(), 1);
    assert!(registry.get("missing").is_none());
    assert!(ValueStringCodecRegistry::empty().registrations().is_empty());
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
fn test_global_value_string_codec_registry_collects_macro_registration() {
    let registry = ValueStringCodecRegistry::try_global().expect("valid global registry");
    assert!(ptr::eq(registry, ValueStringCodecRegistry::global()));
    let registration = registry.get("example.u32").expect("linked registration");
    assert_eq!(
        registration.descriptor().codec_type_id(),
        TypeId::of::<U32StringCodec>()
    );
    assert_eq!(registration.source().crate_name(), env!("CARGO_PKG_NAME"));
}

#[test]
fn test_global_value_bytes_codec_registry_collects_macro_registration() {
    let registry = ValueBytesCodecRegistry::try_global().expect("valid global registry");
    assert!(ptr::eq(registry, ValueBytesCodecRegistry::global()));
    let registration = registry.get("example.u32").expect("linked registration");
    assert_eq!(
        registration.descriptor().codec_type_id(),
        TypeId::of::<U32BeBytesCodec>()
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
    assert_eq!(string.id(), bytes.id());
    assert_ne!(string.descriptor().codec_type_id(), bytes.descriptor().codec_type_id());
}

#[test]
fn test_global_value_string_codec_registry_initialization_is_unique_across_threads() {
    let addresses = (0..8)
        .map(|_| {
            thread::spawn(|| {
                ValueStringCodecRegistry::try_global().expect("valid global registry")
                    as *const ValueStringCodecRegistry as usize
            })
        })
        .map(|thread| thread.join().expect("registry thread must complete"))
        .collect::<Vec<_>>();

    assert!(addresses.iter().all(|address| *address == addresses[0]));
}
