// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Safe type-erased bidirectional string value-codec descriptors.

use core::any::type_name;
use core::fmt::Debug;
use core::fmt::Formatter;
use core::fmt::Result as FmtResult;
use std::any::Any;
use std::any::TypeId;
use std::error::Error;

use crate::ValueCodecExecutionError;
use crate::ValueDecoder;
use crate::ValueEncoder;

/// Checked type-erased encoding entry point returning an owned JSON string.
type EncodeFn = fn(&dyn Any) -> Result<String, ValueCodecExecutionError>;
/// Type-erased decoding entry point returning a boxed owned value.
type DecodeFn = fn(&str) -> Result<Box<dyn Any>, ValueCodecExecutionError>;

/// An immutable, safely erased bidirectional string codec for one value type.
///
/// Each operation creates a fresh default codec; no codec state is retained
/// between calls. Type identities are process-local and names are diagnostic,
/// not stable IDs.
///
/// # Examples
///
/// ```
/// use core::convert::Infallible;
///
/// use qubit_codec::ValueDecoder;
/// use qubit_codec::ValueEncoder;
/// use qubit_codec::ValueStringCodecDescriptor;
///
/// #[derive(Default)]
/// struct Text;
/// impl ValueEncoder<u32> for Text {
///     type Output = String;
///     type Error = Infallible;
///
///     fn encode(&mut self, input: &u32) -> Result<String, Infallible> {
///         Ok(input.to_string())
///     }
/// }
/// impl ValueDecoder<str> for Text {
///     type Output = u32;
///     type Error = Infallible;
///
///     fn decode(&mut self, input: &str) -> Result<u32, Infallible> {
///         Ok(input.len() as u32)
///     }
/// }
///
/// let descriptor = ValueStringCodecDescriptor::of::<Text, u32>();
///
/// // `encode` takes an erased `&dyn Any`, so the value type must be named.
/// let value: u32 = 7;
/// assert_eq!(descriptor.encode(&value).unwrap(), "7");
///
/// let decoded = descriptor.decode("hello").unwrap();
/// assert_eq!(*decoded.downcast::<u32>().unwrap(), 5);
/// ```
#[derive(Clone, Copy)]
pub struct ValueStringCodecDescriptor {
    /// Returns the process-local identity of the concrete codec.
    codec_type_id: fn() -> TypeId,
    /// Returns the diagnostic name of the concrete codec.
    codec_type_name: fn() -> &'static str,
    /// Returns the process-local identity of the decoded value.
    value_type_id: fn() -> TypeId,
    /// Returns the diagnostic name of the decoded value.
    value_type_name: fn() -> &'static str,
    /// Checks the erased input type and invokes a fresh default encoder.
    encode: EncodeFn,
    /// Invokes a fresh default decoder and boxes its owned output.
    decode: DecodeFn,
}

impl ValueStringCodecDescriptor {
    /// Creates a descriptor for codec `C` and value type `V`.
    ///
    /// # Type Parameters
    ///
    /// - `C`: Default-constructible string encoder/decoder with owned errors.
    /// - `V`: Owned value type accepted and produced by `C`.
    ///
    /// # Returns
    ///
    /// Returns immutable function metadata without constructing a codec.
    #[must_use]
    #[inline]
    pub const fn of<C, V>() -> Self
    where
        C: Default + ValueEncoder<V, Output = String> + ValueDecoder<str, Output = V> + 'static,
        <C as ValueEncoder<V>>::Error: Error + 'static,
        <C as ValueDecoder<str>>::Error: Error + 'static,
        V: 'static,
    {
        Self {
            codec_type_id: TypeId::of::<C>,
            codec_type_name: type_name::<C>,
            value_type_id: TypeId::of::<V>,
            value_type_name: type_name::<V>,
            encode: encode::<C, V>,
            decode: decode::<C, V>,
        }
    }

    /// Returns the process-local identity of the concrete codec.
    ///
    /// # Returns
    ///
    /// Returns the `TypeId` of the codec this descriptor was built for.
    #[must_use]
    #[inline]
    pub fn codec_type_id(&self) -> TypeId {
        (self.codec_type_id)()
    }

    /// Returns the diagnostic name of the concrete codec.
    ///
    /// # Returns
    ///
    /// Returns the `type_name` of the codec, for diagnostics only.
    #[must_use]
    #[inline]
    pub fn codec_type_name(&self) -> &'static str {
        (self.codec_type_name)()
    }

    /// Returns the process-local identity of the decoded value.
    ///
    /// # Returns
    ///
    /// Returns the `TypeId` of the value this descriptor accepts and produces.
    #[must_use]
    #[inline]
    pub fn value_type_id(&self) -> TypeId {
        (self.value_type_id)()
    }

    /// Returns the diagnostic name of the decoded value.
    ///
    /// # Returns
    ///
    /// Returns the `type_name` of the value, for diagnostics only.
    #[must_use]
    #[inline]
    pub fn value_type_name(&self) -> &'static str {
        (self.value_type_name)()
    }

    /// Encodes one erased value after checking its concrete type.
    ///
    /// # Parameters
    ///
    /// - `value`: Borrowed value whose concrete type must match the descriptor.
    ///
    /// # Returns
    ///
    /// Returns an owned JSON string produced by a fresh default codec.
    ///
    /// # Errors
    ///
    /// Returns a type mismatch or the typed encoder source error.
    pub fn encode(&self, value: &dyn Any) -> Result<String, ValueCodecExecutionError> {
        (self.encode)(value)
    }

    /// Decodes one string into a safely erased value of the declared type.
    ///
    /// # Parameters
    ///
    /// - `input`: String borrowed for one decoding operation.
    ///
    /// # Returns
    ///
    /// Returns a boxed owned value that can be downcast to the declared type.
    ///
    /// # Errors
    ///
    /// Returns the typed decoder source error.
    pub fn decode(&self, input: &str) -> Result<Box<dyn Any>, ValueCodecExecutionError> {
        (self.decode)(input)
    }
}

impl Debug for ValueStringCodecDescriptor {
    /// Formats descriptor metadata without invoking either codec function.
    ///
    /// # Parameters
    ///
    /// - `formatter`: Destination receiving the diagnostic type names.
    ///
    /// # Returns
    ///
    /// Returns success after both names have been formatted.
    ///
    /// # Errors
    ///
    /// Returns the underlying formatting failure.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter
            .debug_struct("ValueStringCodecDescriptor")
            .field("codec_type_name", &self.codec_type_name())
            .field("value_type_name", &self.value_type_name())
            .finish_non_exhaustive()
    }
}

/// Downcasts and encodes one typed value.
fn encode<C, V>(value: &dyn Any) -> Result<String, ValueCodecExecutionError>
where
    C: Default + ValueEncoder<V, Output = String> + ValueDecoder<str, Output = V> + 'static,
    <C as ValueEncoder<V>>::Error: Error + 'static,
    <C as ValueDecoder<str>>::Error: Error + 'static,
    V: 'static,
{
    let value = value
        .downcast_ref::<V>()
        .ok_or_else(|| ValueCodecExecutionError::TypeMismatch {
            expected_type: type_name::<V>(),
            actual_type: value.type_id(),
        })?;
    C::default()
        .encode(value)
        .map_err(|source| ValueCodecExecutionError::EncodeFailed {
            codec_type: type_name::<C>(),
            source: Box::new(source),
        })
}

/// Decodes one string and erases the typed output safely.
fn decode<C, V>(input: &str) -> Result<Box<dyn Any>, ValueCodecExecutionError>
where
    C: Default + ValueEncoder<V, Output = String> + ValueDecoder<str, Output = V> + 'static,
    <C as ValueEncoder<V>>::Error: Error + 'static,
    <C as ValueDecoder<str>>::Error: Error + 'static,
    V: 'static,
{
    C::default()
        .decode(input)
        .map(|value| Box::new(value) as Box<dyn Any>)
        .map_err(|source| ValueCodecExecutionError::DecodeFailed {
            codec_type: type_name::<C>(),
            source: Box::new(source),
        })
}
