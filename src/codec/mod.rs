/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Low-level codec contracts and decode failure metadata.

#[allow(clippy::module_inception)]
mod codec;
mod decode_error_info;
mod decode_failure;

pub use codec::Codec;
pub use decode_error_info::DecodeErrorInfo;
pub use decode_failure::DecodeFailure;
