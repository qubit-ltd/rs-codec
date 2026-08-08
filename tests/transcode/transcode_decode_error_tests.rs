// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec as codec;
use qubit_codec::DecodeFailure;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeDomainError;
use qubit_codec::TranscodeFailure;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("domain error")]
struct DomainError;

#[test]
fn test_decode_error_wraps_framework_and_domain_errors() {
    let failure = TranscodeDecodeError::<DomainError>::Failure(
        TranscodeFailure::invalid_input_index(3, 1),
    );
    assert_eq!(
        TranscodeDecodeError::Failure(TranscodeFailure::InvalidInputIndex {
            index: 3,
            input_len: 1,
        }),
        failure,
    );

    let domain =
        TranscodeDecodeError::<DomainError>::domain_finish(DomainError);
    assert_eq!(Some(&DomainError), domain.domain_ref());
    assert_eq!(
        Some(&TranscodeDomainError::Finish {
            source: DomainError,
        }),
        domain.domain_error_ref(),
    );
}

#[test]
fn test_decode_error_maps_decode_failure() {
    let incomplete =
        DecodeFailure::<DomainError>::incomplete(crate::nonzero(4));
    assert_eq!(
        codec::TranscodeDecodeError::Failure(
            codec::TranscodeFailure::incomplete_input(2, 4, 1)
        ),
        TranscodeDecodeError::from_decode_failure(incomplete, 2, 1),
    );

    let invalid = DecodeFailure::invalid(DomainError, crate::nonzero(1));
    assert_eq!(
        TranscodeDecodeError::domain_main_with_consumed(
            DomainError,
            5,
            Some(crate::nonzero(1)),
        ),
        TranscodeDecodeError::from_decode_failure(invalid, 5, 3),
    );
}

#[test]
fn test_decode_error_preserves_incomplete_domain_source_at_eof() {
    let incomplete =
        DecodeFailure::incomplete_with_source(DomainError, crate::nonzero(4));

    assert_eq!(
        TranscodeDecodeError::domain_main(DomainError, 2),
        TranscodeDecodeError::from_decode_failure(incomplete, 2, 1),
    );
}

#[test]
fn test_decode_error_accessors_mapping_and_validation() {
    let failure = TranscodeDecodeError::Failure(
        TranscodeFailure::invalid_output_index(3, 1),
    );
    let domain = TranscodeDecodeError::<&str>::domain_main("decode", 2);

    assert!(!failure.is_domain());
    assert!(domain.is_domain());
    assert_eq!(
        Some(&TranscodeFailure::InvalidOutputIndex {
            index: 3,
            output_len: 1,
        }),
        failure.failure_ref(),
    );
    assert_eq!(None, domain.failure_ref());
    assert_eq!(None, failure.domain_error_ref());
    assert_eq!(
        Some(&TranscodeDomainError::Main {
            source: "decode",
            input_index: 2,
            input_consumed: None,
        }),
        domain.domain_error_ref(),
    );
    assert_eq!(None, failure.domain_ref());

    assert_eq!(
        TranscodeDecodeError::Failure(TranscodeFailure::invalid_output_index(
            3, 1
        )),
        failure.map_domain(str::len),
    );
    assert_eq!(
        TranscodeDecodeError::<usize>::domain_main(6, 2),
        domain.map_domain(str::len),
    );

    assert_eq!(
        Ok::<(), TranscodeDecodeError<&str>>(()),
        TranscodeFailure::ensure_input_index(2, 2)
            .map_err(TranscodeDecodeError::<&str>::from)
    );
    assert!(matches!(
        TranscodeFailure::ensure_input_index(2, 3)
            .map_err(TranscodeDecodeError::<&str>::from),
        Err(TranscodeDecodeError::Failure(
            TranscodeFailure::InvalidInputIndex { .. }
        ))
    ));
    assert_eq!(
        Ok::<(), TranscodeDecodeError<&str>>(()),
        TranscodeFailure::ensure_min_input(3, 1, 2)
            .map_err(TranscodeDecodeError::<&str>::from)
    );
    assert!(matches!(
        TranscodeFailure::ensure_min_input(3, 2, 2)
            .map_err(TranscodeDecodeError::<&str>::from),
        Err(TranscodeDecodeError::Failure(
            TranscodeFailure::IncompleteInput { .. }
        ))
    ));
    assert_eq!(
        Ok::<(), TranscodeDecodeError<&str>>(()),
        TranscodeFailure::ensure_no_trailing_input(2, 2)
            .map_err(TranscodeDecodeError::<&str>::from),
    );
    assert!(matches!(
        TranscodeFailure::ensure_no_trailing_input(1, 2)
            .map_err(TranscodeDecodeError::<&str>::from),
        Err(TranscodeDecodeError::Failure(
            TranscodeFailure::TrailingInput { .. }
        ))
    ));
    assert_eq!(
        Err(TranscodeDecodeError::Failure(
            TranscodeFailure::InvalidInputIndex {
                index: 3,
                input_len: 2,
            },
        )),
        TranscodeFailure::ensure_no_trailing_input(3, 2)
            .map_err(TranscodeDecodeError::<&str>::from),
    );
    assert_eq!(
        Ok::<(), TranscodeDecodeError<&str>>(()),
        TranscodeFailure::ensure_output_range(4, 1, 2, 2)
            .map_err(TranscodeDecodeError::<&str>::from)
    );
    assert!(matches!(
        TranscodeFailure::ensure_output_range(4, 1, 1, 2)
            .map_err(TranscodeDecodeError::<&str>::from),
        Err(TranscodeDecodeError::Failure(
            TranscodeFailure::InsufficientOutput { .. }
        ))
    ));
}
