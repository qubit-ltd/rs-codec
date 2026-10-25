// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Serde JSON string encoder for one owned value.

use core::marker::PhantomData;

use serde::Serialize;

use super::ValueEncoder;

/// Encodes one serializable value into a UTF-8 JSON string via `serde_json`.
/// Its stateless marker implements `Clone`, `Copy`, and `Default` without
/// requiring those traits on `T`.
///
/// # Type Parameters
///
/// - `T`: Source value type serialized through Serde.
///
/// # Examples
///
/// ```rust
/// use qubit_codec::{JsonStringValueEncoder, ValueEncoder};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// let mut encoder = JsonStringValueEncoder::<Point>::new();
/// let json = encoder.encode(&Point { x: 1, y: 2 }).unwrap();
/// assert_eq!(json, r#"{"x":1,"y":2}"#);
/// ```
#[must_use]
#[derive(Debug)]
pub struct JsonStringValueEncoder<T> {
    /// Associates the encoder with `T` without retaining a source value.
    _marker: PhantomData<T>,
}

impl<T> Copy for JsonStringValueEncoder<T> {}

impl<T> Clone for JsonStringValueEncoder<T> {
    /// Copies the stateless encoder marker without requiring `T: Clone`.
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Default for JsonStringValueEncoder<T> {
    /// Creates the stateless encoder through its const constructor.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> JsonStringValueEncoder<T> {
    /// Creates a JSON string encoder for values of type `T`.
    ///
    /// # Returns
    ///
    /// Returns a stateless encoder without allocating an output string.
    #[inline]
    pub const fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T> ValueEncoder<T> for JsonStringValueEncoder<T>
where
    T: Serialize,
{
    type Error = serde_json::Error;
    type Output = String;

    /// Serializes `input` into a compact JSON string.
    ///
    /// # Parameters
    /// - `input`: Value to serialize.
    ///
    /// # Returns
    /// The serialized JSON text as an owned UTF-8 string.
    ///
    /// # Errors
    /// Returns the `serde_json` error if serialization fails, such as when a
    /// custom serializer reports an error or the value contains unsupported
    /// data.
    #[inline]
    fn encode(&mut self, input: &T) -> Result<Self::Output, Self::Error> {
        serde_json::to_string(input)
    }
}
