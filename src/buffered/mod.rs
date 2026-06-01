/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Buffered conversion traits, adapters, and progress status types.

mod buffered_convert_engine;
mod buffered_convert_hooks;
mod buffered_converter;
mod buffered_decode_engine;
mod buffered_decode_hooks;
mod buffered_decoder;
mod buffered_encode_engine;
mod buffered_encode_hooks;
mod buffered_encoder;
mod capacity_error;
mod codec_buffered_convert_hooks;
mod codec_buffered_converter;
mod codec_buffered_decode_hooks;
mod codec_buffered_decoder;
mod codec_buffered_encode_hooks;
mod codec_buffered_encoder;
mod convert_decode_attempt_result;
mod convert_encode_result;
mod convert_error_of;
mod convert_state;
mod convert_step_result;
mod decode_action;
mod decode_context;
mod decode_state;
mod decode_step;
mod encode_attempt;
mod encode_plan;
mod encode_state;
mod pending_value;
mod transcode_progress;
mod transcode_status;
mod transcoder;

pub use buffered_convert_engine::BufferedConvertEngine;
pub use buffered_convert_hooks::BufferedConvertHooks;
pub use buffered_converter::BufferedConverter;
pub use buffered_decode_engine::BufferedDecodeEngine;
pub use buffered_decode_hooks::BufferedDecodeHooks;
pub use buffered_decoder::BufferedDecoder;
pub use buffered_encode_engine::BufferedEncodeEngine;
pub use buffered_encode_hooks::BufferedEncodeHooks;
pub use buffered_encoder::BufferedEncoder;
pub use capacity_error::CapacityError;
pub use codec_buffered_converter::CodecBufferedConverter;
pub use codec_buffered_decoder::CodecBufferedDecoder;
pub use codec_buffered_encoder::CodecBufferedEncoder;
pub use decode_action::DecodeAction;
pub use decode_context::DecodeContext;
pub use encode_plan::EncodePlan;
pub use transcode_progress::TranscodeProgress;
pub use transcode_status::TranscodeStatus;
pub use transcoder::Transcoder;
