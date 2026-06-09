// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Result type for converter steps that may stop with public progress.

use super::{convert_error_of::ConvertErrorOf, transcode_progress::TranscodeProgress};

/// Result type for converter steps that may stop with public progress.
///
/// # Type Parameters
///
/// - `D`: Source codec.
/// - `E`: Target codec.
/// - `H`: Converter hook set.
///
/// # Returns
///
/// Returns:
/// - `Ok(Some(progress))` when the converter should propagate public progress,
/// - `Ok(None)` when the current call can continue,
/// - `Err(...)` when an error halts conversion.
pub(super) type ConvertStepResult<D, E, H> =
    Result<Option<TranscodeProgress>, ConvertErrorOf<D, E, H>>;
