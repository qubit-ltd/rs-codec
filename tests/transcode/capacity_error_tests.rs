// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec as codec;
use qubit_codec::CapacityError;
use qubit_codec::TranscodeConvertError;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeEncodeError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("domain error")]
struct DomainError;

#[test]
fn test_capacity_errors_map_to_framework_failures() {
    let decode: TranscodeDecodeError<DomainError> = CapacityError::OutputLengthOverflow.into();
    let encode: TranscodeEncodeError<DomainError, char> = CapacityError::OutputLengthOverflow.into();
    let convert: TranscodeConvertError<DomainError, DomainError, char> = CapacityError::OutputLengthOverflow.into();

    assert_eq!(
        TranscodeDecodeError::Failure(codec::TranscodeFailure::output_length_overflow()),
        decode,
    );
    assert_eq!(
        TranscodeEncodeError::Failure(codec::TranscodeFailure::output_length_overflow()),
        encode,
    );
    assert_eq!(
        TranscodeConvertError::Failure(codec::TranscodeFailure::output_length_overflow()),
        convert,
    );
}
