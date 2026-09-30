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
use serde_json::json;

#[derive(Debug, Serialize, PartialEq, Eq)]
struct Sample {
    name: String,
    count: u32,
}

#[test]
fn encodes_struct_to_json_bytes() {
    let value = Sample {
        name: "gamma".to_owned(),
        count: 11,
    };

    let mut encoder = JsonBytesValueEncoder::<Sample>::new();
    let encoded = encoder.encode(&value).expect("encode should succeed");

    assert_eq!(encoded, br#"{"name":"gamma","count":11}"#);
}

#[test]
fn encoded_bytes_match_string_encoder_output() {
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
fn encodes_serde_json_value() {
    let value = json!([null, 1, "x"]);

    let encoded = JsonBytesValueEncoder::<serde_json::Value>::default()
        .encode(&value)
        .expect("encode should succeed");

    assert_eq!(encoded, br#"[null,1,"x"]"#);
}

#[test]
fn dispatches_through_value_encoder_trait() {
    let value = json!({"k": "v"});

    let encoded =
        ValueEncoder::<serde_json::Value>::encode(&mut JsonBytesValueEncoder::<serde_json::Value>::new(), &value)
            .expect("trait dispatch should succeed");

    assert_eq!(encoded, br#"{"k":"v"}"#);
}
