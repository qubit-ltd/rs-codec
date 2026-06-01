/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Common codec traits and buffer conversion primitives.
//!
//! Importing this module brings the domain-neutral codec traits, convenience
//! value encoder/decoder traits, byte-order markers, and progress-oriented
//! transcoder types into scope.

pub use crate::{
    BigEndian,
    BufferedConvertEngine,
    BufferedConvertHooks,
    BufferedConverter,
    BufferedDecoder,
    BufferedEncoder,
    ByteOrder,
    ByteOrderSpec,
    CapacityError,
    Codec,
    CodecBufferedConverter,
    CodecBufferedDecoder,
    CodecBufferedEncoder,
    CodecConvertError,
    CodecDecodeError,
    CodecEncodeError,
    CodecValueDecoder,
    CodecValueEncoder,
    ConvertErrorFactory,
    EncodePlan,
    LittleEndian,
    TranscodeProgress,
    TranscodeStatus,
    Transcoder,
    ValueDecoder,
    ValueEncoder,
};
