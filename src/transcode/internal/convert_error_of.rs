// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Converter error type alias selected by converter hooks.

use super::super::hooks::TranscodeConvertHooks;

/// Converter error type selected by hooks for one target output unit type.
///
/// # Type Parameters
///
/// - `D`: Source codec type.
/// - `E`: Target codec type.
/// - `H`: Converter hook type exposing `Error`.
pub(in crate::transcode) type ConvertErrorOf<D, E, H> = <H as TranscodeConvertHooks<D, E>>::Error;
