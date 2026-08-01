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
mod engine;
mod internal;
#[cfg(feature = "io")]
mod io;
mod transcode_contract_error;
mod transcode_convert_error;
mod transcode_converter;
mod transcode_decode_error;
mod transcode_decoder;
mod transcode_domain_error;
mod transcode_encode_error;
mod transcode_encoder;
mod transcode_failure;
mod transcode_progress;
mod transcode_status;
mod transcoder;

pub use adapter::{CodecTranscodeConverter, CodecTranscodeDecoder, CodecTranscodeEncoder};
pub use capacity_error::CapacityError;
pub use engine::{
    DecodeContext, DecodeInvalidAction, DecodeInvalidActionOf, EncodeContext,
    EncodeUnencodableAction, EncodeUnencodableActionOf, TranscodeConvertEngine,
    TranscodeDecodeEngine, TranscodeDecodeHooks, TranscodeEncodeEngine, TranscodeEncodeHooks,
};
#[cfg(feature = "io")]
pub use io::{
    AsyncTranscodeDecodeInput, AsyncTranscodeDecodeStep, AsyncTranscodeEncodeOutput,
    TranscodeDecodeInput, TranscodeEncodeOutput,
};
pub use transcode_contract_error::TranscodeContractError;
pub use transcode_convert_error::{TranscodeConvertError, TranscodeConvertErrorOf};
pub use transcode_converter::TranscodeConverter;
pub use transcode_decode_error::{TranscodeDecodeError, TranscodeDecodeErrorOf};
pub use transcode_decoder::TranscodeDecoder;
pub use transcode_domain_error::TranscodeDomainError;
pub use transcode_encode_error::{TranscodeEncodeError, TranscodeEncodeErrorOf};
pub use transcode_encoder::TranscodeEncoder;
pub use transcode_failure::TranscodeFailure;
pub use transcode_progress::TranscodeProgress;
pub use transcode_status::TranscodeStatus;
pub use transcoder::Transcoder;
