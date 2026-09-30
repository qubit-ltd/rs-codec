// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal single-codec drivers used by the buffered transcode inputs.

mod codec_decode_driver;

pub(in crate::transcode::io) use codec_decode_driver::CodecDecodeDriver;
