/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Result type for converter steps that may stop with public progress.

use super::{
    convert_error_of::ConvertErrorOf,
    transcode_progress::TranscodeProgress,
};

/// Result type for converter steps that may stop with public progress.
pub(super) type ConvertStepResult<D, E, H, Input, Value, Output> =
    Result<Option<TranscodeProgress>, ConvertErrorOf<D, E, H, Input, Value, Output>>;
