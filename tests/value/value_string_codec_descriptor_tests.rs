// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests the safe type-erased bidirectional string value-codec descriptor.

use std::any::Any;
use std::any::TypeId;
use std::any::type_name;
use std::convert::Infallible;
use std::io::Error;
use std::rc::Rc;

use qubit_codec::ValueCodecExecutionError;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_codec::ValueStringCodecDescriptor;

/// String codec that rejects `u32::MAX` on encode and non-numeric text on
/// decode, so both failure directions stay reachable from one descriptor.
#[derive(Default)]
struct MirrorU32StringCodec;

impl ValueEncoder<u32> for MirrorU32StringCodec {
    type Output = String;
    type Error = Error;

    fn encode(&mut self, input: &u32) -> Result<Self::Output, Self::Error> {
        if *input == u32::MAX {
            Err(Error::other("mirror string encode rejected u32::MAX"))
        } else {
            Ok(input.to_string())
        }
    }
}

impl ValueDecoder<str> for MirrorU32StringCodec {
    type Output = u32;
    type Error = Error;

    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        input
            .parse()
            .map_err(|_| Error::other("mirror string decode needs a decimal number"))
    }
}

/// String codec for `String` used to check ownership of the erased output.
#[derive(Default)]
struct MirrorTextStringCodec;

impl ValueEncoder<String> for MirrorTextStringCodec {
    type Output = String;
    type Error = Error;

    fn encode(&mut self, input: &String) -> Result<Self::Output, Self::Error> {
        Ok(input.clone())
    }
}

#[derive(Default)]
struct FreshStringCodec {
    calls: usize,
}

impl ValueEncoder<String> for FreshStringCodec {
    type Output = String;
    type Error = Error;

    fn encode(&mut self, input: &String) -> Result<String, Error> {
        self.calls += 1;
        Ok(format!("{}:{input}", self.calls))
    }
}

impl ValueDecoder<str> for FreshStringCodec {
    type Output = String;
    type Error = Error;

    fn decode(&mut self, input: &str) -> Result<String, Error> {
        self.calls += 1;
        Ok(format!("{}:{input}", self.calls))
    }
}

impl ValueDecoder<str> for MirrorTextStringCodec {
    type Output = String;
    type Error = Error;

    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

static U32_DESCRIPTOR: ValueStringCodecDescriptor = ValueStringCodecDescriptor::of::<MirrorU32StringCodec, u32>();
static TEXT_DESCRIPTOR: ValueStringCodecDescriptor = ValueStringCodecDescriptor::of::<MirrorTextStringCodec, String>();
static FRESH_STRING: ValueStringCodecDescriptor = ValueStringCodecDescriptor::of::<FreshStringCodec, String>();

#[derive(Default)]
struct LocalValueCodec {
    _local: Rc<()>,
}

impl ValueEncoder<Rc<u32>> for LocalValueCodec {
    type Output = String;
    type Error = Infallible;

    fn encode(&mut self, input: &Rc<u32>) -> Result<String, Infallible> {
        Ok(input.to_string())
    }
}

impl ValueDecoder<str> for LocalValueCodec {
    type Output = Rc<u32>;
    type Error = Infallible;

    fn decode(&mut self, _input: &str) -> Result<Rc<u32>, Infallible> {
        Ok(Rc::new(7))
    }
}

#[test]
fn test_metadata_matches_the_types_selected_at_construction() {
    assert_eq!(U32_DESCRIPTOR.codec_type_id(), TypeId::of::<MirrorU32StringCodec>());
    assert_eq!(
        U32_DESCRIPTOR.codec_type_name(),
        type_name::<MirrorU32StringCodec>(),
        "the codec name must be diagnostic metadata, not a stable ID"
    );
    assert_eq!(U32_DESCRIPTOR.value_type_id(), TypeId::of::<u32>());
    assert_eq!(
        U32_DESCRIPTOR.value_type_name(),
        "u32",
        "a primitive value type must report its plain diagnostic name"
    );
}

#[test]
fn test_debug_output_lists_both_names_and_is_non_exhaustive() {
    let rendered = format!("{U32_DESCRIPTOR:?}");

    for fragment in [
        "ValueStringCodecDescriptor",
        "codec_type_name",
        type_name::<MirrorU32StringCodec>(),
        "value_type_name",
        "u32",
        "..",
    ] {
        assert!(
            rendered.contains(fragment),
            "the Debug output must contain {fragment:?}, but rendered {rendered:?}"
        );
    }
}

#[test]
fn test_type_mismatch_reports_the_declared_and_actual_types() {
    let error = U32_DESCRIPTOR
        .encode(&42_u64)
        .expect_err("a `u64` must not be accepted by a `u32` descriptor");

    match error {
        ValueCodecExecutionError::TypeMismatch {
            expected_type,
            actual_type,
        } => {
            assert_eq!(
                expected_type,
                type_name::<u32>(),
                "the mismatch must name the value type declared by the descriptor"
            );
            assert_eq!(
                actual_type,
                TypeId::of::<u64>(),
                "the mismatch must report the process-local identity of the value supplied"
            );
        }
        other => panic!("expected a TypeMismatch error, got {other:?}"),
    }
}

#[test]
fn test_encode_failure_names_the_codec_and_keeps_the_source_error() {
    let error = U32_DESCRIPTOR
        .encode(&u32::MAX)
        .expect_err("the fixture codec must reject u32::MAX");

    match error {
        ValueCodecExecutionError::EncodeFailed { codec_type, source } => {
            assert_eq!(
                codec_type,
                type_name::<MirrorU32StringCodec>(),
                "an encode failure must name the concrete codec that failed"
            );
            assert_eq!(
                source.to_string(),
                "mirror string encode rejected u32::MAX",
                "the typed encoder error must remain reachable through the erasure layer"
            );
        }
        other => panic!("expected an EncodeFailed error, got {other:?}"),
    }
}

#[test]
fn test_decode_failure_names_the_codec_and_keeps_the_source_error() {
    let error = U32_DESCRIPTOR
        .decode("not-a-number")
        .expect_err("the fixture codec must reject non-numeric text");

    match error {
        ValueCodecExecutionError::DecodeFailed { codec_type, source } => {
            assert_eq!(
                codec_type,
                type_name::<MirrorU32StringCodec>(),
                "a decode failure must name the concrete codec that failed"
            );
            assert_eq!(
                source.to_string(),
                "mirror string decode needs a decimal number",
                "the typed decoder error must survive erasure"
            );
        }
        other => panic!("expected a DecodeFailed error, got {other:?}"),
    }
}

#[test]
fn test_decoded_output_owns_its_data_and_outlives_the_input_buffer() {
    let decoded = {
        let input = String::from("mirror-owned-text");
        TEXT_DESCRIPTOR.decode(&input).expect("any text decodes into a String")
    };

    let owned = decoded
        .downcast_ref::<String>()
        .expect("the erased output must be downcastable to the declared value type");
    assert_eq!(
        owned, "mirror-owned-text",
        "the decoded box must own its string rather than borrow the input"
    );
}

#[test]
fn test_decoded_output_downcasts_to_the_declared_value_type_only() {
    let decoded = U32_DESCRIPTOR.decode("42").expect("decimal text decodes into a u32");

    assert!(decoded.downcast_ref::<u32>().is_some());
    assert!(
        decoded.downcast_ref::<i32>().is_none(),
        "the erased output must only downcast to the value type the descriptor declared"
    );
}

#[test]
fn test_encode_accepts_any_erased_handle_to_the_declared_type() {
    let value: Box<dyn Any> = Box::new(42_u32);
    let boxed = U32_DESCRIPTOR.encode(&*value).expect("a boxed `u32` is still a `u32`");
    let inline = U32_DESCRIPTOR.encode(&42_u32).expect("a stack `u32` is still a `u32`");

    assert_eq!(
        boxed, inline,
        "the erased entry point must behave identically for boxed and inline values"
    );
    assert_eq!(boxed, "42");
}

#[test]
fn test_each_operation_uses_a_fresh_default_codec() {
    let first = TEXT_DESCRIPTOR.encode(&String::from("mirror")).expect("encode");
    let second = TEXT_DESCRIPTOR.encode(&String::from("mirror")).expect("encode");

    assert_eq!(
        first, second,
        "the descriptor must keep no state between operations, so encoding twice must agree"
    );
    assert_eq!(first, "mirror");
}

#[test]
fn test_copying_a_descriptor_preserves_its_behaviour() {
    let copied = U32_DESCRIPTOR;

    assert_eq!(copied.codec_type_id(), U32_DESCRIPTOR.codec_type_id());
    assert_eq!(
        copied.encode(&5_u32).expect("encode"),
        U32_DESCRIPTOR.encode(&5_u32).expect("encode"),
        "a copied descriptor must stay executable"
    );
}

#[test]
fn test_encode_decode_round_trip_preserves_the_value() {
    let value = u32::from(u16::MAX);
    let encoded = U32_DESCRIPTOR.encode(&value).expect("encode");
    let decoded = U32_DESCRIPTOR
        .decode(&encoded)
        .expect("decode the text just produced")
        .downcast_ref::<u32>()
        .expect("the erased output must be the declared value type")
        .to_owned();

    assert_eq!(decoded, value, "a round trip must preserve the value");
}

#[test]
fn test_decode_of_empty_input_reports_the_codec_failure() {
    let error = U32_DESCRIPTOR
        .decode("")
        .expect_err("empty text is not a decimal number");

    assert!(
        matches!(error, ValueCodecExecutionError::DecodeFailed { .. }),
        "an empty string must surface as a decode failure, got {error:?}"
    );
}

#[test]
fn test_each_operation_uses_a_fresh_default_stateful_codec() {
    let mut direct = FreshStringCodec::default();
    assert_eq!(direct.encode(&String::from("payload")).unwrap(), "1:payload");
    assert_eq!(direct.encode(&String::from("payload")).unwrap(), "2:payload");

    let payload = String::from("payload");
    assert_eq!(FRESH_STRING.encode(&payload).unwrap(), "1:payload");
    assert_eq!(FRESH_STRING.encode(&payload).unwrap(), "1:payload");
    let decoded = FRESH_STRING.decode("payload").unwrap();
    assert_eq!(decoded.downcast_ref::<String>().unwrap(), "1:payload");
    let decoded_again = FRESH_STRING.decode("payload").unwrap();
    assert_eq!(decoded_again.downcast_ref::<String>().unwrap(), "1:payload");
    assert_eq!(FRESH_STRING.encode(&payload).unwrap(), "1:payload");
    let decoded_after_encode = FRESH_STRING.decode("payload").unwrap();
    assert_eq!(decoded_after_encode.downcast_ref::<String>().unwrap(), "1:payload");
    let copied = FRESH_STRING;
    assert_eq!(copied.encode(&payload).unwrap(), "1:payload");
    let copied_decoded = copied.decode("payload").unwrap();
    assert_eq!(copied_decoded.downcast_ref::<String>().unwrap(), "1:payload");
    let copied_decoded_again = copied.decode("payload").unwrap();
    assert_eq!(copied_decoded_again.downcast_ref::<String>().unwrap(), "1:payload");
}

#[test]
fn test_thread_safe_errors_do_not_require_thread_safe_codec_or_value() {
    let descriptor = ValueStringCodecDescriptor::of::<LocalValueCodec, Rc<u32>>();
    let value = Rc::new(7_u32);
    assert_eq!(descriptor.encode(&value).unwrap(), "7");
    let decoded = descriptor.decode("7").unwrap().downcast::<Rc<u32>>().unwrap();
    assert_eq!(**decoded, 7);
}
