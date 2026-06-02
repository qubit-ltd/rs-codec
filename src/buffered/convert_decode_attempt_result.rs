/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Result type for private decode steps.

use super::{
    convert_error_of::ConvertErrorOf,
    decode_step::DecodeStep,
};

/// Result type for private decode steps.
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
/// - `Ok(DecodeStep<D::Value>)` for one source decode attempt,
/// - `Err(...)` for mapped converter-level decode errors.
pub(super) type ConvertDecodeAttemptResult<D, E, H> =
    Result<DecodeStep<<D as crate::Codec>::Value>, ConvertErrorOf<D, E, H>>;
