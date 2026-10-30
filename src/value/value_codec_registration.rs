// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Distributed value-codec registrations.

use crate::ValueCodecId;
use crate::ValueCodecRegistrationSource;

/// One statically linked value-codec implementation.
///
/// A registration bundles the stable identifier used for lookup and duplicate
/// detection, a borrowed executable descriptor, and the link-time location the
/// registration was declared at. It is produced by
/// [`register_value_string_codec!`](crate::register_value_string_codec) or
/// [`register_value_bytes_codec!`](crate::register_value_bytes_codec)
/// rather than built by hand.
///
/// The registration macros validate codecs through the matching descriptor.
/// Encoder and decoder error types must implement `Error + Send + Sync +
/// 'static`; codec and value types do not need those bounds.
#[derive(Clone, Copy, Debug)]
pub struct ValueCodecRegistration<D: 'static> {
    /// Stable identifier shared with registry lookups and duplicate detection.
    id: ValueCodecId,
    /// Executable descriptor borrowed from static storage.
    descriptor: &'static D,
    /// Link-time location the registration was declared at.
    source: ValueCodecRegistrationSource,
}

impl<D: 'static> ValueCodecRegistration<D> {
    /// Creates a registration from validated static facts.
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub const fn new(id: ValueCodecId, descriptor: &'static D, source: ValueCodecRegistrationSource) -> Self {
        Self { id, descriptor, source }
    }

    /// Returns the stable value-codec ID.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> ValueCodecId {
        self.id
    }

    /// Returns the executable descriptor.
    #[inline]
    #[must_use]
    pub const fn descriptor(&self) -> &'static D {
        self.descriptor
    }

    /// Returns the linked source location.
    #[inline]
    #[must_use]
    pub const fn source(&self) -> ValueCodecRegistrationSource {
        self.source
    }
}

/// Registers a default-constructible bidirectional string codec.
///
/// The macro must be invoked at module scope; it defines the descriptor, a
/// registration function, and an inventory submission in an anonymous const.
/// The `id` must be a constant expression. Use a literal, constant, or
/// `const fn` result; a runtime function call cannot define a static ID.
/// The encoder and decoder error types must implement `Error + Send + Sync +
/// 'static`, as required by [`crate::ValueStringCodecDescriptor::of`].
/// Without `input`, the encoder input type is `value`. The optional `input`
/// form selects a borrowed view of the owned `value` type for encoding; the
/// descriptor still accepts and decodes the owned value type.
///
/// # Examples
///
/// ```
/// use core::convert::Infallible;
/// use qubit_codec::ValueDecoder;
/// use qubit_codec::ValueEncoder;
/// use qubit_codec::register_value_string_codec;
///
/// #[derive(Default)]
/// struct TextCodec;
/// impl ValueEncoder<String> for TextCodec {
///     type Output = String;
///     type Error = Infallible;
///     fn encode(&mut self, value: &String) -> Result<String, Infallible> {
///         Ok(value.clone())
///     }
/// }
/// impl ValueDecoder<str> for TextCodec {
///     type Output = String;
///     type Error = Infallible;
///     fn decode(&mut self, input: &str) -> Result<String, Infallible> {
///         Ok(input.to_owned())
///     }
/// }
/// register_value_string_codec!(id = "example.text", codec = TextCodec, value = String);
/// ```
///
/// ```compile_fail
/// use core::convert::Infallible;
/// use qubit_codec::ValueDecoder;
/// use qubit_codec::ValueEncoder;
/// use qubit_codec::register_value_string_codec;
///
/// #[derive(Default)]
/// struct TextCodec;
/// impl ValueEncoder<String> for TextCodec {
///     type Output = String;
///     type Error = Infallible;
///     fn encode(&mut self, value: &String) -> Result<String, Infallible> { Ok(value.clone()) }
/// }
/// impl ValueDecoder<str> for TextCodec {
///     type Output = String;
///     type Error = Infallible;
///     fn decode(&mut self, input: &str) -> Result<String, Infallible> { Ok(input.to_owned()) }
/// }
/// register_value_string_codec!(id = "invalid-id", codec = TextCodec, value = String);
/// ```
///
/// ```compile_fail
/// use core::convert::Infallible;
/// use qubit_codec::ValueDecoder;
/// use qubit_codec::ValueEncoder;
/// use qubit_codec::register_value_string_codec;
///
/// #[derive(Default)]
/// struct TextCodec;
/// impl ValueEncoder<String> for TextCodec {
///     type Output = String;
///     type Error = Infallible;
///     fn encode(&mut self, value: &String) -> Result<String, Infallible> { Ok(value.clone()) }
/// }
/// impl ValueDecoder<str> for TextCodec {
///     type Output = String;
///     type Error = Infallible;
///     fn decode(&mut self, input: &str) -> Result<String, Infallible> { Ok(input.to_owned()) }
/// }
/// fn invalid_id() -> &'static str { "example.text" }
/// register_value_string_codec!(id = invalid_id(), codec = TextCodec, value = String);
/// ```
#[macro_export]
macro_rules! register_value_string_codec {
    (id = $id:expr, codec = $codec:ty, value = $value:ty, input = $input:ty $(,)?) => {
        const _: () = {
            const ID: $crate::ValueCodecId = $crate::ValueCodecId::new($id);
            static DESCRIPTOR: $crate::ValueStringCodecDescriptor =
                $crate::ValueStringCodecDescriptor::of_borrowed::<$codec, $value, $input>();
            fn registration() -> $crate::ValueStringCodecRegistration {
                $crate::ValueCodecRegistration::new(
                    ID,
                    &DESCRIPTOR,
                    $crate::ValueCodecRegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
                )
            }
            $crate::__private::inventory::submit! {
                $crate::ValueStringCodecRegistrationFactory(registration)
            }
        };
    };
    (id = $id:expr, codec = $codec:ty, value = $value:ty $(,)?) => {
        const _: () = {
            const ID: $crate::ValueCodecId = $crate::ValueCodecId::new($id);
            static DESCRIPTOR: $crate::ValueStringCodecDescriptor =
                $crate::ValueStringCodecDescriptor::of::<$codec, $value>();

            fn registration() -> $crate::ValueStringCodecRegistration {
                $crate::ValueCodecRegistration::new(
                    ID,
                    &DESCRIPTOR,
                    $crate::ValueCodecRegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
                )
            }

            $crate::__private::inventory::submit! {
                $crate::ValueStringCodecRegistrationFactory(registration)
            }
        };
    };
}

/// Registers a default-constructible bidirectional bytes codec.
///
/// The macro must be invoked at module scope; it defines the descriptor, a
/// registration function, and an inventory submission in an anonymous const.
/// The `id` must be a constant expression. Use a literal, constant, or
/// `const fn` result; a runtime function call cannot define a static ID.
/// The encoder and decoder error types must implement `Error + Send + Sync +
/// 'static`, as required by [`crate::ValueBytesCodecDescriptor::of`].
///
/// # Examples
///
/// ```
/// use core::convert::Infallible;
/// use qubit_codec::ValueDecoder;
/// use qubit_codec::ValueEncoder;
/// use qubit_codec::register_value_bytes_codec;
///
/// #[derive(Default)]
/// struct BytesCodec;
/// impl ValueEncoder<Vec<u8>> for BytesCodec {
///     type Output = Vec<u8>;
///     type Error = Infallible;
///     fn encode(&mut self, value: &Vec<u8>) -> Result<Vec<u8>, Infallible> { Ok(value.clone()) }
/// }
/// impl ValueDecoder<[u8]> for BytesCodec {
///     type Output = Vec<u8>;
///     type Error = Infallible;
///     fn decode(&mut self, input: &[u8]) -> Result<Vec<u8>, Infallible> { Ok(input.to_vec()) }
/// }
/// register_value_bytes_codec!(id = "example.bytes", codec = BytesCodec, value = Vec<u8>);
/// ```
///
/// ```compile_fail
/// use core::convert::Infallible;
/// use qubit_codec::ValueDecoder;
/// use qubit_codec::ValueEncoder;
/// use qubit_codec::register_value_bytes_codec;
///
/// #[derive(Default)]
/// struct BytesCodec;
/// impl ValueEncoder<Vec<u8>> for BytesCodec {
///     type Output = Vec<u8>;
///     type Error = Infallible;
///     fn encode(&mut self, value: &Vec<u8>) -> Result<Vec<u8>, Infallible> { Ok(value.clone()) }
/// }
/// impl ValueDecoder<[u8]> for BytesCodec {
///     type Output = Vec<u8>;
///     type Error = Infallible;
///     fn decode(&mut self, input: &[u8]) -> Result<Vec<u8>, Infallible> { Ok(input.to_vec()) }
/// }
/// register_value_bytes_codec!(id = "invalid-id", codec = BytesCodec, value = Vec<u8>);
/// ```
#[macro_export]
macro_rules! register_value_bytes_codec {
    (id = $id:expr, codec = $codec:ty, value = $value:ty $(,)?) => {
        const _: () = {
            const ID: $crate::ValueCodecId = $crate::ValueCodecId::new($id);
            static DESCRIPTOR: $crate::ValueBytesCodecDescriptor =
                $crate::ValueBytesCodecDescriptor::of::<$codec, $value>();

            fn registration() -> $crate::ValueBytesCodecRegistration {
                $crate::ValueCodecRegistration::new(
                    ID,
                    &DESCRIPTOR,
                    $crate::ValueCodecRegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
                )
            }

            $crate::__private::inventory::submit! {
                $crate::ValueBytesCodecRegistrationFactory(registration)
            }
        };
    };
}
