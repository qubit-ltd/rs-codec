// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared test helpers for the value-codec integration tests.

mod reset_sensitive_lifecycle_codec;

#[cfg(feature = "json")]
mod owned_json_bytes;

#[cfg(feature = "json")]
pub(crate) use owned_json_bytes::OwnedJsonBytes;
pub(crate) use reset_sensitive_lifecycle_codec::ResetSensitiveLifecycleCodec;
