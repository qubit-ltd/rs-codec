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
pub trait ValueDecoder<Input: ?Sized> {
    /// Decoded output type.
    type Output;
    /// Decoding error type.
    type Error;
    /// Domain error type accepted by this value facade.
    type DomainError;

    /// Maps a domain error into the public decoding error.
    ///
    /// # Parameters
    /// - `error`: Domain error produced by the underlying codec or policy.
    ///
    /// # Returns
    /// Decoded facade error.
    fn map_error(&self, error: Self::DomainError) -> Self::Error;

    /// Decodes `input`.
    ///
    /// # Parameters
    /// - `input`: Source value to decode.
    ///
    /// # Returns
    /// Decoded output.
    ///
    /// # Errors
    /// Returns an error when the input is malformed or unsupported by the
    /// codec.
    fn decode(&mut self, input: &Input) -> Result<Self::Output, Self::Error>;
}
