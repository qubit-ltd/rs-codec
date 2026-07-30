// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Low-level codec contracts.

#[allow(clippy::module_inception)]
mod codec;
mod decode_failure;

pub use codec::Codec;
pub(crate) use codec::{assert_decode_lifecycle_bounds, assert_unit_bounds};
pub use decode_failure::DecodeFailure;
