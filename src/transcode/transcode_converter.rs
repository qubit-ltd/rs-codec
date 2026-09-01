// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Semantic marker trait for buffered converters.

use super::TranscodeConvertError;
use super::Transcoder;

/// Converts encoded units of one representation into encoded units of another.
///
/// `TranscodeConverter` refines [`Transcoder`] for implementations whose
/// input and output are both encoded unit streams. Any intermediate logical
/// values are implementation details of the concrete converter.
///
/// The trait adds no methods. It exists to make generic bounds distinguish
/// unit-to-unit conversion from value-to-unit encoding and unit-to-value
/// decoding.
///
/// # Examples
///
/// ```
/// use qubit_codec::TranscodeConverter;
///
/// fn accepts_converter<C: TranscodeConverter>(converter: &C) {
///     let _ = converter;
/// }
/// ```
pub trait TranscodeConverter:
    Transcoder<Error = TranscodeConvertError<Self::DecodeError, Self::EncodeError, Self::Value>>
{
    /// Domain error type produced by source decoding.
    type DecodeError;

    /// Domain error type produced by target encoding.
    type EncodeError;

    /// Intermediate logical value type converted between source and target.
    type Value;
}
