/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Low-level codec contracts and adapter error types.

#[allow(clippy::module_inception)]
mod codec;
mod codec_convert_error;
mod codec_decode_error;
mod codec_encode_error;
mod convert_error_factory;
mod encode_error_factory;

pub use codec::Codec;
pub(crate) use codec::debug_assert_unit_bounds;
pub use codec_convert_error::CodecConvertError;
pub use codec_decode_error::CodecDecodeError;
pub use codec_encode_error::CodecEncodeError;
pub use convert_error_factory::ConvertErrorFactory;
pub use encode_error_factory::EncodeErrorFactory;
