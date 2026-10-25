// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal drivers and error translation used by buffered transcode adapters.

mod codec_decode_driver;
mod encode_output_error;

pub(in crate::transcode::io) use codec_decode_driver::CodecDecodeDriver;
pub(in crate::transcode::io) use encode_output_error::capacity_error_to_invalid_data;
pub(in crate::transcode::io) use encode_output_error::map_encode_value_result;
