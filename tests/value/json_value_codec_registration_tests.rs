// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::any::TypeId;

use qubit_codec::JSON_BYTES_VALUE_CODEC_ID;
use qubit_codec::JSON_STRING_VALUE_CODEC_ID;
use qubit_codec::JsonBytesValueCodec;
use qubit_codec::JsonStringValueCodec;
use qubit_codec::ValueBytesCodecDescriptor;
use qubit_codec::ValueBytesCodecRegistry;
use qubit_codec::ValueCodecExecutionError;
use qubit_codec::ValueCodecId;
use qubit_codec::ValueStringCodecDescriptor;
use qubit_codec::ValueStringCodecRegistry;
use serde_json::json;

static STRING_DESCRIPTOR: ValueStringCodecDescriptor =
    ValueStringCodecDescriptor::of::<JsonStringValueCodec<serde_json::Value>, serde_json::Value>();
static BYTES_DESCRIPTOR: ValueBytesCodecDescriptor =
    ValueBytesCodecDescriptor::of::<JsonBytesValueCodec<serde_json::Value>, serde_json::Value>();

#[test]
fn json_value_codec_ids_match_stable_constants() {
    assert_eq!(
        ValueCodecId::new(JSON_STRING_VALUE_CODEC_ID).as_str(),
        "qubit.codec.json.string"
    );
    assert_eq!(
        ValueCodecId::new(JSON_BYTES_VALUE_CODEC_ID).as_str(),
        "qubit.codec.json.bytes"
    );
}

#[test]
fn json_string_value_codec_descriptor_metadata() {
    assert_eq!(
        STRING_DESCRIPTOR.codec_type_id(),
        TypeId::of::<JsonStringValueCodec<serde_json::Value>>()
    );
    assert_eq!(
        STRING_DESCRIPTOR.codec_type_name(),
        std::any::type_name::<JsonStringValueCodec<serde_json::Value>>()
    );
    assert_eq!(STRING_DESCRIPTOR.value_type_id(), TypeId::of::<serde_json::Value>());
    assert_eq!(
        STRING_DESCRIPTOR.value_type_name(),
        std::any::type_name::<serde_json::Value>()
    );
    assert!(format!("{STRING_DESCRIPTOR:?}").contains("JsonStringValueCodec"));
}

#[test]
fn json_bytes_value_codec_descriptor_metadata() {
    assert_eq!(
        BYTES_DESCRIPTOR.codec_type_id(),
        TypeId::of::<JsonBytesValueCodec<serde_json::Value>>()
    );
    assert_eq!(BYTES_DESCRIPTOR.value_type_id(), TypeId::of::<serde_json::Value>());
    assert!(format!("{BYTES_DESCRIPTOR:?}").contains("JsonBytesValueCodec"));
}

#[test]
fn json_string_value_codec_descriptor_executes_both_directions() {
    let encoded = STRING_DESCRIPTOR
        .encode(&json!({"ok": true}))
        .expect("encode should succeed");
    assert_eq!(encoded, r#"{"ok":true}"#);

    let decoded = STRING_DESCRIPTOR.decode(&encoded).expect("decode should succeed");
    assert_eq!(decoded.downcast_ref::<serde_json::Value>(), Some(&json!({"ok": true})));
}

#[test]
fn json_bytes_value_codec_descriptor_executes_both_directions() {
    let encoded = BYTES_DESCRIPTOR
        .encode(&json!([1, 2, 3]))
        .expect("encode should succeed");
    assert_eq!(encoded, br#"[1,2,3]"#);

    let decoded = BYTES_DESCRIPTOR.decode(&encoded).expect("decode should succeed");
    assert_eq!(decoded.downcast_ref::<serde_json::Value>(), Some(&json!([1, 2, 3])));
}

#[test]
fn json_string_value_codec_descriptor_reports_type_and_domain_errors() {
    let mismatch = STRING_DESCRIPTOR.encode(&42_u32).expect_err("wrong value type");
    assert!(matches!(mismatch, ValueCodecExecutionError::TypeMismatch { .. }));
    assert!(
        mismatch
            .to_string()
            .contains(std::any::type_name::<serde_json::Value>())
    );

    let decode = STRING_DESCRIPTOR.decode("{").expect_err("malformed JSON");
    assert!(matches!(decode, ValueCodecExecutionError::DecodeFailed { .. }));
}

#[test]
fn json_bytes_value_codec_descriptor_reports_type_and_domain_errors() {
    let mismatch = BYTES_DESCRIPTOR
        .encode(&"not-json-value")
        .expect_err("wrong value type");
    assert!(matches!(mismatch, ValueCodecExecutionError::TypeMismatch { .. }));

    let decode = BYTES_DESCRIPTOR.decode(b"[").expect_err("malformed JSON");
    assert!(matches!(decode, ValueCodecExecutionError::DecodeFailed { .. }));
}

#[test]
fn global_registries_collect_json_value_codec_registrations() {
    let string_registry = ValueStringCodecRegistry::try_global().expect("valid string registry");
    let string_registration = string_registry
        .get(JSON_STRING_VALUE_CODEC_ID)
        .expect("string JSON codec should be registered");
    assert_eq!(string_registration.id(), ValueCodecId::new(JSON_STRING_VALUE_CODEC_ID));
    assert_eq!(
        string_registration.descriptor().codec_type_id(),
        TypeId::of::<JsonStringValueCodec<serde_json::Value>>()
    );
    assert_eq!(string_registration.source().crate_name(), env!("CARGO_PKG_NAME"));

    let bytes_registry = ValueBytesCodecRegistry::try_global().expect("valid bytes registry");
    let bytes_registration = bytes_registry
        .get(JSON_BYTES_VALUE_CODEC_ID)
        .expect("bytes JSON codec should be registered");
    assert_eq!(bytes_registration.id(), ValueCodecId::new(JSON_BYTES_VALUE_CODEC_ID));
    assert_eq!(
        bytes_registration.descriptor().codec_type_id(),
        TypeId::of::<JsonBytesValueCodec<serde_json::Value>>()
    );
    assert_eq!(bytes_registration.source().crate_name(), env!("CARGO_PKG_NAME"));
}

#[test]
fn same_id_may_exist_in_string_and_bytes_json_registries() {
    let string = ValueStringCodecRegistry::global()
        .get(JSON_STRING_VALUE_CODEC_ID)
        .expect("string registration");
    let bytes = ValueBytesCodecRegistry::global()
        .get(JSON_BYTES_VALUE_CODEC_ID)
        .expect("bytes registration");

    assert_ne!(string.id(), bytes.id());
    assert_ne!(string.descriptor().codec_type_id(), bytes.descriptor().codec_type_id());
}
