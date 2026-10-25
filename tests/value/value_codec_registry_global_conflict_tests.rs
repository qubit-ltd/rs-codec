// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::Error;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_codec::__private::inventory::submit;
use qubit_codec::ValueBytesCodecDescriptor;
use qubit_codec::ValueBytesCodecRegistration;
use qubit_codec::ValueBytesCodecRegistrationFactory;
use qubit_codec::ValueBytesCodecRegistry;
use qubit_codec::ValueCodecId;
use qubit_codec::ValueCodecRegistration;
use qubit_codec::ValueCodecRegistrationSource;
use qubit_codec::ValueCodecRegistryError;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_codec::ValueStringCodecDescriptor;
use qubit_codec::ValueStringCodecRegistration;
use qubit_codec::ValueStringCodecRegistrationFactory;
use qubit_codec::ValueStringCodecRegistry;

#[derive(Default)]
struct StringCodec;

impl ValueEncoder<u8> for StringCodec {
    type Output = String;
    type Error = Error;

    fn encode(&mut self, input: &u8) -> Result<Self::Output, Self::Error> {
        Ok(input.to_string())
    }
}

impl ValueDecoder<str> for StringCodec {
    type Output = u8;
    type Error = Error;

    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        input.parse().map_err(Error::other)
    }
}

#[derive(Default)]
struct BytesCodec;

impl ValueEncoder<u8> for BytesCodec {
    type Output = Vec<u8>;
    type Error = Error;

    fn encode(&mut self, input: &u8) -> Result<Self::Output, Self::Error> {
        Ok(vec![*input])
    }
}

impl ValueDecoder<[u8]> for BytesCodec {
    type Output = u8;
    type Error = Error;

    fn decode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        let [byte] = input else {
            return Err(Error::other("expected exactly one byte"));
        };
        Ok(*byte)
    }
}

static STRING_CALLS: AtomicUsize = AtomicUsize::new(0);
static STRING_DESCRIPTOR: ValueStringCodecDescriptor = ValueStringCodecDescriptor::of::<StringCodec, u8>();
static BYTES_CALLS: AtomicUsize = AtomicUsize::new(0);
static BYTES_DESCRIPTOR: ValueBytesCodecDescriptor = ValueBytesCodecDescriptor::of::<BytesCodec, u8>();

fn string_z() -> ValueStringCodecRegistration {
    STRING_CALLS.fetch_add(1, Ordering::SeqCst);
    ValueCodecRegistration::new(
        ValueCodecId::new("example.duplicate"),
        &STRING_DESCRIPTOR,
        ValueCodecRegistrationSource::new("fixture", "z", "z.rs", 3),
    )
}

fn string_a() -> ValueStringCodecRegistration {
    STRING_CALLS.fetch_add(1, Ordering::SeqCst);
    ValueCodecRegistration::new(
        ValueCodecId::new("example.duplicate"),
        &STRING_DESCRIPTOR,
        ValueCodecRegistrationSource::new("fixture", "a", "a.rs", 1),
    )
}

fn string_m() -> ValueStringCodecRegistration {
    STRING_CALLS.fetch_add(1, Ordering::SeqCst);
    ValueCodecRegistration::new(
        ValueCodecId::new("example.duplicate"),
        &STRING_DESCRIPTOR,
        ValueCodecRegistrationSource::new("fixture", "m", "m.rs", 2),
    )
}

submit! { ValueStringCodecRegistrationFactory(string_z) }
submit! { ValueStringCodecRegistrationFactory(string_a) }
submit! { ValueStringCodecRegistrationFactory(string_m) }

fn bytes_z() -> ValueBytesCodecRegistration {
    BYTES_CALLS.fetch_add(1, Ordering::SeqCst);
    ValueCodecRegistration::new(
        ValueCodecId::new("example.duplicate"),
        &BYTES_DESCRIPTOR,
        ValueCodecRegistrationSource::new("fixture", "z", "z.rs", 3),
    )
}

fn bytes_a() -> ValueBytesCodecRegistration {
    BYTES_CALLS.fetch_add(1, Ordering::SeqCst);
    ValueCodecRegistration::new(
        ValueCodecId::new("example.duplicate"),
        &BYTES_DESCRIPTOR,
        ValueCodecRegistrationSource::new("fixture", "a", "a.rs", 1),
    )
}

fn bytes_m() -> ValueBytesCodecRegistration {
    BYTES_CALLS.fetch_add(1, Ordering::SeqCst);
    ValueCodecRegistration::new(
        ValueCodecId::new("example.duplicate"),
        &BYTES_DESCRIPTOR,
        ValueCodecRegistrationSource::new("fixture", "m", "m.rs", 2),
    )
}

submit! { ValueBytesCodecRegistrationFactory(bytes_z) }
submit! { ValueBytesCodecRegistrationFactory(bytes_a) }
submit! { ValueBytesCodecRegistrationFactory(bytes_m) }

fn expected_sources() -> Vec<ValueCodecRegistrationSource> {
    vec![
        ValueCodecRegistrationSource::new("fixture", "a", "a.rs", 1),
        ValueCodecRegistrationSource::new("fixture", "m", "m.rs", 2),
        ValueCodecRegistrationSource::new("fixture", "z", "z.rs", 3),
    ]
}

fn duplicate_parts(error: &ValueCodecRegistryError) -> (&'static str, &[ValueCodecRegistrationSource]) {
    let ValueCodecRegistryError::DuplicateId { id, sources } = error;
    (id, sources)
}

fn panic_text(payload: &Box<dyn std::any::Any + Send>) -> &str {
    payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .expect("global registry panic payload must be text")
}

#[test]
fn test_global_string_registry_caches_duplicate_error_and_panic() {
    let first = ValueStringCodecRegistry::try_global().expect_err("duplicate string ID");
    let (id, sources) = duplicate_parts(&first);
    assert_eq!(id, "example.duplicate");
    assert_eq!(sources, expected_sources());
    let first_display = first.to_string();

    let second = ValueStringCodecRegistry::try_global().expect_err("cached duplicate string ID");
    assert_eq!(duplicate_parts(&second), duplicate_parts(&first));
    assert_eq!(second.to_string(), first_display);
    assert_eq!(STRING_CALLS.load(Ordering::SeqCst), 3);

    let panic = std::panic::catch_unwind(ValueStringCodecRegistry::global)
        .expect_err("global string registry must panic on duplicate IDs");
    assert_eq!(
        panic_text(&panic),
        format!("invalid global value string codec registry: {first_display}")
    );
    assert_eq!(STRING_CALLS.load(Ordering::SeqCst), 3);
}

#[test]
fn test_global_bytes_registry_caches_duplicate_error_and_panic() {
    let first = ValueBytesCodecRegistry::try_global().expect_err("duplicate bytes ID");
    let (id, sources) = duplicate_parts(&first);
    assert_eq!(id, "example.duplicate");
    assert_eq!(sources, expected_sources());
    let first_display = first.to_string();

    let second = ValueBytesCodecRegistry::try_global().expect_err("cached duplicate bytes ID");
    assert_eq!(duplicate_parts(&second), duplicate_parts(&first));
    assert_eq!(second.to_string(), first_display);
    assert_eq!(BYTES_CALLS.load(Ordering::SeqCst), 3);

    let panic = std::panic::catch_unwind(ValueBytesCodecRegistry::global)
        .expect_err("global bytes registry must panic on duplicate IDs");
    assert_eq!(
        panic_text(&panic),
        format!("invalid global value bytes codec registry: {first_display}")
    );
    assert_eq!(BYTES_CALLS.load(Ordering::SeqCst), 3);
}
