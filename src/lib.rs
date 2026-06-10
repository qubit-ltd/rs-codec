// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # qubit-codec
//!
//! Core codec traits and buffer conversion primitives for Rust applications.
//!
//! This crate contains only domain-neutral building blocks such as value
//! codecs, owned value encoder/decoder helpers, byte-order markers, and
//! progress-oriented buffer transcoders. Concrete binary, text, misc, and I/O
//! adapters live in sibling crates.

#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

mod byte_order;
mod core;
mod transcode;
mod value;

pub mod prelude;
pub use byte_order::{BigEndian, ByteOrder, ByteOrderSpec, LittleEndian};
pub use core::{Codec, CodecConvertError, CodecDecodeError, CodecEncodeError};
pub use transcode::{
    CapacityError, CodecTranscodeConverter, CodecTranscodeDecoder, CodecTranscodeEncoder,
    DecodeAction, DecodeContext, EncodeContext, EncodePlan, FinishError, TranscodeConvertEngine,
    TranscodeConvertHooks, TranscodeConverter, TranscodeDecodeEngine, TranscodeDecodeHooks,
    TranscodeDecodeInput, TranscodeDecoder, TranscodeEncodeEngine, TranscodeEncodeHooks,
    TranscodeEncodeOutput, TranscodeEncoder, TranscodeProgress, TranscodeStatus, Transcoder,
};
pub use value::{CodecValueDecoder, CodecValueEncoder, ValueDecoder, ValueEncoder};
