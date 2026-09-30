// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests the inventory factory that builds bytes value-codec registrations.

use std::io::Error;
use std::ptr;

use qubit_codec::ValueBytesCodecDescriptor;
use qubit_codec::ValueBytesCodecRegistration;
use qubit_codec::ValueBytesCodecRegistrationFactory;
use qubit_codec::ValueBytesCodecRegistry;
use qubit_codec::ValueCodecId;
use qubit_codec::ValueCodecRegistration;
use qubit_codec::ValueCodecRegistrationSource;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_codec::ValueStringCodecDescriptor;

#[derive(Default)]
struct MirrorU32BytesCodec;

impl ValueEncoder<u32> for MirrorU32BytesCodec {
    type Output = Vec<u8>;
    type Error = Error;

    fn encode(&mut self, input: &u32) -> Result<Self::Output, Self::Error> {
        Ok(input.to_be_bytes().to_vec())
    }
}

impl ValueDecoder<[u8]> for MirrorU32BytesCodec {
    type Output = u32;
    type Error = Error;

    fn decode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        let bytes: [u8; 4] = input
            .try_into()
            .map_err(|_| Error::other("expected exactly four bytes"))?;
        Ok(u32::from_be_bytes(bytes))
    }
}

#[derive(Default)]
struct MirrorU32StringCodec;

impl ValueEncoder<u32> for MirrorU32StringCodec {
    type Output = String;
    type Error = Error;

    fn encode(&mut self, input: &u32) -> Result<Self::Output, Self::Error> {
        Ok(input.to_string())
    }
}

impl ValueDecoder<str> for MirrorU32StringCodec {
    type Output = u32;
    type Error = Error;

    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        input.parse().map_err(Error::other)
    }
}

static BYTES_DESCRIPTOR: ValueBytesCodecDescriptor = ValueBytesCodecDescriptor::of::<MirrorU32BytesCodec, u32>();
static STRING_DESCRIPTOR: ValueStringCodecDescriptor = ValueStringCodecDescriptor::of::<MirrorU32StringCodec, u32>();

fn build_first() -> ValueBytesCodecRegistration {
    ValueCodecRegistration::new(
        ValueCodecId::new("mirror.bytes.factory.first"),
        &BYTES_DESCRIPTOR,
        ValueCodecRegistrationSource::new("mirror-crate", "mirror::factory", "src/factory.rs", 23),
    )
}

fn build_second() -> ValueBytesCodecRegistration {
    ValueCodecRegistration::new(
        ValueCodecId::new("mirror.bytes.factory.second"),
        &BYTES_DESCRIPTOR,
        ValueCodecRegistrationSource::new("mirror-crate", "mirror::factory", "src/other.rs", 5),
    )
}

static FIRST: ValueBytesCodecRegistration = ValueCodecRegistration::new(
    ValueCodecId::new("mirror.bytes.factory.static"),
    &BYTES_DESCRIPTOR,
    ValueCodecRegistrationSource::new("mirror-crate", "mirror::factory", "src/factory.rs", 23),
);

#[test]
fn test_invoking_the_factory_returns_the_registration_built_by_its_callback() {
    let factory = ValueBytesCodecRegistrationFactory(build_first);

    let registration = (factory.0)();

    assert_eq!(
        registration.id().as_str(),
        "mirror.bytes.factory.first",
        "the factory must yield exactly what its callback produced"
    );
    assert!(
        ptr::eq(registration.descriptor(), &BYTES_DESCRIPTOR),
        "the built registration must keep borrowing the callback's static descriptor"
    );
}

#[test]
fn test_factory_callback_needs_no_arguments_and_no_registry_context() {
    let factory = ValueBytesCodecRegistrationFactory(build_first);
    let callback: fn() -> ValueBytesCodecRegistration = factory.0;

    let registration = callback();

    assert_eq!(
        registration.source().line(),
        23,
        "a zero-argument callback must be callable without any registry state"
    );
}

#[test]
fn test_two_invocations_produce_independent_registrations_over_one_descriptor() {
    let factory = ValueBytesCodecRegistrationFactory(build_first);

    let first = (factory.0)();
    let second = (factory.0)();

    assert_eq!(
        first.id(),
        second.id(),
        "repeated invocation must build the same stable ID"
    );
    assert!(
        ptr::eq(first.descriptor(), second.descriptor()),
        "repeated invocation must not construct fresh descriptor state"
    );
}

#[test]
fn test_two_factories_serve_different_stable_ids() {
    let first = ValueBytesCodecRegistrationFactory(build_first);
    let second = ValueBytesCodecRegistrationFactory(build_second);

    assert_ne!(
        (first.0)().id(),
        (second.0)().id(),
        "independent factories must remain independently addressable"
    );
}

#[test]
fn test_a_factory_built_registration_freezes_into_a_local_registry() {
    let registry = ValueBytesCodecRegistry::from_registrations([&FIRST])
        .expect("a single unique factory-built registration must freeze");

    let found = registry
        .get("mirror.bytes.factory.static")
        .expect("the factory-built registration must be reachable by its stable ID");
    assert_eq!(
        found.source().module_path(),
        "mirror::factory",
        "the frozen registry must retain the declaration site of the factory callback"
    );
}

#[test]
fn test_factory_built_registration_remains_executable() {
    let factory = ValueBytesCodecRegistrationFactory(build_first);

    let registration = (factory.0)();

    assert_eq!(
        registration.descriptor().encode(&11_u32).expect("encode"),
        vec![0, 0, 0, 11],
        "a factory-built registration must stay executable through its bytes descriptor"
    );
}

#[test]
fn test_bytes_factory_does_not_smuggle_in_the_string_descriptor() {
    let factory = ValueBytesCodecRegistrationFactory(build_first);

    let registration = (factory.0)();

    assert_ne!(
        registration.descriptor().codec_type_id(),
        STRING_DESCRIPTOR.codec_type_id(),
        "a bytes factory must keep the bytes codec the callback declared"
    );
}
