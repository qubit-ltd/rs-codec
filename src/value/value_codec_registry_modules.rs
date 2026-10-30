// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable local and process-wide value-codec registries.

#[path = "value_codec_registry.rs"]
mod local_value_codec_registry;

pub use local_value_codec_registry::ValueBytesCodecRegistry;
pub use local_value_codec_registry::ValueCodecRegistry;
pub use local_value_codec_registry::ValueStringCodecRegistry;
