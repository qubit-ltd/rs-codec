// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Converter progress result type alias selected by converter hooks.

use super::convert_error_of::ConvertErrorOf;
use crate::TranscodeProgress;

/// Converter progress result type selected by hooks for one target output unit
/// type.
///
/// # Type Parameters
///
/// - `D`: Source codec type.
/// - `E`: Target codec type.
/// - `H`: Converter hook type exposing `Error`.
///
/// # Returns
///
/// Returns a [`Result`] carrying:
/// - `Ok(progress)` when a conversion step advances or completes without error,
///   or
/// - `Err(error)` when conversion cannot continue.
pub(in crate::transcode) type ConvertProgressResult<D, E, H> =
    Result<TranscodeProgress, ConvertErrorOf<D, E, H>>;
