// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Buffered conversion traits, adapters, and progress status types.

mod buffered_convert_engine;
mod buffered_convert_hooks;
mod buffered_converter;
mod buffered_decode_engine;
mod buffered_decode_hooks;
mod buffered_decode_input;
mod buffered_decoder;
mod buffered_encode_engine;
mod buffered_encode_hooks;
mod buffered_encode_output;
mod buffered_encoder;
mod buffered_transcoder;
mod capacity_error;
mod codec_buffered_convert_hooks;
mod codec_buffered_converter;
mod codec_buffered_decode_hooks;
mod codec_buffered_decoder;
mod codec_buffered_encode_hooks;
mod codec_buffered_encoder;
mod convert_error_of;
mod convert_state;
mod convert_step_result;
mod decode_action;
mod decode_context;
mod decode_state;
mod decode_step;
mod encode_context;
mod encode_plan;
mod encode_state;
mod encode_step;
mod finish_error;
mod pending_encode_step;
mod pending_value;
mod pending_value_slot;
mod transcode_progress;
mod transcode_status;

pub use buffered_convert_engine::BufferedConvertEngine;
pub use buffered_convert_hooks::BufferedConvertHooks;
pub use buffered_converter::BufferedConverter;
pub use buffered_decode_engine::BufferedDecodeEngine;
pub use buffered_decode_hooks::BufferedDecodeHooks;
pub use buffered_decode_input::BufferedDecodeInput;
pub use buffered_decoder::BufferedDecoder;
pub use buffered_encode_engine::BufferedEncodeEngine;
pub use buffered_encode_hooks::BufferedEncodeHooks;
pub use buffered_encode_output::BufferedEncodeOutput;
pub use buffered_encoder::BufferedEncoder;
pub use buffered_transcoder::BufferedTranscoder;
pub use capacity_error::CapacityError;
pub use codec_buffered_converter::CodecBufferedConverter;
pub use codec_buffered_decoder::CodecBufferedDecoder;
pub use codec_buffered_encoder::CodecBufferedEncoder;
pub use decode_action::DecodeAction;
pub use decode_context::DecodeContext;
pub use encode_context::EncodeContext;
pub use encode_plan::EncodePlan;
pub use finish_error::FinishError;
pub use transcode_progress::TranscodeProgress;
pub use transcode_status::TranscodeStatus;
