// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests the safe type-erased bidirectional bytes value-codec descriptor.

use std::any::Any;
use std::any::TypeId;
use std::any::type_name;
use std::io::Error;

use qubit_codec::ValueBytesCodecDescriptor;
use qubit_codec::ValueCodecExecutionError;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;

/// Bytes codec that rejects `u32::MAX` on encode and rejects short input on
/// decode, so both failure directions stay reachable from one descriptor.
#[derive(Default)]
struct MirrorU32BytesCodec;

impl ValueEncoder<u32> for MirrorU32BytesCodec {
    type Output = Vec<u8>;
    type Error = Error;

    fn encode(&mut self, input: &u32) -> Result<Self::Output, Self::Error> {
        if *input == u32::MAX {
            Err(Error::other("mirror bytes encode rejected u32::MAX"))
        } else {
            Ok(input.to_be_bytes().to_vec())
        }
    }
}

impl ValueDecoder<[u8]> for MirrorU32BytesCodec {
    type Output = u32;
    type Error = Error;

    fn decode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        let bytes: [u8; 4] = input
            .try_into()
            .map_err(|_| Error::other("mirror bytes decode needs exactly four bytes"))?;
        Ok(u32::from_be_bytes(bytes))
    }
}

/// Bytes codec for `Vec<u8>` used to check ownership of the erased output.
#[derive(Default)]
struct MirrorVecU8BytesCodec;

impl ValueEncoder<Vec<u8>> for MirrorVecU8BytesCodec {
    type Output = Vec<u8>;
    type Error = Error;

    fn encode(&mut self, input: &Vec<u8>) -> Result<Self::Output, Self::Error> {
        Ok(input.clone())
    }
}

impl ValueDecoder<[u8]> for MirrorVecU8BytesCodec {
    type Output = Vec<u8>;
    type Error = Error;

    fn decode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        Ok(input.to_vec())
    }
}

static U32_DESCRIPTOR: ValueBytesCodecDescriptor = ValueBytesCodecDescriptor::of::<MirrorU32BytesCodec, u32>();
static VEC_DESCRIPTOR: ValueBytesCodecDescriptor = ValueBytesCodecDescriptor::of::<MirrorVecU8BytesCodec, Vec<u8>>();

#[test]
fn test_metadata_matches_the_types_selected_at_construction() {
    assert_eq!(U32_DESCRIPTOR.codec_type_id(), TypeId::of::<MirrorU32BytesCodec>());
    assert_eq!(
        U32_DESCRIPTOR.codec_type_name(),
        type_name::<MirrorU32BytesCodec>(),
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
        "ValueBytesCodecDescriptor",
        "codec_type_name",
        type_name::<MirrorU32BytesCodec>(),
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
                type_name::<MirrorU32BytesCodec>(),
                "an encode failure must name the concrete codec that failed"
            );
            assert_eq!(
                source.to_string(),
                "mirror bytes encode rejected u32::MAX",
                "the typed encoder error must remain reachable through the erasure layer"
            );
        }
        other => panic!("expected an EncodeFailed error, got {other:?}"),
    }
}

#[test]
fn test_decode_failure_names_the_codec_and_keeps_the_source_error() {
    let error = U32_DESCRIPTOR
        .decode(&[0, 0])
        .expect_err("two bytes cannot be decoded as a big-endian u32");

    match error {
        ValueCodecExecutionError::DecodeFailed { codec_type, source } => {
            assert_eq!(
                codec_type,
                type_name::<MirrorU32BytesCodec>(),
                "a decode failure must name the concrete codec that failed"
            );
            assert!(
                source.to_string().contains("exactly four bytes"),
                "the typed decoder error must survive erasure, got {source}"
            );
        }
        other => panic!("expected a DecodeFailed error, got {other:?}"),
    }
}

#[test]
fn test_decoded_output_owns_its_data_and_outlives_the_input_buffer() {
    let decoded = {
        let input = vec![1_u8, 2, 3, 4];
        VEC_DESCRIPTOR.decode(&input).expect("four bytes decode into a Vec<u8>")
    };

    let owned = decoded
        .downcast_ref::<Vec<u8>>()
        .expect("the erased output must be downcastable to the declared value type");
    assert_eq!(
        owned,
        &vec![1_u8, 2, 3, 4],
        "the decoded box must own its bytes rather than borrow the input slice"
    );
}

#[test]
fn test_decoded_output_downcasts_to_the_declared_value_type_only() {
    let decoded = U32_DESCRIPTOR
        .decode(&[0, 0, 0, 42])
        .expect("four bytes decode into a u32");

    assert!(decoded.downcast_ref::<u32>().is_some());
    assert!(
        decoded.downcast_ref::<u64>().is_none(),
        "the erased output must only downcast to the value type the descriptor declared"
    );
}

#[test]
fn test_encode_accepts_any_erased_handle_to_the_declared_type() {
    let value: Box<dyn Any> = Box::new(42_u32);
    let direct = U32_DESCRIPTOR.encode(&*value).expect("a boxed `u32` is still a `u32`");
    let inline = U32_DESCRIPTOR.encode(&42_u32).expect("a stack `u32` is still a `u32`");

    assert_eq!(
        direct, inline,
        "the erased entry point must behave identically for boxed and inline values"
    );
    assert_eq!(direct, vec![0, 0, 0, 42]);
}

#[test]
fn test_each_operation_uses_a_fresh_default_codec() {
    let first = VEC_DESCRIPTOR.encode(&vec![9_u8]).expect("encode");
    let second = VEC_DESCRIPTOR.encode(&vec![9_u8]).expect("encode");

    assert_eq!(
        first, second,
        "the descriptor must keep no state between operations, so encoding twice must agree"
    );
    assert_eq!(first, vec![9_u8]);
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
    let encoded = U32_DESCRIPTOR.encode(&0x0102_0304_u32).expect("encode");
    let decoded = U32_DESCRIPTOR
        .decode(&encoded)
        .expect("decode the bytes just produced")
        .downcast_ref::<u32>()
        .expect("the erased output must be the declared value type")
        .to_owned();

    assert_eq!(decoded, 0x0102_0304, "a round trip must preserve the value");
}

#[test]
fn test_decode_of_empty_input_reports_the_codec_failure() {
    let error = U32_DESCRIPTOR
        .decode(&[])
        .expect_err("an empty buffer cannot be a big-endian u32");

    assert!(
        matches!(error, ValueCodecExecutionError::DecodeFailed { .. }),
        "empty input must surface as a decode failure, got {error:?}"
    );
}
