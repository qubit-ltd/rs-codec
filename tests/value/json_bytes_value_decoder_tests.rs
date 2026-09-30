// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests JSON-bytes decoding into serde values.

use qubit_codec::JsonBytesValueDecoder;
use qubit_codec::JsonBytesValueEncoder;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct Sample {
    name: String,
    count: u32,
}

#[test]
fn test_decodes_struct_from_json_bytes() {
    let mut decoder = JsonBytesValueDecoder::<Sample>::new();
    let decoded = decoder
        .decode(br#"{"name":"epsilon","count":13}"#)
        .expect("decode should succeed");

    assert_eq!(
        decoded,
        Sample {
            name: "epsilon".to_owned(),
            count: 13,
        }
    );
}

#[test]
fn test_round_trips_with_bytes_encoder() {
    let value = Sample {
        name: "round".to_owned(),
        count: 2,
    };

    let encoded = JsonBytesValueEncoder::<Sample>::new()
        .encode(&value)
        .expect("encode should succeed");
    let decoded = JsonBytesValueDecoder::<Sample>::new()
        .decode(&encoded)
        .expect("decode should succeed");

    assert_eq!(decoded, value);
}

#[test]
fn test_decodes_serde_json_value() {
    let mut decoder = JsonBytesValueDecoder::<Value>::default();
    let decoded = decoder
        .decode(br#"{"n":3.5,"s":"text"}"#)
        .expect("decode should succeed");

    assert_eq!(decoded, json!({"n": 3.5, "s": "text"}));
}

#[test]
fn test_dispatches_through_value_decoder_trait() {
    let decoded = ValueDecoder::<[u8]>::decode(&mut JsonBytesValueDecoder::<Value>::new(), br#""hello""#)
        .expect("trait dispatch should succeed");

    assert_eq!(decoded, json!("hello"));
}

#[test]
fn test_rejects_malformed_json_bytes() {
    JsonBytesValueDecoder::<Sample>::new()
        .decode(b"{not-json")
        .expect_err("malformed JSON should fail");
}
