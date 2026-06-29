// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::error::Error;
use std::io::ErrorKind;

use qubit_codec::{CapacityError, CodecPhase, DecodeFailure, TranscodeError};

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("domain failure")]
struct DomainError;

#[test]
fn test_transcode_error_domain_helpers() {
    let domain = TranscodeError::domain("failure", CodecPhase::Main, Some(7));
    assert!(domain.is_domain());
    assert_eq!(Some(&"failure"), domain.domain_ref());

    let framework = TranscodeError::<&'static str>::invalid_input_index(1, 0);
    assert!(!framework.is_domain());
    assert_eq!(None, framework.domain_ref());

    assert_eq!(
        None,
        TranscodeError::<&'static str>::invalid_output_index(1, 0).domain_ref(),
    );
    assert_eq!(
        None,
        TranscodeError::<&'static str>::insufficient_output(0, 1, 0).domain_ref(),
    );
    assert_eq!(
        None,
        TranscodeError::<&'static str>::output_length_overflow().domain_ref(),
    );
    assert_eq!(
        None,
        TranscodeError::<&'static str>::incomplete_input(2, 4, 1).domain_ref(),
    );
}

#[test]
fn test_transcode_error_converts_capacity_error() {
    let error: TranscodeError<DomainError> = CapacityError::OutputLengthOverflow.into();

    assert_eq!(TranscodeError::OutputLengthOverflow, error);
}

#[test]
fn test_transcode_error_map_domain_preserves_framework_errors() {
    let mapped = TranscodeError::invalid_input_index(3, 1)
        .map_domain(|error: &'static str| format!("mapped {error}"));
    assert_eq!(
        TranscodeError::InvalidInputIndex { index: 3, len: 1 },
        mapped
    );

    let mapped = TranscodeError::invalid_output_index(4, 2)
        .map_domain(|error: &'static str| format!("mapped {error}"));
    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 4, len: 2 },
        mapped
    );

    let mapped = TranscodeError::insufficient_output(1, 3, 2)
        .map_domain(|error: &'static str| format!("mapped {error}"));
    assert_eq!(
        TranscodeError::InsufficientOutput {
            output_index: 1,
            required: 3,
            available: 2
        },
        mapped,
    );

    let mapped = TranscodeError::<&'static str>::output_length_overflow()
        .map_domain(|error: &'static str| format!("mapped {error}"));
    assert_eq!(TranscodeError::OutputLengthOverflow, mapped);

    let mapped = TranscodeError::<&'static str>::incomplete_input(2, 4, 1)
        .map_domain(|error: &'static str| format!("mapped {error}"));
    assert_eq!(
        TranscodeError::IncompleteInput {
            input_index: 2,
            required: 4,
            available: 1,
        },
        mapped,
    );

    let mapped = TranscodeError::<String>::domain("inner".to_string(), CodecPhase::Flush, None)
        .map_domain(|error| format!("mapped {error}"));
    assert_eq!(
        TranscodeError::Domain {
            source: "mapped inner".to_string(),
            phase: CodecPhase::Flush,
            input_index: None,
        },
        mapped,
    );

    let mapped = TranscodeError::<&'static str>::trailing_input(2, 1)
        .map_domain(|error| format!("mapped {error}"));
    assert_eq!(
        TranscodeError::TrailingInput {
            consumed: 2,
            remaining: 1,
        },
        mapped,
    );

    let mapped = TranscodeError::<&'static str>::unencodable_value(4)
        .map_domain(|error| format!("mapped {error}"));
    assert_eq!(TranscodeError::UnencodableValue { input_index: 4 }, mapped,);
}

#[test]
fn test_transcode_error_display_formats_all_variants() {
    assert_eq!(
        "invalid input index 3 for input length 1",
        TranscodeError::<DomainError>::invalid_input_index(3, 1).to_string(),
    );
    assert_eq!(
        "invalid output index 4 for output length 2",
        TranscodeError::<DomainError>::invalid_output_index(4, 2).to_string(),
    );
    assert_eq!(
        "insufficient output at index 1: required 3 units, available 2",
        TranscodeError::<DomainError>::insufficient_output(1, 3, 2).to_string(),
    );
    assert_eq!(
        "output length arithmetic overflow",
        TranscodeError::<DomainError>::output_length_overflow().to_string(),
    );
    assert_eq!(
        "incomplete input at index 2: required 4 units, available 1",
        TranscodeError::<DomainError>::incomplete_input(2, 4, 1).to_string(),
    );
    assert_eq!(
        "unencodable value at input index 9",
        TranscodeError::<DomainError>::unencodable_value(9).to_string(),
    );
    assert_eq!(
        "codec Main error at input index Some(5): domain failure",
        TranscodeError::domain(DomainError, CodecPhase::Main, Some(5)).to_string(),
    );
}

#[test]
fn test_transcode_error_into_encode_io_error_maps_framework_variants() {
    let mut map_domain =
        |error: DomainError| std::io::Error::new(ErrorKind::Other, error.to_string());

    assert_eq!(
        ErrorKind::InvalidData,
        TranscodeError::invalid_input_index(3, 1)
            .into_encode_io_error(&mut map_domain)
            .kind(),
    );
    assert_eq!(
        ErrorKind::InvalidData,
        TranscodeError::invalid_output_index(4, 2)
            .into_encode_io_error(&mut map_domain)
            .kind(),
    );
    assert_eq!(
        ErrorKind::InvalidData,
        TranscodeError::insufficient_output(1, 3, 2)
            .into_encode_io_error(&mut map_domain)
            .kind(),
    );
    assert_eq!(
        ErrorKind::InvalidData,
        TranscodeError::<DomainError>::output_length_overflow()
            .into_encode_io_error(&mut map_domain)
            .kind(),
    );
    assert_eq!(
        ErrorKind::InvalidInput,
        TranscodeError::<DomainError>::unencodable_value(9)
            .into_encode_io_error(&mut map_domain)
            .kind(),
    );
    assert_eq!(
        "codec cannot encode value",
        TranscodeError::<DomainError>::unencodable_value(9)
            .into_encode_io_error(&mut map_domain)
            .to_string(),
    );
    assert_eq!(
        ErrorKind::InvalidData,
        TranscodeError::<DomainError>::incomplete_input(2, 4, 1)
            .into_encode_io_error(&mut map_domain)
            .kind(),
    );
    assert_eq!(
        ErrorKind::InvalidData,
        TranscodeError::<DomainError>::trailing_input(2, 1)
            .into_encode_io_error(&mut map_domain)
            .kind(),
    );
    assert_eq!(
        "domain failure",
        TranscodeError::domain(DomainError, CodecPhase::Main, Some(5))
            .into_encode_io_error(&mut map_domain)
            .to_string(),
    );
}

#[test]
fn test_transcode_error_source_returns_domain_error() {
    let error = TranscodeError::domain(DomainError, CodecPhase::Reset, None);
    assert!(error.source().is_some());
    assert!(
        TranscodeError::<DomainError>::invalid_input_index(0, 0)
            .source()
            .is_none()
    );
}

#[test]
fn test_transcode_error_ensure_input_index_accepts_valid_index() {
    TranscodeError::<&'static str>::ensure_input_index(4, 2).expect("valid index");
}

#[test]
fn test_transcode_error_ensure_input_index_rejects_out_of_range() {
    let error =
        TranscodeError::<&'static str>::ensure_input_index(2, 5).expect_err("out-of-range index");

    assert_eq!(TranscodeError::invalid_input_index(5, 2), error,);
}

#[test]
fn test_transcode_error_ensure_min_input_accepts_sufficient_input() {
    TranscodeError::<&'static str>::ensure_min_input(4, 1, 2).expect("sufficient input");
}

#[test]
fn test_transcode_error_ensure_min_input_delegates_to_input_index() {
    let error =
        TranscodeError::<&'static str>::ensure_min_input(2, 5, 0).expect_err("invalid input index");

    assert_eq!(TranscodeError::invalid_input_index(5, 2), error);
}

#[test]
fn test_transcode_error_ensure_min_input_rejects_insufficient_input() {
    let error =
        TranscodeError::<&'static str>::ensure_min_input(4, 2, 3).expect_err("insufficient input");

    assert_eq!(TranscodeError::incomplete_input(2, 3, 2), error);
}

#[test]
fn test_transcode_error_ensure_min_input_accepts_exact_minimum() {
    TranscodeError::<&'static str>::ensure_min_input(4, 1, 3).expect("exact minimum input");
}

#[test]
fn test_transcode_error_ensure_min_input_accepts_zero_minimum_at_end_index() {
    TranscodeError::<&'static str>::ensure_min_input(4, 4, 0).expect("zero minimum at end index");
}

#[test]
fn test_transcode_error_ensure_no_trailing_input_accepts_exact_consumption() {
    TranscodeError::<&'static str>::ensure_no_trailing_input(3, 3).expect("exact consumption");
}

#[test]
fn test_transcode_error_ensure_no_trailing_input_rejects_trailing_input() {
    let error =
        TranscodeError::<&'static str>::ensure_no_trailing_input(2, 5).expect_err("trailing input");

    assert_eq!(TranscodeError::trailing_input(2, 3), error);
}

#[test]
fn test_transcode_error_ensure_no_trailing_input_rejects_unconsumed_prefix() {
    let error = TranscodeError::<&'static str>::ensure_no_trailing_input(0, 2)
        .expect_err("unconsumed prefix");

    assert_eq!(TranscodeError::trailing_input(0, 2), error);
}

#[test]
fn test_transcode_error_from_decode_failure_maps_incomplete() {
    let failure = DecodeFailure::incomplete(qubit_io::nz!(4));
    let error = TranscodeError::<DomainError>::from_decode_failure(failure, 2, 1);

    assert_eq!(TranscodeError::incomplete_input(2, 4, 1), error);
    assert!(!error.is_domain());
    assert_eq!(None, error.domain_ref());
}

#[test]
fn test_transcode_error_from_decode_failure_maps_invalid_with_consumed() {
    let failure = DecodeFailure::invalid(DomainError, qubit_io::nz!(1));
    let error = TranscodeError::<DomainError>::from_decode_failure(failure, 5, 3);

    assert_eq!(
        TranscodeError::domain(DomainError, CodecPhase::Main, Some(5)),
        error,
    );
    assert!(error.is_domain());
    assert_eq!(Some(&DomainError), error.domain_ref());
}

#[test]
fn test_transcode_error_from_decode_failure_maps_invalid_without_consumed() {
    let failure = DecodeFailure::invalid_without_consumed(DomainError);
    let error = TranscodeError::<DomainError>::from_decode_failure(failure, 0, 8);

    assert_eq!(
        TranscodeError::domain(DomainError, CodecPhase::Main, Some(0)),
        error,
    );
}

#[test]
fn test_transcode_error_from_decode_failure_preserves_framework_error_through_map_domain() {
    let failure = DecodeFailure::incomplete(qubit_io::nz!(3));
    let mapped = TranscodeError::<DomainError>::from_decode_failure(failure, 1, 2)
        .map_domain(|error| format!("mapped {error:?}"));

    assert_eq!(
        TranscodeError::IncompleteInput {
            input_index: 1,
            required: 3,
            available: 2,
        },
        mapped,
    );
}

#[test]
fn test_transcode_error_ensure_output_index_accepts_valid_index() {
    TranscodeError::<&'static str>::ensure_output_index(4, 4).expect("valid index");
}

#[test]
fn test_transcode_error_ensure_output_index_rejects_out_of_range() {
    let error =
        TranscodeError::<&'static str>::ensure_output_index(1, 2).expect_err("out-of-range index");

    assert_eq!(TranscodeError::invalid_output_index(2, 1), error);
}

#[test]
fn test_transcode_error_ensure_transcode_indices_accepts_valid_indices() {
    TranscodeError::<&'static str>::ensure_transcode_indices(3, 1, 5, 2).expect("valid indices");
}

#[test]
fn test_transcode_error_ensure_transcode_indices_rejects_invalid_output_index() {
    let error = TranscodeError::<&'static str>::ensure_transcode_indices(3, 0, 1, 2)
        .expect_err("invalid output index");

    assert_eq!(TranscodeError::invalid_output_index(2, 1), error);
}

#[test]
fn test_transcode_error_ensure_output_capacity_accepts_sufficient_capacity() {
    TranscodeError::<&'static str>::ensure_output_capacity(4, 1, 2).expect("sufficient capacity");
}

#[test]
fn test_transcode_error_ensure_output_capacity_delegates_to_output_index() {
    let error = TranscodeError::<&'static str>::ensure_output_capacity(2, 5, 0)
        .expect_err("invalid output index");

    assert_eq!(TranscodeError::invalid_output_index(5, 2), error,);
}

#[test]
fn test_transcode_error_ensure_output_capacity_rejects_insufficient_capacity() {
    let error = TranscodeError::<&'static str>::ensure_output_capacity(4, 2, 3)
        .expect_err("insufficient capacity");

    assert_eq!(TranscodeError::insufficient_output(2, 3, 2), error);
}

#[test]
fn test_transcode_error_ensure_output_range_accepts_valid_range() {
    TranscodeError::<&'static str>::ensure_output_range(4, 1, 2, 2).expect("valid range");
}

#[test]
fn test_transcode_error_ensure_output_range_rejects_insufficient_range() {
    let error = TranscodeError::<&'static str>::ensure_output_range(4, 1, 1, 2)
        .expect_err("insufficient range");

    assert_eq!(TranscodeError::insufficient_output(1, 2, 1), error,);
}

#[test]
fn test_transcode_error_ensure_output_range_rejects_overflowing_range() {
    let error = TranscodeError::<&'static str>::ensure_output_range(4, 3, 2, 0)
        .expect_err("overflowing range");

    assert_eq!(TranscodeError::invalid_output_index(3, 4), error,);
}

#[test]
fn test_transcode_error_ensure_output_range_rejects_invalid_output_index() {
    let error = TranscodeError::<&'static str>::ensure_output_range(4, 5, 0, 0)
        .expect_err("invalid output index");

    assert_eq!(TranscodeError::invalid_output_index(5, 4), error);
}

#[test]
fn test_transcode_error_ensure_output_range_rejects_range_length_overflow() {
    let error = TranscodeError::<&'static str>::ensure_output_range(usize::MAX, usize::MAX, 1, 0)
        .expect_err("range length overflow");

    assert_eq!(
        TranscodeError::invalid_output_index(usize::MAX, usize::MAX),
        error,
    );
}
