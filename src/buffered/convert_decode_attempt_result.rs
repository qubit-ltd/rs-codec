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
pub(super) type ConvertDecodeAttemptResult<D, E, H, Input, Value, Output> =
    Result<DecodeStep<Value>, ConvertErrorOf<D, E, H, Input, Value, Output>>;
