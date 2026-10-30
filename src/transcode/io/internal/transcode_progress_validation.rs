// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Validation shared by transcode I/O adapters.

use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;

use crate::TranscodeProgress;
use crate::TranscodeStatus;

/// Validates a decoder progress report before the input adapter commits it.
///
/// # Parameters
///
/// - `progress`: Report returned by the transcoder for the current call.
/// - `input_index`: Absolute input index at the start of that call.
/// - `available_input`: Input units available from the starting index.
/// - `output_index`: Absolute output index at the start of that call.
/// - `available_output`: Writable units available from the starting index.
///
/// # Returns
///
/// Returns the unchanged report when its counts and status satisfy the
/// contract.
///
/// # Errors
///
/// Returns [`ErrorKind::InvalidData`] if the report violates progress bounds
/// or status invariants. No adapter state is changed by this validation.
pub(in crate::transcode::io) fn validate_decode_progress(
    progress: TranscodeProgress,
    input_index: usize,
    available_input: usize,
    output_index: usize,
    available_output: usize,
) -> Result<TranscodeProgress> {
    progress
        .validate(input_index, available_input, output_index, available_output)
        .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
    Ok(progress)
}

/// Validates EOF progress before the input adapter commits source consumption.
///
/// # Parameters
///
/// - `progress`: Report returned by the transcoder for the EOF call.
/// - `input_index`: Absolute input index at the start of that call.
/// - `available_input`: Input units available from the starting index.
/// - `output_index`: Absolute output index at the start of that call.
/// - `available_output`: Writable units available from the starting index.
///
/// # Returns
///
/// Returns the unchanged report when its counts and status satisfy the EOF
/// contract.
///
/// # Errors
///
/// Returns `InvalidData` for invalid counts, status invariants, or `NeedInput`.
/// The adapter has not committed source consumption, but this check does not
/// roll back decoder state or caller-owned output.
pub(in crate::transcode::io) fn validate_eof_decode_progress(
    progress: TranscodeProgress,
    input_index: usize,
    available_input: usize,
    output_index: usize,
    available_output: usize,
) -> Result<TranscodeProgress> {
    let progress = validate_decode_progress(progress, input_index, available_input, output_index, available_output)?;
    if matches!(progress.status(), TranscodeStatus::NeedInput { .. }) {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "decoder violated the EOF contract by requesting more input",
        ));
    }
    Ok(progress)
}

/// Validates an encoder progress report before the output adapter commits it.
///
/// # Parameters
///
/// - `progress`: Report returned by the transcoder for the current call.
/// - `input_index`: Absolute input index at the start of that call.
/// - `available_input`: Input units available from the starting index.
/// - `output_index`: Absolute output index at the start of that call.
/// - `available_output`: Writable units available from the starting index.
///
/// # Returns
///
/// Returns the unchanged report when its counts and status satisfy the
/// contract.
///
/// # Errors
///
/// Returns [`ErrorKind::InvalidData`] if the report violates progress bounds
/// or status invariants, or requests more input. This changes no adapter state.
pub(in crate::transcode::io) fn validate_encode_progress(
    progress: TranscodeProgress,
    input_index: usize,
    available_input: usize,
    output_index: usize,
    available_output: usize,
) -> Result<TranscodeProgress> {
    progress
        .validate(input_index, available_input, output_index, available_output)
        .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
    if matches!(progress.status(), TranscodeStatus::NeedInput { .. }) {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "encoder violated the TranscodeEncoder contract by requesting more input",
        ));
    }
    Ok(progress)
}
