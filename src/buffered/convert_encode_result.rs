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
pub(super) type ConvertEncodeResult<D, E, H, Input, Value, Output> =
    Result<PendingEncodeStep<Value>, ConvertErrorOf<D, E, H, Input, Value, Output>>;
