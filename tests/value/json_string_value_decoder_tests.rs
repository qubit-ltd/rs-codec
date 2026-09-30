// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests JSON-string decoding into serde values.

use qubit_codec::JsonStringValueDecoder;
use qubit_codec::ValueDecoder;
use serde::Deserialize;
use serde_json::Value;
use serde_json::json;

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Sample {
    name: String,
    count: u32,
}

#[test]
fn test_decodes_struct_from_json_string() {
    let mut decoder = JsonStringValueDecoder::<Sample>::new();
    let decoded = decoder
        .decode(r#"{"name":"beta","count":7}"#)
        .expect("decode should succeed");

    assert_eq!(
        decoded,
        Sample {
            name: "beta".to_owned(),
            count: 7,
        }
    );
}

#[test]
fn test_decodes_serde_json_value() {
    let mut decoder = JsonStringValueDecoder::<Value>::default();
    let decoded = decoder
        .decode(r#"{"ok":true,"items":[1,2]}"#)
        .expect("decode should succeed");

    assert_eq!(decoded, json!({"ok": true, "items": [1, 2]}));
}

#[test]
fn test_dispatches_through_value_decoder_trait() {
    let decoded = ValueDecoder::<str>::decode(
        &mut JsonStringValueDecoder::<Sample>::new(),
        r#"{"name":"trait","count":1}"#,
    )
    .expect("trait dispatch should succeed");

    assert_eq!(
        decoded,
        Sample {
            name: "trait".to_owned(),
            count: 1,
        }
    );
}

#[test]
fn test_rejects_malformed_json() {
    JsonStringValueDecoder::<Sample>::new()
        .decode("{not-json")
        .expect_err("malformed JSON should fail");
}
