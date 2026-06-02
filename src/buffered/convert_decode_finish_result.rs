/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Result type for source-side finish steps.

use super::{
    convert_error_of::ConvertErrorOf,
    decode_finish_step::DecodeFinishStep,
};

/// Result type for source-side finish steps.
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
/// - `Ok(DecodeFinishStep<D::Value>)` for mapped finish behavior,
/// - `Err(...)` for mapped converter-level decode errors.
pub(super) type ConvertDecodeFinishResult<D, E, H> =
    Result<DecodeFinishStep<<D as crate::Codec>::Value>, ConvertErrorOf<D, E, H>>;
