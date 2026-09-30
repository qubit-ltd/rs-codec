// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Safe type-erased bidirectional bytes value-codec descriptors.

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

/// Checked type-erased encoding entry point returning caller-owned bytes.
type EncodeFn = fn(&dyn Any) -> Result<Vec<u8>, ValueCodecExecutionError>;
/// Type-erased decoding entry point returning a boxed owned value.
type DecodeFn = fn(&[u8]) -> Result<Box<dyn Any>, ValueCodecExecutionError>;

/// An immutable, safely erased bidirectional bytes codec for one value type.
///
/// Each operation creates a fresh default codec; no codec state is retained.
/// Type identities are process-local and names are diagnostic, not stable IDs.
///
/// # Examples
///
/// ```
/// use core::convert::Infallible;
///
/// use qubit_codec::ValueBytesCodecDescriptor;
/// use qubit_codec::ValueDecoder;
/// use qubit_codec::ValueEncoder;
///
/// #[derive(Default)]
/// struct Bytes;
/// impl ValueEncoder<Vec<u8>> for Bytes {
///     type Error = Infallible;
///     type Output = Vec<u8>;
///     fn encode(&mut self, value: &Vec<u8>) -> Result<Vec<u8>, Infallible> {
///         Ok(value.clone())
///     }
/// }
/// impl ValueDecoder<[u8]> for Bytes {
///     type Error = Infallible;
///     type Output = Vec<u8>;
///     fn decode(&mut self, input: &[u8]) -> Result<Vec<u8>, Infallible> {
///         Ok(input.to_vec())
///     }
/// }
/// let descriptor = ValueBytesCodecDescriptor::of::<Bytes, Vec<u8>>();
/// let bytes = descriptor.encode(&vec![1_u8, 2]).unwrap();
/// let decoded = descriptor.decode(&bytes).unwrap();
/// assert_eq!(*decoded.downcast::<Vec<u8>>().unwrap(), vec![1, 2]);
/// ```
#[derive(Clone, Copy)]
pub struct ValueBytesCodecDescriptor {
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

impl ValueBytesCodecDescriptor {
    /// Creates a descriptor for codec `C` and value type `V`.
    ///
    /// # Type Parameters
    ///
    /// - `C`: Default-constructible bytes encoder/decoder with owned errors.
    /// - `V`: Owned value type accepted and produced by `C`.
    ///
    /// # Returns
    ///
    /// Returns immutable function metadata without constructing a codec.
    #[must_use]
    #[inline]
    pub const fn of<C, V>() -> Self
    where
        C: Default + ValueEncoder<V, Output = Vec<u8>> + ValueDecoder<[u8], Output = V> + 'static,
        <C as ValueEncoder<V>>::Error: Error + 'static,
        <C as ValueDecoder<[u8]>>::Error: Error + 'static,
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

    /// Returns the process-local codec type identity.
    ///
    /// # Returns
    ///
    /// Returns metadata for the type selected by [`Self::of`], without
    /// allocation.
    #[must_use]
    #[inline]
    pub fn codec_type_id(&self) -> TypeId {
        (self.codec_type_id)()
    }

    /// Returns the diagnostic codec type name.
    ///
    /// # Returns
    ///
    /// Returns metadata for the type selected by [`Self::of`], without
    /// allocation.
    #[must_use]
    #[inline]
    pub fn codec_type_name(&self) -> &'static str {
        (self.codec_type_name)()
    }

    /// Returns the process-local value type identity.
    ///
    /// # Returns
    ///
    /// Returns metadata for the type selected by [`Self::of`], without
    /// allocation.
    #[must_use]
    #[inline]
    pub fn value_type_id(&self) -> TypeId {
        (self.value_type_id)()
    }

    /// Returns the diagnostic value type name.
    ///
    /// # Returns
    ///
    /// Returns metadata for the type selected by [`Self::of`], without
    /// allocation.
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
    /// Returns owned bytes produced by a fresh default codec.
    ///
    /// # Errors
    ///
    /// Returns a type mismatch or the typed encoder source error.
    pub fn encode(&self, value: &dyn Any) -> Result<Vec<u8>, ValueCodecExecutionError> {
        (self.encode)(value)
    }

    /// Decodes one byte slice into a safely erased value of the declared type.
    ///
    /// # Parameters
    ///
    /// - `input`: Bytes borrowed for one decoding operation.
    ///
    /// # Returns
    ///
    /// Returns a boxed owned value that can be downcast to the declared type.
    ///
    /// # Errors
    ///
    /// Returns the typed decoder source error.
    pub fn decode(&self, input: &[u8]) -> Result<Box<dyn Any>, ValueCodecExecutionError> {
        (self.decode)(input)
    }
}

impl Debug for ValueBytesCodecDescriptor {
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
    /// Returns the formatting error if the destination rejects output.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter
            .debug_struct("ValueBytesCodecDescriptor")
            .field("codec_type_name", &self.codec_type_name())
            .field("value_type_name", &self.value_type_name())
            .finish_non_exhaustive()
    }
}

/// Downcasts and encodes one typed value using a fresh default codec.
///
/// # Type Parameters
///
/// - `C`: Default codec with matching bytes encode/decode contracts.
/// - `V`: Concrete owned value type required by the descriptor.
///
/// # Parameters
///
/// - `value`: Erased input borrowed until serialization completes.
///
/// # Returns
///
/// Returns caller-owned encoded bytes.
///
/// # Errors
///
/// Returns `TypeMismatch` for a different concrete type, or `EncodeFailed`
/// retaining the codec source error if encoding fails.
fn encode<C, V>(value: &dyn Any) -> Result<Vec<u8>, ValueCodecExecutionError>
where
    C: Default + ValueEncoder<V, Output = Vec<u8>> + ValueDecoder<[u8], Output = V> + 'static,
    <C as ValueEncoder<V>>::Error: Error + 'static,
    <C as ValueDecoder<[u8]>>::Error: Error + 'static,
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

/// Decodes one byte slice and erases the typed output safely.
///
/// # Type Parameters
///
/// - `C`: Default codec with matching bytes encode/decode contracts.
/// - `V`: Concrete owned output type placed in the returned box.
///
/// # Parameters
///
/// - `input`: Borrowed bytes passed to a fresh default decoder.
///
/// # Returns
///
/// Returns a newly boxed decoded value with no borrow of the input.
///
/// # Errors
///
/// Returns `DecodeFailed` retaining the codec source error on decode failure.
fn decode<C, V>(input: &[u8]) -> Result<Box<dyn Any>, ValueCodecExecutionError>
where
    C: Default + ValueEncoder<V, Output = Vec<u8>> + ValueDecoder<[u8], Output = V> + 'static,
    <C as ValueEncoder<V>>::Error: Error + 'static,
    <C as ValueDecoder<[u8]>>::Error: Error + 'static,
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
