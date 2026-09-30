// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Owned-value decoder trait.

/// Decodes a borrowed input value into an owned representation.
///
/// This trait is a convenience-layer API. Use [`crate::Codec`] for low-level
/// single-value buffer decoding and [`crate::Transcoder`] for batch
/// conversion over caller-provided buffers.
///
/// # Type Parameters
///
/// - `Input`: Borrowed source type accepted by [`decode`](Self::decode). It may
///   be unsized, so implementations can accept slices and string types without
///   an extra reference layer.
///
/// # Examples
///
/// ```
/// use core::convert::Infallible;
///
/// use qubit_codec::ValueDecoder;
///
/// struct Identity;
/// impl ValueDecoder<str> for Identity {
///     type Output = String;
///     type Error = Infallible;
///
///     fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
///         Ok(input.to_owned())
///     }
/// }
///
/// let mut decoder = Identity;
/// assert_eq!(decoder.decode("hello").unwrap(), "hello");
/// ```
pub trait ValueDecoder<Input: ?Sized> {
    /// Owned value produced from one successfully decoded `Input`.
    ///
    /// The output is returned by value, so an implementation that borrows from
    /// `Input` cannot satisfy this associated type and must allocate or copy.
    type Output;

    /// Error reported when `Input` cannot be decoded.
    ///
    /// Implementations report malformed or unsupported input here; callers
    /// cannot observe partial results because a failed call yields no value.
    type Error;

    /// Decodes `input`.
    ///
    /// # Parameters
    /// - `input`: Source value to decode.
    ///
    /// # Returns
    /// Returns the owned value decoded from `input`.
    ///
    /// # Errors
    /// Returns an error when the input is malformed or unsupported by the
    /// codec.
    fn decode(&mut self, input: &Input) -> Result<Self::Output, Self::Error>;
}
