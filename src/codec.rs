/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Low-level value codec trait.

/// Encodes and decodes one value or codec quantum against a unit buffer.
///
/// `Codec` is the lowest-level abstraction in the codec stack. It is intended
/// for hot paths that have already validated buffer capacity and want to avoid
/// constructing subslices for every value. Higher-level coders and convenience
/// APIs are responsible for checked buffer management, partial-input reporting,
/// and owned output allocation.
///
/// # Type Parameters
///
/// - `Value`: Logical value decoded from or encoded into the buffer. This may be
///   a scalar such as `u64`, a `char`, or a fixed quantum such as `[u8; 3]`.
/// - `Unit`: Buffer unit used by the encoded representation.
///
/// # Safety
///
/// Implementors must uphold the safety contract documented by
/// [`decode_unchecked`](Self::decode_unchecked) and
/// [`encode_unchecked`](Self::encode_unchecked). In particular, unchecked
/// implementations must not read or write outside the caller-provided ranges.
pub unsafe trait Codec<Value, Unit: Copy> {
    /// Error reported when decoding malformed units.
    type DecodeError;

    /// Error reported when encoding an unsupported value.
    type EncodeError;

    /// Returns the minimum unit count needed to encode or decode one value.
    ///
    /// # Returns
    ///
    /// Returns a lower bound for one complete value. Variable-width codecs such
    /// as LEB128 should return the shortest valid representation length.
    #[must_use]
    fn min_units_per_value(&self) -> usize;

    /// Returns the maximum unit count needed to encode or decode one value.
    ///
    /// # Returns
    ///
    /// Returns an upper bound for one complete value or codec quantum.
    #[must_use]
    fn max_units_per_value(&self) -> usize;

    /// Decodes one value from `input` starting at `index`.
    ///
    /// # Parameters
    ///
    /// - `input`: Source unit buffer.
    /// - `index`: Start index in `input`.
    ///
    /// # Returns
    ///
    /// Returns the decoded value and the number of consumed units.
    ///
    /// # Errors
    ///
    /// Returns `Self::DecodeError` when the units are malformed for this codec.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the implementation can read enough units
    /// for one complete value starting at `index`. For fixed-width codecs this
    /// means at least [`max_units_per_value`](Self::max_units_per_value) units.
    /// For variable-width codecs this means either a valid terminator appears
    /// before that maximum, or the full maximum-width range is readable.
    unsafe fn decode_unchecked(&self, input: &[Unit], index: usize) -> Result<(Value, usize), Self::DecodeError>;

    /// Encodes one value into `output` starting at `index`.
    ///
    /// # Parameters
    ///
    /// - `value`: Value to encode.
    /// - `output`: Destination unit buffer.
    /// - `index`: Start index in `output`.
    ///
    /// # Returns
    ///
    /// Returns the number of written units.
    ///
    /// # Errors
    ///
    /// Returns `Self::EncodeError` when `value` cannot be represented by this
    /// codec.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the implementation can write up to
    /// [`max_units_per_value`](Self::max_units_per_value) units starting at
    /// `index`.
    unsafe fn encode_unchecked(
        &self,
        value: Value,
        output: &mut [Unit],
        index: usize,
    ) -> Result<usize, Self::EncodeError>;
}
