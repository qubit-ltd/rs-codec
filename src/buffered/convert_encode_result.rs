/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Result type for private encode attempts.

use super::{
    convert_error_of::ConvertErrorOf,
    encode_attempt::EncodeAttempt,
};

/// Result type for private encode attempts.
pub(super) type ConvertEncodeResult<D, E, H, Input, Value, Output> =
    Result<EncodeAttempt<Value>, ConvertErrorOf<D, E, H, Input, Value, Output>>;
