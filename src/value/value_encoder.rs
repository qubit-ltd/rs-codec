// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Owned-value encoder trait.

/// Encodes a borrowed input value into an owned representation.
///
/// This trait is a convenience-layer API. Use [`crate::Codec`] for low-level
/// single-value buffer encoding and [`crate::Transcoder`] for batch
/// conversion over caller-provided buffers.
///
/// # Examples
///
/// ```
/// use qubit_codec::ValueEncoder;
///
/// struct TextEncoder;
/// impl ValueEncoder<str> for TextEncoder {
///     type Output = Vec<u8>;
///     type Error = core::convert::Infallible;
///
///     fn encode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
///         Ok(input.as_bytes().to_vec())
///     }
/// }
///
/// let mut encoder = TextEncoder;
/// assert_eq!(encoder.encode("hello").unwrap(), b"hello");
/// ```
pub trait ValueEncoder<Input: ?Sized> {
    /// Encoded output type.
    type Output;
    /// Encoding error type.
    type Error;

    /// Encodes `input`.
    ///
    /// # Parameters
    /// - `input`: Source value to encode.
    ///
    /// # Returns
    /// Encoded output.
    ///
    /// # Errors
    /// Returns an error when the codec cannot represent the supplied input.
    fn encode(&mut self, input: &Input) -> Result<Self::Output, Self::Error>;
}
