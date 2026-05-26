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
//! codecs, owned encoder/decoder helpers, byte-order markers, and
//! progress-oriented buffer coders. Concrete binary, text, misc, and I/O
//! adapters live in sibling crates.
//!

#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

mod byte_order;
mod codec;
mod coder;
mod decoder;
mod encoder;

pub mod prelude;
pub use byte_order::{
    BigEndian,
    ByteOrder,
    ByteOrderSpec,
    LittleEndian,
};
pub use codec::Codec;
pub use coder::{
    Coder,
    CoderProgress,
    CoderStatus,
};
pub use decoder::Decoder;
pub use encoder::Encoder;
