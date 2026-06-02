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
///
/// # Type Parameters
///
/// - `D`: Source codec type.
/// - `E`: Target codec type.
/// - `H`: Converter hook type exposing `Error`.
pub(super) type ConvertErrorOf<D, E, H> = <H as BufferedConvertHooks<D, E>>::Error;

/// Converter progress result type selected by hooks for one target output unit type.
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
/// - `Ok(progress)` when a conversion step advances or completes without error, or
/// - `Err(error)` when conversion cannot continue.
pub(super) type ConvertProgressResult<D, E, H> = Result<TranscodeProgress, ConvertErrorOf<D, E, H>>;
