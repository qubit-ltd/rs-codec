// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::JsonBytesValueEncoder;
use qubit_codec::JsonStringValueEncoder;
use qubit_codec::ValueEncoder;
use serde::Serialize;
use serde::Serializer;
use serde::ser::Error as _;
use serde_json::Value as JsonValue;
use serde_json::json;

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
struct Sample {
    name: String,
    count: u32,
}

/// Fixture whose Serde serialization always fails.
struct FailingSerialize;

impl Serialize for FailingSerialize {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Err(S::Error::custom("intentional serialization failure"))
    }
}

#[test]
fn test_encodes_struct_to_json_bytes() {
    let value = Sample {
        name: "gamma".to_owned(),
        count: 11,
    };

    let mut encoder = JsonBytesValueEncoder::<Sample>::new();
    let encoded = encoder.encode(&value).expect("encode should succeed");

    assert_eq!(encoded, br#"{"name":"gamma","count":11}"#);
}

#[test]
fn test_encoded_bytes_match_string_encoder_output() {
    let value = Sample {
        name: "delta".to_owned(),
        count: 5,
    };

    let string = JsonStringValueEncoder::<Sample>::new()
        .encode(&value)
        .expect("string encode should succeed");
    let bytes = JsonBytesValueEncoder::<Sample>::new()
        .encode(&value)
        .expect("bytes encode should succeed");

    assert_eq!(string.as_bytes(), bytes.as_slice());
}

#[test]
fn test_encodes_serde_json_value() {
    let value = json!([null, 1, "x"]);

    let encoded = JsonBytesValueEncoder::<JsonValue>::default()
        .encode(&value)
        .expect("encode should succeed");

    assert_eq!(encoded, br#"[null,1,"x"]"#);
}

#[test]
fn test_dispatches_through_value_encoder_trait() {
    let value = json!({"k": "v"});

    let encoded = ValueEncoder::<JsonValue>::encode(&mut JsonBytesValueEncoder::<JsonValue>::new(), &value)
        .expect("trait dispatch should succeed");

    assert_eq!(encoded, br#"{"k":"v"}"#);
}

#[test]
fn test_reports_serialization_failure() {
    let mut encoder = JsonBytesValueEncoder::<FailingSerialize>::new();

    let error = encoder
        .encode(&FailingSerialize)
        .expect_err("serialization failure should be reported");

    assert!(error.to_string().contains("intentional serialization failure"));
}

#[test]
fn test_default_and_new_encoders_agree() {
    let value = Sample {
        name: "epsilon".to_owned(),
        count: 8,
    };

    let from_new = JsonBytesValueEncoder::<Sample>::new()
        .encode(&value)
        .expect("new encoder should succeed");
    let from_default = JsonBytesValueEncoder::<Sample>::default()
        .encode(&value)
        .expect("default encoder should succeed");

    assert_eq!(from_new, from_default);
    assert_eq!(from_default, br#"{"name":"epsilon","count":8}"#);
}
