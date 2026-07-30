// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared progress validation and state decisions for transcode I/O adapters.

use core::num::NonZeroUsize;
use std::io::{Error, ErrorKind, Result};

use crate::{TranscodeProgress, TranscodeStatus};

/// Next operation required after a decoder progress report.
pub(super) enum DecodeStep {
    /// The currently visible input was completely consumed.
    Complete,
    /// The decoder needs this many buffered input units before retrying.
    NeedInput(NonZeroUsize),
    /// The caller-provided output range is full.
    NeedOutput,
}

/// Validated counters and next step returned by a decoder.
pub(super) struct DecodeProgress {
    /// Number of units safe to consume from the buffered input.
    pub(super) consumed: usize,
    /// Number of values initialized in the caller output range.
    pub(super) written: usize,
    /// Required continuation after applying the counters.
    pub(super) step: DecodeStep,
}

/// Validates decoder progress and converts its status to a driver decision.
pub(super) fn decode_progress(
    progress: TranscodeProgress,
    input_index: usize,
    available_input: usize,
    output_index: usize,
    available_output: usize,
) -> Result<DecodeProgress> {
    progress
        .validate(input_index, available_input, output_index, available_output)
        .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
    let step = match progress.status() {
        TranscodeStatus::Complete => DecodeStep::Complete,
        TranscodeStatus::NeedInput { required, .. } => DecodeStep::NeedInput(required),
        TranscodeStatus::NeedOutput { .. } => DecodeStep::NeedOutput,
    };
    Ok(DecodeProgress {
        consumed: progress.read(),
        written: progress.written(),
        step,
    })
}

/// Next operation required after an encoder progress report.
pub(super) enum EncodeStep {
    /// The supplied input range was completely consumed.
    Complete,
    /// The encoder needs this much spare output capacity before retrying.
    NeedOutput(NonZeroUsize),
}

/// Validated counters and next step returned by an encoder.
pub(super) struct EncodeProgress {
    /// Number of values consumed from the caller input range.
    pub(super) read: usize,
    /// Number of initialized units in the buffered output range.
    pub(super) written: usize,
    /// Required continuation after applying the counters.
    pub(super) step: EncodeStep,
}

/// Validates encoder progress and converts its status to a driver decision.
pub(super) fn encode_progress(
    progress: TranscodeProgress,
    input_index: usize,
    available_input: usize,
    output_index: usize,
    available_output: usize,
) -> Result<EncodeProgress> {
    progress
        .validate(input_index, available_input, output_index, available_output)
        .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
    let step = match progress.status() {
        TranscodeStatus::Complete => EncodeStep::Complete,
        TranscodeStatus::NeedOutput { required, .. } => EncodeStep::NeedOutput(required),
        TranscodeStatus::NeedInput { .. } => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "encoder violated the TranscodeEncoder contract by requesting more input",
            ));
        }
    };
    Ok(EncodeProgress {
        read: progress.read(),
        written: progress.written(),
        step,
    })
}
