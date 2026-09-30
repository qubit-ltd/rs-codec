// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests the bidirectional JSON string codec that combines the encoder and
//! decoder.

use qubit_codec::JsonStringValueCodec;
use qubit_codec::JsonStringValueDecoder;
use qubit_codec::JsonStringValueEncoder;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;
use serde::ser::Error as _;
use serde_json::Value as JsonValue;
use serde_json::json;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct MirrorSample {
    name: String,
    count: u32,
}

/// Serde type whose serialization always fails, to reach the encoder error
/// path.
struct MirrorFailingSerialize;

impl Serialize for MirrorFailingSerialize {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Err(S::Error::custom("mirror bidirectional encode failure"))
    }
}

#[test]
fn test_one_codec_instance_encodes_and_decodes() {
    let mut codec = JsonStringValueCodec::<MirrorSample>::new();

    let encoded = codec
        .encode(&MirrorSample {
            name: "bidirectional".to_owned(),
            count: 4,
        })
        .expect("encode should succeed through the combined codec");

    assert_eq!(encoded, r#"{"name":"bidirectional","count":4}"#);

    let decoded = codec
        .decode(&encoded)
        .expect("the same codec instance must be able to decode what it encoded");

    assert_eq!(
        decoded,
        MirrorSample {
            name: "bidirectional".to_owned(),
            count: 4,
        },
        "the combined codec must recover the value it encoded"
    );
}

#[test]
fn test_combined_codec_matches_the_dedicated_encoder_output() {
    let value = json!({"k": [1, null, false]});

    let combined = JsonStringValueCodec::<JsonValue>::new()
        .encode(&value)
        .expect("combined encode should succeed");
    let dedicated = JsonStringValueEncoder::<JsonValue>::new()
        .encode(&value)
        .expect("dedicated encode should succeed");

    assert_eq!(
        combined, dedicated,
        "the combined codec must delegate to the string encoder without altering the output"
    );
}

#[test]
fn test_combined_codec_matches_the_dedicated_decoder_output() {
    let decoded_combined = JsonStringValueCodec::<JsonValue>::new()
        .decode(r#"{"n":2.5}"#)
        .expect("combined decode should succeed");
    let decoded_dedicated = JsonStringValueDecoder::<JsonValue>::new()
        .decode(r#"{"n":2.5}"#)
        .expect("dedicated decode should succeed");

    assert_eq!(
        decoded_combined, decoded_dedicated,
        "the combined codec must delegate to the string decoder without altering the output"
    );
}

#[test]
fn test_encode_reports_a_serde_serialization_failure() {
    let error = JsonStringValueCodec::<MirrorFailingSerialize>::new()
        .encode(&MirrorFailingSerialize)
        .expect_err("a failing Serialize implementation must be reported");

    assert!(
        error.to_string().contains("mirror bidirectional encode failure"),
        "the Serde message must reach the caller, got {error}"
    );
}

#[test]
fn test_decode_reports_malformed_json() {
    let error = JsonStringValueCodec::<JsonValue>::new()
        .decode("{not-json")
        .expect_err("malformed JSON must be reported");

    assert!(
        error.is_syntax() || error.is_eof(),
        "the failure must be reported as a JSON syntax problem, got {error}"
    );
}

#[test]
fn test_decode_reports_a_missing_field_for_the_target_type() {
    let error = JsonStringValueCodec::<MirrorSample>::new()
        .decode("{}")
        .expect_err("a JSON object without the required fields must be reported");

    assert!(
        error.to_string().contains("name"),
        "the missing-field diagnostic must name the absent field, got {error}"
    );
}

#[test]
fn test_default_and_new_codecs_are_interchangeable() {
    let value = json!({"default": true});

    let from_new = JsonStringValueCodec::<JsonValue>::new()
        .encode(&value)
        .expect("new codec should encode");
    let from_default = JsonStringValueCodec::<JsonValue>::default()
        .encode(&value)
        .expect("default codec should encode");

    assert_eq!(
        from_new, from_default,
        "the codec is stateless, so `default` and `new` must agree"
    );
}

#[test]
fn test_codec_is_copy_and_debug() {
    let codec = JsonStringValueCodec::<JsonValue>::new();
    let rendered = format!("{codec:?}");
    let copied = codec;

    assert_eq!(
        rendered,
        format!("{copied:?}"),
        "a copied codec must remain identical to the original"
    );
    assert!(
        rendered.contains("JsonStringValueCodec"),
        "the Debug output must name the codec type"
    );
}

#[test]
fn test_round_trip_preserves_nested_json() {
    let value = json!({"list": [1, {"deep": [true, null]}], "text": "mirrored"});

    let mut codec = JsonStringValueCodec::<JsonValue>::new();
    let encoded = codec.encode(&value).expect("encode should succeed");
    let decoded = codec.decode(&encoded).expect("decode should succeed");

    assert_eq!(decoded, value, "a JSON round trip must be lossless");
}
