// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Low-level codec contracts and adapter error types.

#[allow(clippy::module_inception)]
mod codec;
mod codec_convert_error;
mod codec_decode_error;
mod codec_decode_failure;
mod codec_decode_flush_error;
mod codec_encode_error;
mod codec_encode_reset_error;

pub use codec::Codec;
pub(crate) use codec::assert_unit_bounds;
pub use codec_convert_error::CodecConvertError;
pub use codec_decode_error::CodecDecodeError;
pub use codec_decode_failure::CodecDecodeFailure;
pub use codec_decode_flush_error::CodecDecodeFlushError;
pub use codec_encode_error::CodecEncodeError;
pub use codec_encode_reset_error::CodecEncodeResetError;
