// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Serde JSON bytes encoder for one owned value.

use core::marker::PhantomData;

use serde::Serialize;
use serde_json::Error as JsonError;
use serde_json::to_vec;

use super::ValueEncoder;

/// Encodes one serializable value into UTF-8 JSON bytes via `serde_json`.
///
/// # Type Parameters
///
/// - `T`: Source value type serialized through Serde.
///
/// # Examples
///
/// ```rust
/// use qubit_codec::JsonBytesValueEncoder;
/// use qubit_codec::ValueEncoder;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// let mut encoder = JsonBytesValueEncoder::<Point>::new();
/// let json = encoder.encode(&Point { x: 1, y: 2 }).unwrap();
/// assert_eq!(json, br#"{"x":1,"y":2}"#);
/// ```
#[must_use]
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonBytesValueEncoder<T> {
    /// Associates the encoder with `T` without retaining a source value.
    _marker: PhantomData<T>,
}

impl<T> JsonBytesValueEncoder<T> {
    /// Creates a JSON bytes encoder for values of type `T`.
    ///
    /// # Returns
    ///
    /// Returns a stateless encoder without allocating an output buffer.
    #[inline]
    pub const fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T> ValueEncoder<T> for JsonBytesValueEncoder<T>
where
    T: Serialize,
{
    /// Serialization failure reported by Serde JSON.
    type Error = JsonError;
    /// Newly allocated UTF-8 JSON bytes owned by the caller.
    type Output = Vec<u8>;

    /// Serializes `input` into UTF-8 JSON bytes.
    ///
    /// # Parameters
    ///
    /// - `input`: Value borrowed for the duration of serialization.
    ///
    /// # Returns
    ///
    /// Returns a newly allocated vector containing the serialized JSON.
    ///
    /// # Errors
    ///
    /// Returns a JSON error if `T` rejects serialization or contains a map key
    /// that cannot be represented as a JSON object key.
    #[inline]
    fn encode(&mut self, input: &T) -> Result<Self::Output, Self::Error> {
        to_vec(input)
    }
}
