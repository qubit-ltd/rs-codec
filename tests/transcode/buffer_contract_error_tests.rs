// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::BufferContractError;

#[test]
fn test_buffer_contract_error_reports_invalid_input_index() {
    let error = BufferContractError::invalid_input_index(5, 2);

    assert_eq!(
        BufferContractError::InvalidInputIndex { index: 5, len: 2 },
        error
    );
    assert_eq!(
        "invalid input index 5 for input length 2",
        error.to_string()
    );
}

#[test]
fn test_buffer_contract_error_reports_invalid_output_index() {
    let error = BufferContractError::invalid_output_index(5, 2);

    assert_eq!(
        BufferContractError::InvalidOutputIndex { index: 5, len: 2 },
        error
    );
    assert_eq!(
        "invalid output index 5 for output length 2",
        error.to_string()
    );
}

#[test]
fn test_buffer_contract_error_reports_insufficient_output() {
    let error = BufferContractError::insufficient_output(2, 4, 1);

    assert_eq!(
        BufferContractError::InsufficientOutput {
            output_index: 2,
            required: 4,
            available: 1,
        },
        error
    );
    assert!(error.to_string().contains("insufficient output"));
}

#[test]
fn test_buffer_contract_error_reports_output_length_overflow() {
    let error = BufferContractError::output_length_overflow();

    assert_eq!(BufferContractError::OutputLengthOverflow, error);
    assert_eq!("output length arithmetic overflow", error.to_string());
}

#[test]
fn test_buffer_contract_error_ensure_helpers_return_shared_error() {
    assert_eq!(
        Err(BufferContractError::invalid_input_index(4, 2)),
        BufferContractError::ensure_input_index(2, 4)
    );
    assert_eq!(
        Err(BufferContractError::invalid_output_index(4, 2)),
        BufferContractError::ensure_output_index(2, 4)
    );
    assert_eq!(
        Err(BufferContractError::insufficient_output(2, 3, 2)),
        BufferContractError::ensure_output_capacity(4, 2, 3)
    );
}
