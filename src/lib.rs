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
//! progress-oriented buffer transcoders. The only I/O-facing types are the
//! low-level `qubit_io::Input`/`qubit_io::Output` bridges used by downstream
//! stream crates. Concrete binary, text, misc, and `std::io` adapters live in
//! sibling crates.

#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

mod byte_order;
mod codec;
mod transcode;
mod value;

pub use byte_order::{
    BigEndian,
    ByteOrder,
    ByteOrderSpec,
    LittleEndian,
};
pub use codec::{
    Codec,
    CodecConvertError,
    CodecDecodeError,
    CodecDecodeSignal,
    CodecEncodeError,
};
pub use qubit_io::nz;
pub use transcode::{
    CapacityError,
    CodecTranscodeConverter,
    CodecTranscodeDecoder,
    CodecTranscodeEncoder,
    DecodeAction,
    DecodeContext,
    EncodeContext,
    EncodePlan,
    TranscodeConvertEngine,
    TranscodeConvertHooks,
    TranscodeConverter,
    TranscodeDecodeEngine,
    TranscodeDecodeHooks,
    TranscodeDecodeInput,
    TranscodeDecoder,
    TranscodeEncodeEngine,
    TranscodeEncodeHooks,
    TranscodeEncodeOutput,
    TranscodeEncoder,
    TranscodeError,
    TranscodeProgress,
    TranscodeStatus,
    Transcoder,
};
pub use value::{
    CodecValueDecoder,
    CodecValueEncoder,
    ValueDecoder,
    ValueEncoder,
};
