// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal policy hook implementations for the codec-backed adapters.

mod codec_transcode_decode_hooks;
mod codec_transcode_encode_hooks;

pub(in crate::transcode) use codec_transcode_decode_hooks::CodecTranscodeDecodeHooks;
pub(in crate::transcode) use codec_transcode_encode_hooks::CodecTranscodeEncodeHooks;
