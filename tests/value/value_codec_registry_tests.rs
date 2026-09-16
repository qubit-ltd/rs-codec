// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use std::any::TypeId;

use qubit_codec::ValueCodecDescriptor;
use qubit_codec::ValueCodecExecutionError;
use qubit_codec::ValueCodecId;
use qubit_codec::ValueCodecIdError;
use qubit_codec::ValueCodecRegistration;
use qubit_codec::ValueCodecRegistrationSource;
use qubit_codec::ValueCodecRegistry;
use qubit_codec::ValueCodecRegistryError;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_codec::register_value_codec;

#[derive(Default)]
struct U32Codec;

impl ValueEncoder<u32> for U32Codec {
    type Output = String;
    type Error = std::io::Error;

    fn encode(&mut self, input: &u32) -> Result<Self::Output, Self::Error> {
        if *input == u32::MAX {
            Err(std::io::Error::other("encode fixture failure"))
        } else {
            Ok(input.to_string())
        }
    }
}

impl ValueDecoder<str> for U32Codec {
    type Output = u32;
    type Error = std::io::Error;

    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        input.parse().map_err(std::io::Error::other)
    }
}

register_value_codec!(id = "example.u32", codec = U32Codec, value = u32,);

static DESCRIPTOR: ValueCodecDescriptor = ValueCodecDescriptor::of::<U32Codec, u32>();
static FIRST: ValueCodecRegistration = ValueCodecRegistration::new(
    ValueCodecId::new("example.local"),
    &DESCRIPTOR,
    ValueCodecRegistrationSource::new("fixture", "first", "first.rs", 1),
);
static SECOND: ValueCodecRegistration = ValueCodecRegistration::new(
    ValueCodecId::new("example.local"),
    &DESCRIPTOR,
    ValueCodecRegistrationSource::new("fixture", "second", "second.rs", 2),
);

#[test]
fn test_value_codec_descriptor_executes_both_directions() {
    assert_eq!(DESCRIPTOR.codec_type_id(), TypeId::of::<U32Codec>());
    assert_eq!(DESCRIPTOR.codec_type_name(), std::any::type_name::<U32Codec>());
    assert_eq!(DESCRIPTOR.value_type_id(), TypeId::of::<u32>());
    assert_eq!(DESCRIPTOR.value_type_name(), "u32");
    assert!(format!("{DESCRIPTOR:?}").contains("U32Codec"));
    assert_eq!(DESCRIPTOR.encode(&42_u32).expect("encode"), "42");
    let decoded = DESCRIPTOR.decode("42").expect("decode");
    assert_eq!(decoded.downcast_ref::<u32>(), Some(&42));
}

#[test]
fn test_value_codec_descriptor_reports_type_and_domain_errors() {
    let mismatch = DESCRIPTOR.encode(&42_u64).expect_err("wrong type");
    assert!(matches!(mismatch, ValueCodecExecutionError::TypeMismatch { .. }));
    assert!(mismatch.to_string().contains("u32"));

    let encode = DESCRIPTOR.encode(&u32::MAX).expect_err("fixture encode error");
    assert!(matches!(encode, ValueCodecExecutionError::EncodeFailed { .. }));
    assert!(encode.to_string().contains("encode fixture failure"));

    let decode = DESCRIPTOR.decode("not-a-number").expect_err("fixture decode error");
    assert!(matches!(decode, ValueCodecExecutionError::DecodeFailed { .. }));
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
fn test_local_value_codec_registry_owns_and_queries_entries() {
    let registry = ValueCodecRegistry::from_registrations([&FIRST]).expect("valid registry");
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
    assert!(ValueCodecRegistry::empty().registrations().is_empty());
}

#[test]
fn test_value_codec_registry_rejects_duplicate_ids() {
    let error = ValueCodecRegistry::from_registrations([&FIRST, &SECOND]).expect_err("duplicate ID");
    assert!(matches!(error, ValueCodecRegistryError::DuplicateId { .. }));
    assert!(error.to_string().contains("example.local"));
}

#[test]
fn test_global_value_codec_registry_collects_macro_registration() {
    let registry = ValueCodecRegistry::try_global().expect("valid global registry");
    assert!(std::ptr::eq(registry, ValueCodecRegistry::global()));
    let registration = registry.get("example.u32").expect("linked registration");
    assert_eq!(registration.descriptor().codec_type_id(), TypeId::of::<U32Codec>());
    assert_eq!(registration.source().crate_name(), env!("CARGO_PKG_NAME"));
}

#[test]
fn test_global_value_codec_registry_initialization_is_unique_across_threads() {
    let addresses = (0..8)
        .map(|_| {
            std::thread::spawn(|| {
                ValueCodecRegistry::try_global().expect("valid global registry") as *const ValueCodecRegistry as usize
            })
        })
        .map(|thread| thread.join().expect("registry thread must complete"))
        .collect::<Vec<_>>();

    assert!(addresses.iter().all(|address| *address == addresses[0]));
}
