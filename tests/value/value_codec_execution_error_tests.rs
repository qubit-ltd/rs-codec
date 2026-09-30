// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests the type-erased execution errors raised by value-codec descriptors.

use std::any::TypeId;
use std::error::Error;
use std::io::Error as IoError;

use qubit_codec::ValueCodecExecutionError;

#[test]
fn test_type_mismatch_message_names_both_the_expected_and_actual_type() {
    let error = ValueCodecExecutionError::TypeMismatch {
        expected_type: "u32",
        actual_type: TypeId::of::<String>(),
    };

    assert_eq!(
        error.to_string(),
        format!(
            "value codec for u32 received incompatible type {:?}",
            TypeId::of::<String>()
        ),
        "a mismatch must report both the declared value type and the actual type identity"
    );
}

#[test]
fn test_type_mismatch_fields_survive_construction() {
    let error = ValueCodecExecutionError::TypeMismatch {
        expected_type: "mirror.Type",
        actual_type: TypeId::of::<u64>(),
    };

    match error {
        ValueCodecExecutionError::TypeMismatch {
            expected_type,
            actual_type,
        } => {
            assert_eq!(expected_type, "mirror.Type");
            assert_eq!(actual_type, TypeId::of::<u64>());
        }
        other => panic!("expected a TypeMismatch error, got {other:?}"),
    }
}

#[test]
fn test_encode_failure_message_and_source_chain_report_the_typed_error() {
    let source = IoError::other("mirror encode failure");
    let error = ValueCodecExecutionError::EncodeFailed {
        codec_type: "mirror_crate::MirrorCodec",
        source: Box::new(source),
    };

    assert_eq!(
        error.to_string(),
        "value codec mirror_crate::MirrorCodec failed to encode: mirror encode failure",
        "an encode failure must name the codec and forward the source message"
    );

    let reported = error.source().expect("an encode failure must expose its source");
    assert_eq!(
        reported.to_string(),
        "mirror encode failure",
        "the source chain must reach the typed encoder error"
    );
}

#[test]
fn test_decode_failure_message_and_source_chain_report_the_typed_error() {
    let source = IoError::other("mirror decode failure");
    let error = ValueCodecExecutionError::DecodeFailed {
        codec_type: "mirror_crate::MirrorCodec",
        source: Box::new(source),
    };

    assert_eq!(
        error.to_string(),
        "value codec mirror_crate::MirrorCodec failed to decode: mirror decode failure",
        "a decode failure must name the codec and forward the source message"
    );

    let reported = error.source().expect("a decode failure must expose its source");
    assert_eq!(
        reported.to_string(),
        "mirror decode failure",
        "the source chain must reach the typed decoder error"
    );
}

#[test]
fn test_encode_and_decode_failures_stay_distinguishable_by_message() {
    let encode = ValueCodecExecutionError::EncodeFailed {
        codec_type: "mirror_crate::MirrorCodec",
        source: Box::new(IoError::other("boom")),
    };
    let decode = ValueCodecExecutionError::DecodeFailed {
        codec_type: "mirror_crate::MirrorCodec",
        source: Box::new(IoError::other("boom")),
    };

    assert_ne!(
        encode.to_string(),
        decode.to_string(),
        "the direction of the failure must remain visible in diagnostics"
    );
}

#[test]
fn test_type_mismatch_has_no_further_source_error() {
    let error = ValueCodecExecutionError::TypeMismatch {
        expected_type: "u32",
        actual_type: TypeId::of::<u8>(),
    };

    assert!(
        error.source().is_none(),
        "a type mismatch originates in the registry layer and has no source error"
    );
}

#[test]
fn test_debug_output_names_the_failed_direction() {
    let encode = ValueCodecExecutionError::EncodeFailed {
        codec_type: "mirror_crate::MirrorCodec",
        source: Box::new(IoError::other("boom")),
    };

    assert!(
        format!("{encode:?}").starts_with("EncodeFailed"),
        "the derived Debug output must name the failed direction"
    );
}

#[test]
fn test_execution_error_can_be_raised_through_the_standard_error_trait() {
    fn failing() -> Result<(), ValueCodecExecutionError> {
        Err(ValueCodecExecutionError::TypeMismatch {
            expected_type: "mirror.Type",
            actual_type: TypeId::of::<u8>(),
        })
    }

    let boxed: Box<dyn Error> = Box::new(failing().expect_err("the error must be reported"));

    assert!(
        boxed
            .to_string()
            .starts_with("value codec for mirror.Type received incompatible type"),
        "a boxed execution error must keep its diagnostic, rendered as {boxed}",
    );
}
