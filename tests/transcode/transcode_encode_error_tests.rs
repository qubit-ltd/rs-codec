// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{TranscodeDomainError, TranscodeEncodeError, TranscodeFailure};

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("domain error")]
struct DomainError;

#[test]
fn test_encode_error_carries_unencodable_value_context() {
    let error = TranscodeEncodeError::<DomainError, char>::unencodable(4, 'x');
    assert_eq!(Some((4, Some(&'x'))), error.unencodable_ref());

    let mapped = error.map_value(|value| value as u32);
    assert_eq!(
        TranscodeEncodeError::<DomainError, u32>::unencodable(4, 'x' as u32),
        mapped,
    );
}

#[test]
fn test_encode_error_accessors_mapping_and_validation() {
    let failure = TranscodeEncodeError::<&str, char>::incomplete_input(1, 2, 0);
    let trailing = TranscodeEncodeError::<&str, char>::trailing_input(1, 1);
    let domain = TranscodeEncodeError::<&str, char>::domain_reset("reset");
    let main = TranscodeEncodeError::<&str, char>::domain_main("encode", 3);
    let finish = TranscodeEncodeError::<&str, char>::domain_finish("finish");
    let no_context = TranscodeEncodeError::<&str, char>::unencodable_without_context(8);

    assert!(!failure.is_domain());
    assert!(domain.is_domain());
    assert!(matches!(
        trailing.failure_ref(),
        Some(TranscodeFailure::TrailingInput { .. })
    ));
    assert_eq!(None, domain.failure_ref());
    assert_eq!(None, failure.domain_error_ref());
    assert_eq!(
        Some(&TranscodeDomainError::Reset { source: "reset" }),
        domain.domain_error_ref(),
    );
    assert_eq!(Some(&"reset"), domain.domain_ref());
    assert_eq!(None, failure.domain_ref());
    assert_eq!(None, failure.unencodable_ref());
    assert_eq!(Some((8, None)), no_context.unencodable_ref());

    assert_eq!(
        TranscodeEncodeError::<usize, char>::incomplete_input(1, 2, 0),
        failure.map_domain(str::len),
    );
    assert_eq!(
        TranscodeEncodeError::<usize, char>::unencodable_without_context(8),
        no_context.map_domain(str::len),
    );
    assert_eq!(
        TranscodeEncodeError::<usize, char>::domain_main(6, 3),
        main.map_domain(str::len),
    );
    assert_eq!(
        TranscodeEncodeError::<&str, u32>::domain_finish("finish"),
        finish.map_value(|value: char| value as u32),
    );
    assert_eq!(
        TranscodeEncodeError::<&str, u32>::trailing_input(1, 1),
        TranscodeEncodeError::<&str, char>::trailing_input(1, 1)
            .map_value(|value: char| value as u32),
    );

    assert_eq!(
        Ok(()),
        TranscodeEncodeError::<&str, char>::ensure_output_index(2, 2)
    );
    assert!(matches!(
        TranscodeEncodeError::<&str, char>::ensure_output_index(2, 3),
        Err(TranscodeEncodeError::Failure(
            TranscodeFailure::InvalidOutputIndex { .. }
        ))
    ));
    assert_eq!(
        Ok(()),
        TranscodeEncodeError::<&str, char>::ensure_transcode_indices(2, 1, 2, 1),
    );
    assert!(matches!(
        TranscodeEncodeError::<&str, char>::ensure_transcode_indices(2, 3, 2, 1),
        Err(TranscodeEncodeError::Failure(
            TranscodeFailure::InvalidInputIndex { .. }
        ))
    ));
    assert_eq!(
        Ok(()),
        TranscodeEncodeError::<&str, char>::ensure_output_capacity(3, 1, 2),
    );
    assert!(matches!(
        TranscodeEncodeError::<&str, char>::ensure_output_capacity(3, 1, 3),
        Err(TranscodeEncodeError::Failure(
            TranscodeFailure::InsufficientOutput { .. }
        ))
    ));
}
