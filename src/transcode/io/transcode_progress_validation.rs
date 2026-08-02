// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Validation shared by transcode I/O adapters.

use std::io::{
    Error,
    ErrorKind,
    Result,
};

use crate::{
    TranscodeProgress,
    TranscodeStatus,
};

/// Validates a decoder progress report before the input adapter commits it.
pub(super) fn validate_decode_progress(
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

/// Validates an encoder progress report before the output adapter commits it.
pub(super) fn validate_encode_progress(
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
