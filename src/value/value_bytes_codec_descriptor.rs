// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Safe type-erased bidirectional bytes value-codec descriptors.

use std::any::Any;
use std::any::TypeId;

use crate::ValueCodecExecutionError;
use crate::ValueDecoder;
use crate::ValueEncoder;

type EncodeFn = fn(&dyn Any) -> Result<Vec<u8>, ValueCodecExecutionError>;
type DecodeFn = fn(&[u8]) -> Result<Box<dyn Any>, ValueCodecExecutionError>;

/// An immutable, safely erased bidirectional bytes codec for one value type.
#[derive(Clone, Copy)]
pub struct ValueBytesCodecDescriptor {
    codec_type_id: fn() -> TypeId,
    codec_type_name: fn() -> &'static str,
    value_type_id: fn() -> TypeId,
    value_type_name: fn() -> &'static str,
    encode: EncodeFn,
    decode: DecodeFn,
}

impl ValueBytesCodecDescriptor {
    /// Creates a descriptor for codec `C` and value type `V`.
    #[must_use]
    pub const fn of<C, V>() -> Self
    where
        C: Default + ValueEncoder<V, Output = Vec<u8>> + ValueDecoder<[u8], Output = V> + 'static,
        <C as ValueEncoder<V>>::Error: std::error::Error + 'static,
        <C as ValueDecoder<[u8]>>::Error: std::error::Error + 'static,
        V: 'static,
    {
        Self {
            codec_type_id: TypeId::of::<C>,
            codec_type_name: core::any::type_name::<C>,
            value_type_id: TypeId::of::<V>,
            value_type_name: core::any::type_name::<V>,
            encode: encode::<C, V>,
            decode: decode::<C, V>,
        }
    }

    /// Returns the process-local codec type identity.
    #[must_use]
    pub fn codec_type_id(&self) -> TypeId {
        (self.codec_type_id)()
    }

    /// Returns the diagnostic codec type name.
    #[must_use]
    pub fn codec_type_name(&self) -> &'static str {
        (self.codec_type_name)()
    }

    /// Returns the process-local value type identity.
    #[must_use]
    pub fn value_type_id(&self) -> TypeId {
        (self.value_type_id)()
    }

    /// Returns the diagnostic value type name.
    #[must_use]
    pub fn value_type_name(&self) -> &'static str {
        (self.value_type_name)()
    }

    /// Encodes one erased value after checking its concrete type.
    ///
    /// # Errors
    ///
    /// Returns a type mismatch or the typed encoder source error.
    pub fn encode(&self, value: &dyn Any) -> Result<Vec<u8>, ValueCodecExecutionError> {
        (self.encode)(value)
    }

    /// Decodes one byte slice into a safely erased value of the declared type.
    ///
    /// # Errors
    ///
    /// Returns the typed decoder source error.
    pub fn decode(&self, input: &[u8]) -> Result<Box<dyn Any>, ValueCodecExecutionError> {
        (self.decode)(input)
    }
}

impl core::fmt::Debug for ValueBytesCodecDescriptor {
    /// Formats descriptor metadata without invoking either codec function.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ValueBytesCodecDescriptor")
            .field("codec_type_name", &self.codec_type_name())
            .field("value_type_name", &self.value_type_name())
            .finish_non_exhaustive()
    }
}

/// Downcasts and encodes one typed value.
fn encode<C, V>(value: &dyn Any) -> Result<Vec<u8>, ValueCodecExecutionError>
where
    C: Default + ValueEncoder<V, Output = Vec<u8>> + ValueDecoder<[u8], Output = V> + 'static,
    <C as ValueEncoder<V>>::Error: std::error::Error + 'static,
    <C as ValueDecoder<[u8]>>::Error: std::error::Error + 'static,
    V: 'static,
{
    let value = value
        .downcast_ref::<V>()
        .ok_or_else(|| ValueCodecExecutionError::TypeMismatch {
            expected_type: core::any::type_name::<V>(),
            actual_type: value.type_id(),
        })?;
    C::default()
        .encode(value)
        .map_err(|source| ValueCodecExecutionError::EncodeFailed {
            codec_type: core::any::type_name::<C>(),
            source: Box::new(source),
        })
}

/// Decodes one byte slice and erases the typed output safely.
fn decode<C, V>(input: &[u8]) -> Result<Box<dyn Any>, ValueCodecExecutionError>
where
    C: Default + ValueEncoder<V, Output = Vec<u8>> + ValueDecoder<[u8], Output = V> + 'static,
    <C as ValueEncoder<V>>::Error: std::error::Error + 'static,
    <C as ValueDecoder<[u8]>>::Error: std::error::Error + 'static,
    V: 'static,
{
    C::default()
        .decode(input)
        .map(|value| Box::new(value) as Box<dyn Any>)
        .map_err(|source| ValueCodecExecutionError::DecodeFailed {
            codec_type: core::any::type_name::<C>(),
            source: Box::new(source),
        })
}
