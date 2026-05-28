/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Semantic marker trait for buffered decoders.

use super::Transcoder;

/// Decodes encoded units into logical values over caller-provided buffers.
///
/// `BufferedDecoder` refines [`Transcoder`] for implementations whose input is
/// the encoded unit stream and whose output is the logical value stream. The
/// trait adds no methods; it exists to make generic bounds distinguish decoding
/// direction from encoding and unit-to-unit conversion.
///
/// The word "buffered" describes the caller-managed buffer and progress model.
/// It does not require the implementor to own an internal buffer.
///
/// # Type Parameters
///
/// - `Unit`: Encoded unit type accepted by the decoder.
/// - `Value`: Logical value type produced by the decoder.
pub trait BufferedDecoder<Unit, Value>: Transcoder<Unit, Value> {}
