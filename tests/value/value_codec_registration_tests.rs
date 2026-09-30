// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests distributed value-codec registrations and the registration macros.

use std::any::TypeId;
use std::io::Error;
use std::ptr;

use qubit_codec::ValueBytesCodecDescriptor;
use qubit_codec::ValueBytesCodecRegistration;
use qubit_codec::ValueBytesCodecRegistry;
use qubit_codec::ValueCodecId;
use qubit_codec::ValueCodecRegistration;
use qubit_codec::ValueCodecRegistrationSource;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_codec::ValueStringCodecDescriptor;
use qubit_codec::ValueStringCodecRegistration;
use qubit_codec::ValueStringCodecRegistry;
use qubit_codec::register_value_bytes_codec;
use qubit_codec::register_value_string_codec;

/// Registration ID linked by `register_value_string_codec!` below.
const MACRO_STRING_ID: &str = "mirror.registration.string";
/// Registration ID linked by `register_value_bytes_codec!` below.
const MACRO_BYTES_ID: &str = "mirror.registration.bytes";

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

register_value_string_codec!(
    id = "mirror.registration.string",
    codec = MirrorU32StringCodec,
    value = u32,
);
register_value_bytes_codec!(
    id = "mirror.registration.bytes",
    codec = MirrorU32BytesCodec,
    value = u32,
);

static STRING_DESCRIPTOR: ValueStringCodecDescriptor = ValueStringCodecDescriptor::of::<MirrorU32StringCodec, u32>();
static BYTES_DESCRIPTOR: ValueBytesCodecDescriptor = ValueBytesCodecDescriptor::of::<MirrorU32BytesCodec, u32>();

const STRING_ID: ValueCodecId = ValueCodecId::new("mirror.registration.const");
const STRING_SOURCE: ValueCodecRegistrationSource =
    ValueCodecRegistrationSource::new("mirror-crate", "mirror::codecs", "src/codecs.rs", 12);

static STRING_REGISTRATION: ValueStringCodecRegistration =
    ValueCodecRegistration::new(STRING_ID, &STRING_DESCRIPTOR, STRING_SOURCE);

#[test]
fn test_getters_return_the_constructor_inputs() {
    assert_eq!(
        STRING_REGISTRATION.id(),
        STRING_ID,
        "the registration must report the identifier it was filed under"
    );
    assert!(
        ptr::eq(STRING_REGISTRATION.descriptor(), &STRING_DESCRIPTOR),
        "the registration must expose the exact static descriptor it was built from"
    );
    assert_eq!(
        STRING_REGISTRATION.source(),
        STRING_SOURCE,
        "the registration must report the declaration site it was built from"
    );
}

#[test]
fn test_descriptor_is_borrowed_for_the_whole_program_run() {
    let first = STRING_REGISTRATION.descriptor();
    let second = STRING_REGISTRATION.descriptor();

    assert!(
        ptr::eq(first, second),
        "repeated descriptor access must return the same borrowed static, not fresh state"
    );
}

#[test]
fn test_copying_a_registration_keeps_both_copies_pointing_at_one_descriptor() {
    let copied = STRING_REGISTRATION;

    assert_eq!(
        copied.id(),
        STRING_REGISTRATION.id(),
        "copying a registration must not change the reported identifier"
    );
    assert!(
        ptr::eq(copied.descriptor(), STRING_REGISTRATION.descriptor()),
        "both copies must share one borrowed static descriptor"
    );
}

#[test]
fn test_two_registrations_may_share_one_descriptor_under_different_ids() {
    static SECOND_REGISTRATION: ValueStringCodecRegistration = ValueCodecRegistration::new(
        ValueCodecId::new("mirror.registration.second"),
        &STRING_DESCRIPTOR,
        ValueCodecRegistrationSource::new("mirror-crate", "mirror::codecs", "src/other.rs", 1),
    );

    assert!(
        ptr::eq(SECOND_REGISTRATION.descriptor(), STRING_REGISTRATION.descriptor()),
        "one executable descriptor may be registered under several stable IDs"
    );
    assert_ne!(
        SECOND_REGISTRATION.id(),
        STRING_REGISTRATION.id(),
        "the two registrations must remain distinguishable by their stable IDs"
    );
}

#[test]
fn test_a_registration_can_be_frozen_into_a_local_registry() {
    let registry = ValueStringCodecRegistry::from_registrations([&STRING_REGISTRATION])
        .expect("a single unique registration must freeze successfully");

    let found = registry
        .get(STRING_ID.as_str())
        .expect("the registration must be reachable by its stable ID");
    assert_eq!(
        found.source().file(),
        "src/codecs.rs",
        "the registry must retain the declaration site for duplicate diagnostics"
    );
}

#[test]
fn test_string_registration_macro_links_the_declared_codec_into_the_global_registry() {
    let registration = ValueStringCodecRegistry::global()
        .get(MACRO_STRING_ID)
        .expect("the macro registration must be collected by the global registry");

    assert_eq!(
        registration.descriptor().codec_type_id(),
        TypeId::of::<MirrorU32StringCodec>(),
        "the macro must file the declared codec under the declared ID"
    );
    assert_eq!(
        registration.descriptor().encode(&7_u32).expect("encode"),
        "7",
        "the linked codec must remain executable through its registration"
    );
}

#[test]
fn test_bytes_registration_macro_links_the_declared_codec_into_the_global_registry() {
    let registration = ValueBytesCodecRegistry::global()
        .get(MACRO_BYTES_ID)
        .expect("the macro registration must be collected by the global registry");

    assert_eq!(
        registration.descriptor().codec_type_id(),
        TypeId::of::<MirrorU32BytesCodec>(),
        "the macro must file the declared codec under the declared ID"
    );
    assert_eq!(
        registration.descriptor().encode(&7_u32).expect("encode"),
        vec![0, 0, 0, 7],
        "the linked bytes codec must remain executable through its registration"
    );
}

#[test]
fn test_registration_macro_captures_the_declaration_site() {
    let registration = ValueStringCodecRegistry::global()
        .get(MACRO_STRING_ID)
        .expect("the macro registration must be collected by the global registry");
    let source = registration.source();

    assert_eq!(
        source.crate_name(),
        env!("CARGO_PKG_NAME"),
        "the macro must record the declaring package name"
    );
    assert_eq!(
        source.module_path(),
        "tests::value::value_codec_registration_tests",
        "the macro must record the declaring module, not the lookup site"
    );
    assert_eq!(
        source.file(),
        "tests/value/value_codec_registration_tests.rs",
        "the macro must record the declaring file"
    );
    assert!(
        source.line() > 0,
        "the macro must record a one-based declaration line, got {}",
        source.line()
    );
}

#[test]
fn test_string_and_bytes_registration_aliases_expose_the_same_registration_shape() {
    static BYTES_REGISTRATION: ValueBytesCodecRegistration = ValueCodecRegistration::new(
        ValueCodecId::new("mirror.registration.bytes.static"),
        &BYTES_DESCRIPTOR,
        ValueCodecRegistrationSource::new("mirror-crate", "mirror::codecs", "src/codecs.rs", 12),
    );

    assert_eq!(
        BYTES_REGISTRATION.source(),
        STRING_REGISTRATION.source(),
        "both wire-specific aliases must expose the same registration contract"
    );
    assert_eq!(
        BYTES_REGISTRATION.descriptor().value_type_id(),
        TypeId::of::<u32>(),
        "the bytes alias must expose its bytes descriptor"
    );
    assert_eq!(
        STRING_REGISTRATION.descriptor().value_type_id(),
        TypeId::of::<u32>(),
        "the string alias must expose its string descriptor"
    );
}
