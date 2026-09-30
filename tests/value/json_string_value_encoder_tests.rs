// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::JsonStringValueEncoder;
use qubit_codec::ValueEncoder;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;

#[derive(Debug, Serialize, PartialEq, Eq)]
struct Sample {
    name: String,
    count: u32,
}

#[test]
fn test_encodes_struct_to_json_string() {
    let value = Sample {
        name: "alpha".to_owned(),
        count: 3,
    };

    let mut encoder = JsonStringValueEncoder::<Sample>::new();
    let encoded = encoder.encode(&value).expect("encode should succeed");

    assert_eq!(encoded, r#"{"name":"alpha","count":3}"#);
}

#[test]
fn test_encodes_serde_json_value() {
    let value = json!({"nested": [1, null, false]});

    let encoded = JsonStringValueEncoder::<Value>::default()
        .encode(&value)
        .expect("encode should succeed");

    assert_eq!(encoded, r#"{"nested":[1,null,false]}"#);
}

#[test]
fn test_dispatches_through_value_encoder_trait() {
    let value = Sample {
        name: "trait".to_owned(),
        count: 9,
    };

    let encoded = ValueEncoder::<Sample>::encode(&mut JsonStringValueEncoder::<Sample>::new(), &value)
        .expect("trait dispatch should succeed");

    assert_eq!(encoded, r#"{"name":"trait","count":9}"#);
}

#[test]
fn test_default_constructor_matches_new() {
    let value = json!(42);

    let from_new = JsonStringValueEncoder::<Value>::new()
        .encode(&value)
        .expect("new encoder should succeed");
    let from_default = JsonStringValueEncoder::<Value>::default()
        .encode(&value)
        .expect("default encoder should succeed");

    assert_eq!(from_new, from_default);
}
