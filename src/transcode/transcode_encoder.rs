// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Semantic marker trait for buffered encoders.

use super::Transcoder;

/// Encodes logical values into encoded units over caller-provided buffers.
///
/// `TranscodeEncoder` refines [`Transcoder`] for implementations whose
/// input is the logical value stream and whose output is the encoded unit
/// stream. The trait adds no methods; it exists to make generic bounds
/// distinguish encoding direction from decoding and unit-to-unit conversion.
///
/// The word "buffered" describes the caller-managed buffer and progress model.
/// It does not require the implementor to own an internal buffer.
///
/// # Type Parameters
///
/// - `Value`: Logical value type accepted by the encoder.
/// - `Unit`: Encoded unit type produced by the encoder.
pub trait TranscodeEncoder<Value, Unit>: Transcoder<Value, Unit> {}
