// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # qubit-codec
//!
//! Domain-neutral codec traits and buffer conversion primitives.
//!
//! `qubit-codec` is the foundation shared by the Qubit binary, text, misc, and
//! I/O codec crates. It defines the small contracts that concrete format crates
//! build on, without shipping concrete binary or text formats itself.
//!
//! # Overview
//!
//! The crate provides:
//!
//! - [`Codec`] for low-level single-value codecs over caller-managed buffers.
//! - [`CodecValueEncoder`], [`CodecValueDecoder`], and [`CodecValueExt`] for
//!   owned one-value convenience APIs.
//! - [`CodecTranscodeEncoder`], [`CodecTranscodeDecoder`], and
//!   [`CodecTranscodeConverter`] for strict streaming adapters around a
//!   [`Codec`].
//! - [`TranscodeEncodeEngine`], [`TranscodeDecodeEngine`], and
//!   [`TranscodeConvertEngine`] for policy-aware buffered loops.
//! - [`TranscodeEncodeHooks`] and [`TranscodeDecodeHooks`] for replacement,
//!   skip, report, finish, and reset policy decisions.
//! - [`Transcoder`], [`TranscodeProgress`], and [`TranscodeStatus`] for
//!   caller-managed streaming conversion.
//! - [`ValueEncoder`] and [`ValueDecoder`] for whole-value convenience APIs.
//! - [`ByteOrder`], [`ByteOrderSpec`], [`BigEndian`], and [`LittleEndian`] for
//!   shared byte-order metadata.
//!
//! Concrete codecs live in sibling crates such as `qubit-codec-binary`,
//! `qubit-codec-text`, and `qubit-codec-misc`.
//!
//! # Choosing an Abstraction
//!
//! Pick the smallest layer that matches the shape of the problem:
//!
//! ```text
//! What are you writing?
//!
//! +-- A codec for one logical value
//! |   (UTF-8 char, LEB128 integer, fixed-width scalar, ...)
//! |       -> implement Codec
//! |
//! +-- A whole-value codec where a single-value quantum is not useful
//! |   (Base64 text, percent encoding, escaped strings, ...)
//! |       -> implement ValueEncoder<Input> / ValueDecoder<Input>
//! |
//! +-- A strict streaming wrapper around an existing Codec
//! |       -> use CodecTranscodeDecoder<C> / CodecTranscodeEncoder<C>
//! |          / CodecTranscodeConverter<D, E>
//! |
//! +-- An owned-output wrapper around an existing Codec
//! |       -> use CodecValueEncoder<C> / CodecValueDecoder<C>
//! |
//! +-- A streaming codec with policy decisions
//!     (skip, replace, count, report, finish output, ...)
//!         -> implement TranscodeDecodeHooks<C> / TranscodeEncodeHooks<C>
//!            and use TranscodeDecodeEngine<C, H>
//!            / TranscodeEncodeEngine<C, H>
//! ```
//!
//! Unit-to-unit conversions, such as UTF-8 bytes to UTF-16 units, compose a
//! decode side and an encode side. Use [`CodecTranscodeConverter`] for a strict
//! pipeline and [`TranscodeConvertEngine`] when either side needs policy hooks.
//!
//! # Layer Overview
//!
//! ```text
//! concrete I/O crates
//!   |
//! TranscodeDecodeInput / TranscodeEncodeOutput
//!   |
//! Transcode*Engine + Transcode*Hooks
//! CodecTranscodeDecoder / Encoder / Converter
//!   |
//! Transcoder + TranscodeProgress + TranscodeStatus
//! ValueEncoder / ValueDecoder
//!   |
//! Codec
//! ```
//!
//! Implementing further up the stack does not require rewriting the lower
//! layers. The adapter types turn any suitable [`Codec`] into owned-value or
//! streaming APIs. Drop down to the engine and hook layer only when the codec
//! needs policy decisions or retained finish output.

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
    CodecEncodeError,
    DecodeFailure,
};
pub use transcode::{
    CapacityError,
    CodecTranscodeConverter,
    CodecTranscodeDecoder,
    CodecTranscodeEncoder,
    DecodeContext,
    DecodeInvalidAction,
    DecodeOutcome,
    EncodeContext,
    EncodeOutcome,
    EncodeUnencodableAction,
    TranscodeContractError,
    TranscodeConvertEngine,
    TranscodeConvertEngineError,
    TranscodeConverter,
    TranscodeDecodeEngine,
    TranscodeDecodeEngineError,
    TranscodeDecodeHooks,
    TranscodeDecodeInput,
    TranscodeDecoder,
    TranscodeEncodeEngine,
    TranscodeEncodeEngineError,
    TranscodeEncodeHooks,
    TranscodeEncodeOutput,
    TranscodeEncoder,
    TranscodeError,
    TranscodeProgress,
    TranscodeStatus,
    Transcoder,
};
pub use value::{
    CodecDecodeExactValueWithFlushResult,
    CodecDecodeValueWithFlushResult,
    CodecEncodeValueResult,
    CodecValueDecoder,
    CodecValueEncoder,
    CodecValueExt,
    ValueDecoder,
    ValueEncoder,
};
