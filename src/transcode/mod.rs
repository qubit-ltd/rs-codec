// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Progress-oriented streaming transcode traits, adapters, and status types.

mod adapter;
mod capacity_error;
mod codec_phase;
mod convert_error;
mod engine;
mod internal;
mod io;
mod transcode_contract_error;
mod transcode_converter;
mod transcode_decoder;
mod transcode_encoder;
mod transcode_error;
mod transcode_progress;
mod transcode_status;
mod transcoder;

pub use adapter::{CodecTranscodeConverter, CodecTranscodeDecoder, CodecTranscodeEncoder};
pub use capacity_error::CapacityError;
pub use codec_phase::CodecPhase;
pub use convert_error::{ConvertError, TranscodeConvertError};
pub use engine::{
    DecodeContext, DecodeInvalidAction, DecodeOutcome, EncodeContext, EncodeOutcome,
    EncodeUnencodableAction, TranscodeConvertEngine, TranscodeDecodeEngine, TranscodeDecodeHooks,
    TranscodeEncodeEngine, TranscodeEncodeHooks,
};
pub use io::{TranscodeDecodeInput, TranscodeEncodeOutput};
pub use transcode_contract_error::TranscodeContractError;
pub use transcode_converter::TranscodeConverter;
pub use transcode_decoder::TranscodeDecoder;
pub use transcode_encoder::TranscodeEncoder;
pub use transcode_error::{TranscodeDecodeError, TranscodeEncodeError, TranscodeError};
pub use transcode_progress::TranscodeProgress;
pub use transcode_status::TranscodeStatus;
pub use transcoder::Transcoder;
