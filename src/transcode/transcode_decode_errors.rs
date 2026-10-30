// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors reported by transcode decoders.

#[path = "transcode_decode_error.rs"]
mod transcode_decode_error_impl;

pub use transcode_decode_error_impl::TranscodeDecodeError;
pub use transcode_decode_error_impl::TranscodeDecodeErrorOf;
