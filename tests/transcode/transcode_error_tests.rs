// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::error::Error;
use std::io::ErrorKind;

use qubit_codec::{
    CapacityError,
    CodecPhase,
    DecodeFailure,
    TranscodeDomainError,
    TranscodeError,
    TranscodeFailure,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("domain failure")]
struct DomainError;

#[test]
fn test_transcode_error_separates_failure_and_domain_errors() {
    let failure = TranscodeError::<DomainError>::invalid_input_index(3, 1);
    assert_eq!(
        TranscodeError::Failure(TranscodeFailure::InvalidInputIndex {
            index: 3,
            input_len: 1,
        }),
        failure,
    );
    assert_eq!(
        Some(&TranscodeFailure::InvalidInputIndex {
            index: 3,
            input_len: 1,
        }),
        failure.failure_ref(),
    );
    assert_eq!(None, failure.domain_ref());

    let domain = TranscodeError::<DomainError>::domain_finish(DomainError);
    assert_eq!(
        TranscodeError::Domain(TranscodeDomainError::Finish {
            source: DomainError,
        }),
        domain,
    );
    assert_eq!(None, domain.failure_ref());
    assert_eq!(Some(&DomainError), domain.domain_ref());
    assert_eq!(
        Some(&TranscodeDomainError::Finish {
            source: DomainError,
        }),
        domain.domain_error_ref(),
    );
    assert_eq!(
        Some(TranscodeFailure::InvalidInputIndex {
            index: 3,
            input_len: 1,
        }),
        failure.failure(),
    );
    assert_eq!(None, domain.failure());
}

#[test]
fn test_transcode_error_domain_helpers() {
    let domain = TranscodeError::<&'static str>::domain_main("failure", 7);
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
        TranscodeError::<&'static str>::insufficient_output(0, 1, 0)
            .domain_ref(),
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
fn test_transcode_domain_error_accessors_describe_all_phases() {
    let reset = TranscodeDomainError::reset("reset");
    assert_eq!(&"reset", reset.source());
    assert_eq!("reset", reset.into_source());
    assert_eq!(CodecPhase::Reset, reset.phase());
    assert_eq!(None, reset.input_index());
    assert_eq!(None, reset.input_consumed());

    let main = TranscodeDomainError::main("main", 7);
    assert_eq!(&"main", main.source());
    assert_eq!("main", main.into_source());
    assert_eq!(CodecPhase::Main, main.phase());
    assert_eq!(Some(7), main.input_index());
    assert_eq!(None, main.input_consumed());

    let main_with_consumed = TranscodeDomainError::main_with_consumed(
        "invalid",
        9,
        Some(qubit_io::nz!(2)),
    );
    assert_eq!(&"invalid", main_with_consumed.source());
    assert_eq!("invalid", main_with_consumed.into_source());
    assert_eq!(CodecPhase::Main, main_with_consumed.phase());
    assert_eq!(Some(9), main_with_consumed.input_index());
    assert_eq!(Some(qubit_io::nz!(2)), main_with_consumed.input_consumed());

    let finish = TranscodeDomainError::finish("finish");
    assert_eq!(&"finish", finish.source());
    assert_eq!("finish", finish.into_source());
    assert_eq!(CodecPhase::Finish, finish.phase());
    assert_eq!(None, finish.input_index());
    assert_eq!(None, finish.input_consumed());
}

#[test]
fn test_transcode_domain_error_map_source_preserves_phase_context() {
    let reset = TranscodeDomainError::reset("reset")
        .map_source(|source| format!("mapped {source}"));
    assert_eq!(
        TranscodeDomainError::Reset {
            source: "mapped reset".to_string(),
        },
        reset,
    );

    let main = TranscodeDomainError::main_with_consumed(
        "main",
        7,
        Some(qubit_io::nz!(3)),
    )
    .map_source(|source| format!("mapped {source}"));
    assert_eq!(
        TranscodeDomainError::Main {
            source: "mapped main".to_string(),
            input_index: 7,
            input_consumed: Some(qubit_io::nz!(3)),
        },
        main,
    );

    let finish = TranscodeDomainError::finish("finish")
        .map_source(|source| format!("mapped {source}"));
    assert_eq!(
        TranscodeDomainError::Finish {
            source: "mapped finish".to_string(),
        },
        finish,
    );
}

#[test]
fn test_transcode_error_converts_capacity_error() {
    let error: TranscodeError<DomainError> =
        CapacityError::OutputLengthOverflow.into();

    assert_eq!(TranscodeError::output_length_overflow(), error);
}

#[test]
fn test_transcode_error_map_domain_preserves_framework_errors() {
    let mapped = TranscodeError::<&'static str>::invalid_input_index(3, 1)
        .map_domain(|error: &'static str| format!("mapped {error}"));
    assert_eq!(TranscodeError::<String>::invalid_input_index(3, 1), mapped);

    let mapped = TranscodeError::<&'static str>::invalid_output_index(4, 2)
        .map_domain(|error: &'static str| format!("mapped {error}"));
    assert_eq!(TranscodeError::<String>::invalid_output_index(4, 2), mapped);

    let mapped = TranscodeError::<&'static str>::insufficient_output(1, 3, 2)
        .map_domain(|error: &'static str| format!("mapped {error}"));
    assert_eq!(
        TranscodeError::<String>::insufficient_output(1, 3, 2),
        mapped,
    );

    let mapped = TranscodeError::<&'static str>::output_length_overflow()
        .map_domain(|error: &'static str| format!("mapped {error}"));
    assert_eq!(TranscodeError::<String>::output_length_overflow(), mapped);

    let mapped = TranscodeError::<&'static str>::incomplete_input(2, 4, 1)
        .map_domain(|error: &'static str| format!("mapped {error}"));
    assert_eq!(TranscodeError::<String>::incomplete_input(2, 4, 1), mapped,);

    let mapped = TranscodeError::<String>::domain_finish("inner".to_string())
        .map_domain(|error| format!("mapped {error}"));
    assert_eq!(
        TranscodeError::Domain(qubit_codec::TranscodeDomainError::Finish {
            source: "mapped inner".to_string(),
        }),
        mapped,
    );

    let mapped =
        TranscodeError::<&'static str, &'static str>::trailing_input(2, 1)
            .map_domain(|error| format!("mapped {error}"));
    assert_eq!(
        TranscodeError::<String, &'static str>::trailing_input(2, 1),
        mapped,
    );

    let mapped =
        TranscodeError::<&'static str, &'static str>::unencodable_value(4, "x")
            .map_domain(|error| format!("mapped {error}"));
    assert_eq!(
        TranscodeError::<String, &'static str>::unencodable_value(4, "x"),
        mapped,
    );
}

#[test]
fn test_transcode_error_map_failure_value_maps_framework_values() {
    let mapped =
        TranscodeError::<DomainError, &'static str>::unencodable_value(4, "x")
            .map_failure_value(|value| value.len());

    assert_eq!(TranscodeError::unencodable_value(4, 1), mapped);

    let domain = TranscodeError::<DomainError, &'static str>::domain_main(
        DomainError,
        3,
    )
    .map_failure_value(|value| value.len());

    assert_eq!(TranscodeError::domain_main(DomainError, 3), domain,);
}

#[test]
fn test_transcode_failure_map_value_preserves_all_framework_variants() {
    assert_eq!(
        TranscodeFailure::InvalidInputIndex {
            index: 1,
            input_len: 0,
        },
        TranscodeFailure::<&'static str>::InvalidInputIndex {
            index: 1,
            input_len: 0,
        }
        .map_value(str::len),
    );
    assert_eq!(
        TranscodeFailure::InvalidOutputIndex {
            index: 2,
            output_len: 1,
        },
        TranscodeFailure::<&'static str>::InvalidOutputIndex {
            index: 2,
            output_len: 1,
        }
        .map_value(str::len),
    );
    assert_eq!(
        TranscodeFailure::InsufficientOutput {
            output_index: 3,
            required: 4,
            available: 1,
        },
        TranscodeFailure::<&'static str>::InsufficientOutput {
            output_index: 3,
            required: 4,
            available: 1,
        }
        .map_value(str::len),
    );
    assert_eq!(
        TranscodeFailure::OutputLengthOverflow,
        TranscodeFailure::<&'static str>::OutputLengthOverflow
            .map_value(str::len),
    );
    assert_eq!(
        TranscodeFailure::IncompleteInput {
            input_index: 5,
            required: 6,
            available: 2,
        },
        TranscodeFailure::<&'static str>::IncompleteInput {
            input_index: 5,
            required: 6,
            available: 2,
        }
        .map_value(str::len),
    );
    assert_eq!(
        TranscodeFailure::TrailingInput {
            consumed: 7,
            remaining: 8,
        },
        TranscodeFailure::<&'static str>::TrailingInput {
            consumed: 7,
            remaining: 8,
        }
        .map_value(str::len),
    );
    assert_eq!(
        TranscodeFailure::UnencodableValue {
            input_index: 9,
            value: Some(3),
        },
        TranscodeFailure::UnencodableValue {
            input_index: 9,
            value: Some("abc"),
        }
        .map_value(str::len),
    );
    assert_eq!(
        TranscodeFailure::UnencodableValue {
            input_index: 10,
            value: None,
        },
        TranscodeFailure::<&'static str>::UnencodableValue {
            input_index: 10,
            value: None,
        }
        .map_value(str::len),
    );
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
        TranscodeError::<DomainError>::unencodable_value_without_context(9)
            .to_string(),
    );
    assert_eq!(
        "codec main error at input index 5: domain failure",
        TranscodeError::<DomainError>::domain_main(DomainError, 5).to_string(),
    );
}

#[test]
fn test_transcode_error_carries_generic_unencodable_value() {
    let error = TranscodeError::<DomainError, String>::unencodable_value(
        4,
        "hello".to_owned(),
    );

    assert_eq!(
        Some(&TranscodeFailure::UnencodableValue {
            input_index: 4,
            value: Some("hello".to_owned()),
        }),
        error.failure_ref(),
    );
}

#[test]
fn test_transcode_error_into_encode_io_error_maps_framework_variants() {
    let mut map_domain =
        |error: DomainError| std::io::Error::other(error.to_string());

    assert_eq!(
        ErrorKind::InvalidData,
        TranscodeError::<DomainError>::invalid_input_index(3, 1)
            .into_encode_io_error(&mut map_domain)
            .kind(),
    );
    assert_eq!(
        ErrorKind::InvalidData,
        TranscodeError::<DomainError>::invalid_output_index(4, 2)
            .into_encode_io_error(&mut map_domain)
            .kind(),
    );
    assert_eq!(
        ErrorKind::InvalidData,
        TranscodeError::<DomainError>::insufficient_output(1, 3, 2)
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
        TranscodeError::<DomainError>::unencodable_value_without_context(9)
            .into_encode_io_error(&mut map_domain)
            .kind(),
    );
    assert_eq!(
        "codec cannot encode value",
        TranscodeError::<DomainError>::unencodable_value_without_context(9)
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
        TranscodeError::<DomainError>::domain_main(DomainError, 5)
            .into_encode_io_error(&mut map_domain)
            .to_string(),
    );
}

#[test]
fn test_transcode_error_into_decode_io_error_maps_framework_variants() {
    let mut map_domain =
        |error: DomainError| std::io::Error::other(error.to_string());

    assert_eq!(
        ErrorKind::InvalidData,
        TranscodeError::<DomainError>::invalid_input_index(3, 1)
            .into_decode_io_error(&mut map_domain)
            .kind(),
    );
    assert_eq!(
        ErrorKind::InvalidData,
        TranscodeError::<DomainError>::invalid_output_index(4, 2)
            .into_decode_io_error(&mut map_domain)
            .kind(),
    );
    assert_eq!(
        ErrorKind::InvalidData,
        TranscodeError::<DomainError>::insufficient_output(1, 3, 2)
            .into_decode_io_error(&mut map_domain)
            .kind(),
    );
    assert_eq!(
        ErrorKind::InvalidData,
        TranscodeError::<DomainError>::output_length_overflow()
            .into_decode_io_error(&mut map_domain)
            .kind(),
    );
    assert_eq!(
        ErrorKind::InvalidData,
        TranscodeError::<DomainError>::incomplete_input(2, 4, 1)
            .into_decode_io_error(&mut map_domain)
            .kind(),
    );
    assert_eq!(
        ErrorKind::InvalidData,
        TranscodeError::<DomainError>::unencodable_value_without_context(9)
            .into_decode_io_error(&mut map_domain)
            .kind(),
    );
    assert_eq!(
        "codec cannot encode value",
        TranscodeError::<DomainError>::unencodable_value_without_context(9)
            .into_decode_io_error(&mut map_domain)
            .to_string(),
    );
    assert_eq!(
        ErrorKind::InvalidData,
        TranscodeError::<DomainError>::trailing_input(2, 1)
            .into_decode_io_error(&mut map_domain)
            .kind(),
    );
    assert_eq!(
        "trailing input: consumed 2 units, remaining 1",
        TranscodeError::<DomainError>::trailing_input(2, 1)
            .into_decode_io_error(&mut map_domain)
            .to_string(),
    );
    assert_eq!(
        "domain failure",
        TranscodeError::<DomainError>::domain_main(DomainError, 5)
            .into_decode_io_error(&mut map_domain)
            .to_string(),
    );
}

#[test]
fn test_transcode_error_source_returns_domain_error() {
    let error = TranscodeError::<DomainError>::domain_reset(DomainError);
    assert!(error.source().is_some());
    assert!(
        TranscodeError::<DomainError>::invalid_input_index(0, 0)
            .source()
            .is_none()
    );
}

#[test]
fn test_transcode_error_ensure_input_index_accepts_valid_index() {
    TranscodeError::<&'static str>::ensure_input_index(4, 2)
        .expect("valid index");
}

#[test]
fn test_transcode_error_ensure_input_index_rejects_out_of_range() {
    let error = TranscodeError::<&'static str>::ensure_input_index(2, 5)
        .expect_err("out-of-range index");

    assert_eq!(TranscodeError::invalid_input_index(5, 2), error,);
}

#[test]
fn test_transcode_error_ensure_min_input_accepts_sufficient_input() {
    TranscodeError::<&'static str>::ensure_min_input(4, 1, 2)
        .expect("sufficient input");
}

#[test]
fn test_transcode_error_ensure_min_input_delegates_to_input_index() {
    let error = TranscodeError::<&'static str>::ensure_min_input(2, 5, 0)
        .expect_err("invalid input index");

    assert_eq!(TranscodeError::invalid_input_index(5, 2), error);
}

#[test]
fn test_transcode_error_ensure_min_input_rejects_insufficient_input() {
    let error = TranscodeError::<&'static str>::ensure_min_input(4, 2, 3)
        .expect_err("insufficient input");

    assert_eq!(TranscodeError::incomplete_input(2, 3, 2), error);
}

#[test]
fn test_transcode_error_ensure_min_input_accepts_exact_minimum() {
    TranscodeError::<&'static str>::ensure_min_input(4, 1, 3)
        .expect("exact minimum input");
}

#[test]
fn test_transcode_error_ensure_min_input_accepts_zero_minimum_at_end_index() {
    TranscodeError::<&'static str>::ensure_min_input(4, 4, 0)
        .expect("zero minimum at end index");
}

#[test]
fn test_transcode_error_ensure_no_trailing_input_accepts_exact_consumption() {
    TranscodeError::<&'static str>::ensure_no_trailing_input(3, 3)
        .expect("exact consumption");
}

#[test]
fn test_transcode_error_ensure_no_trailing_input_rejects_trailing_input() {
    let error = TranscodeError::<&'static str>::ensure_no_trailing_input(2, 5)
        .expect_err("trailing input");

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
    let error =
        TranscodeError::<DomainError>::from_decode_failure(failure, 2, 1);

    assert_eq!(TranscodeError::incomplete_input(2, 4, 1), error);
    assert!(!error.is_domain());
    assert_eq!(None, error.domain_ref());
}

#[test]
fn test_transcode_error_from_decode_failure_maps_invalid_with_consumed() {
    let failure = DecodeFailure::invalid(DomainError, qubit_io::nz!(1));
    let error =
        TranscodeError::<DomainError>::from_decode_failure(failure, 5, 3);

    assert_eq!(
        TranscodeError::<DomainError>::domain_main_with_consumed(
            DomainError,
            5,
            Some(qubit_io::nz!(1)),
        ),
        error,
    );
    assert!(error.is_domain());
    assert_eq!(Some(&DomainError), error.domain_ref());
    assert_eq!(
        Some(qubit_io::nz!(1)),
        error.domain_error_ref().unwrap().input_consumed()
    );
}

#[test]
fn test_transcode_error_from_decode_failure_maps_invalid_unknown() {
    let failure = DecodeFailure::invalid_unknown(DomainError);
    let error =
        TranscodeError::<DomainError>::from_decode_failure(failure, 0, 8);

    assert_eq!(
        TranscodeError::<DomainError>::domain_main(DomainError, 0),
        error,
    );
    assert_eq!(None, error.domain_error_ref().unwrap().input_consumed());
}

#[test]
fn test_transcode_error_from_decode_failure_preserves_framework_error_through_map_domain()
 {
    let failure = DecodeFailure::incomplete(qubit_io::nz!(3));
    let mapped =
        TranscodeError::<DomainError>::from_decode_failure(failure, 1, 2)
            .map_domain(|error| format!("mapped {error:?}"));

    assert_eq!(TranscodeError::incomplete_input(1, 3, 2), mapped,);
}

#[test]
fn test_transcode_error_ensure_output_index_accepts_valid_index() {
    TranscodeError::<&'static str>::ensure_output_index(4, 4)
        .expect("valid index");
}

#[test]
fn test_transcode_error_ensure_output_index_rejects_out_of_range() {
    let error = TranscodeError::<&'static str>::ensure_output_index(1, 2)
        .expect_err("out-of-range index");

    assert_eq!(TranscodeError::invalid_output_index(2, 1), error);
}

#[test]
fn test_transcode_error_ensure_transcode_indices_accepts_valid_indices() {
    TranscodeError::<&'static str>::ensure_transcode_indices(3, 1, 5, 2)
        .expect("valid indices");
}

#[test]
fn test_transcode_error_ensure_transcode_indices_rejects_invalid_output_index()
{
    let error =
        TranscodeError::<&'static str>::ensure_transcode_indices(3, 0, 1, 2)
            .expect_err("invalid output index");

    assert_eq!(TranscodeError::invalid_output_index(2, 1), error);
}

#[test]
fn test_transcode_error_ensure_output_capacity_accepts_sufficient_capacity() {
    TranscodeError::<&'static str>::ensure_output_capacity(4, 1, 2)
        .expect("sufficient capacity");
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
    TranscodeError::<&'static str>::ensure_output_range(4, 1, 2, 2)
        .expect("valid range");
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
    let error = TranscodeError::<&'static str>::ensure_output_range(
        usize::MAX,
        usize::MAX,
        1,
        0,
    )
    .expect_err("range length overflow");

    assert_eq!(
        TranscodeError::invalid_output_index(usize::MAX, usize::MAX),
        error,
    );
}
