/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Converter error and result type aliases selected by converter hooks.

use super::{
    buffered_convert_hooks::BufferedConvertHooks,
    transcode_progress::TranscodeProgress,
};

/// Converter error type selected by hooks for one target output unit type.
pub(super) type ConvertErrorOf<D, E, H, Input, Value, Output> =
    <H as BufferedConvertHooks<D, E, Input, Value, Output>>::Error;

/// Converter progress result type selected by hooks for one target output unit type.
pub(super) type ConvertProgressResult<D, E, H, Input, Value, Output> =
    Result<TranscodeProgress, ConvertErrorOf<D, E, H, Input, Value, Output>>;
