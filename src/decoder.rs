/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Owned-value decoder trait.

/// Decodes a borrowed input value into an owned representation.
///
/// This trait is a convenience-layer API. Use [`crate::Codec`] for low-level
/// single-value buffer decoding and [`crate::Transcoder`] for batch conversion over
/// caller-provided buffers.
pub trait Decoder<Input: ?Sized> {
    /// Decoded output type.
    type Output;
    /// Decoding error type.
    type Error;

    /// Decodes `input`.
    ///
    /// # Parameters
    /// - `input`: Source value to decode.
    ///
    /// # Returns
    /// Decoded output.
    ///
    /// # Errors
    /// Returns an error when the input is malformed or unsupported by the codec.
    fn decode(&self, input: &Input) -> Result<Self::Output, Self::Error>;
}
