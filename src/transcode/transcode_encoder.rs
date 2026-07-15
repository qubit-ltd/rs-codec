// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Semantic marker trait for buffered encoders.

use super::{TranscodeEncodeError, Transcoder};

/// Encodes logical values into encoded units over caller-provided buffers.
///
/// `TranscodeEncoder` refines [`Transcoder`] for implementations whose
/// input is the logical value stream and whose output is the encoded unit
/// stream. The trait adds no methods; it exists to make generic bounds
/// distinguish encoding direction from decoding and unit-to-unit conversion.
///
/// The word "buffered" describes the caller-managed buffer and progress model.
/// It does not require the implementor to own an internal buffer.
pub trait TranscodeEncoder:
    Transcoder<Error = TranscodeEncodeError<Self::EncodeError, <Self as Transcoder>::Input>>
{
    /// Domain error type produced by encode internals.
    type EncodeError;
}
