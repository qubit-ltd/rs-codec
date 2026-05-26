/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Owned-value encoder trait.

/// Encodes a borrowed input value into an owned representation.
///
/// This trait is a convenience-layer API. Use [`crate::Codec`] for low-level
/// single-value buffer encoding and [`crate::Coder`] for batch conversion over
/// caller-provided buffers.
pub trait Encoder<Input: ?Sized> {
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
    fn encode(&self, input: &Input) -> Result<Self::Output, Self::Error>;
}
