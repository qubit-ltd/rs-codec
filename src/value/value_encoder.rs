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
/// # Type Parameters
///
/// - `Input`: Borrowed source type accepted by [`encode`](Self::encode). It may
///   be unsized, so implementations can accept slices and string types without
///   an extra reference layer.
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
    /// Owned representation produced from one input value.
    ///
    /// Implementations must return a value that owns its data: `Output` may not
    /// borrow from `Input` or from `self`, so the result outlives the borrow
    /// passed to [`Self::encode`]. The representation is free — for example
    /// `String` or `Vec<u8>` — and its size is the encoding's cost.
    type Output;
    /// Failure reported when one input value cannot be represented.
    ///
    /// Implementations choose the error type and determine which encoding
    /// failures it represents. Callers observe it as the `Err` of
    /// [`Self::encode`].
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
