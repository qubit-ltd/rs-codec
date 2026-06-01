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
pub(super) type ConvertDecodeFinishResult<D, E, H, Input, Value, Output> =
    Result<DecodeFinishStep<Value>, ConvertErrorOf<D, E, H, Input, Value, Output>>;
