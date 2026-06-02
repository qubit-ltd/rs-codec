/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Result type for private pending-value encode steps.

use super::{
    convert_error_of::ConvertErrorOf,
    pending_encode_step::PendingEncodeStep,
};

/// Result type for private pending-value encode steps.
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
/// - `Ok(PendingEncodeStep<D::Value>)` when pending write handling proceeds,
/// - `Err(...)` for mapped converter-level encode errors.
pub(super) type ConvertEncodeResult<D, E, H> =
    Result<PendingEncodeStep<<D as crate::Codec>::Value>, ConvertErrorOf<D, E, H>>;
