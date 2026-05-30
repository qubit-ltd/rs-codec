/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! # qubit-codec
//!
//! Core codec traits and buffer conversion primitives for Rust applications.
//!
//! This crate contains only domain-neutral building blocks such as value
//! codecs, owned value encoder/decoder helpers, byte-order markers, and
//! progress-oriented buffer transcoders. Concrete binary, text, misc, and I/O
//! adapters live in sibling crates.
//!

#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

mod buffered;
mod byte_order;
mod codec;
mod value;

pub mod prelude;
pub use buffered::{
    BufferedConvertEngine,
    BufferedConvertHooks,
    BufferedConverter,
    BufferedDecodeEngine,
    BufferedDecodeHooks,
    BufferedDecoder,
    BufferedEncodeEngine,
    BufferedEncodeHooks,
    BufferedEncoder,
    CodecBufferedConverter,
    CodecBufferedDecoder,
    CodecBufferedEncoder,
    ConvertDecodeResult,
    ConvertState,
    ConvertWriteResult,
    DecodeAction,
    DecodeContext,
    EncodePlan,
    TranscodeProgress,
    TranscodeStatus,
    Transcoder,
};
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
    CodecEncodeError,
    ConvertErrorFactory,
    DecodeErrorFactory,
    DecodeErrorInfo,
    DecodeFailure,
    EncodeErrorFactory,
};
pub use value::{
    CodecValueDecoder,
    CodecValueEncoder,
    ValueDecoder,
    ValueEncoder,
};
