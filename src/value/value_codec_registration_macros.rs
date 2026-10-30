// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Distributed value-codec registrations.

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
/// A registry-only example with a local codec:
///
/// ```
/// use core::convert::Infallible;
/// use qubit_codec::{ValueDecoder, ValueEncoder, ValueStringCodecRegistry};
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
/// register_value_string_codec!(id = "example.text", codec = TextCodec, value = String);
/// let registry = ValueStringCodecRegistry::try_global().unwrap();
/// let descriptor = registry.get("example.text").unwrap().descriptor();
/// let encoded = descriptor.encode(&String::from("hello")).unwrap();
/// assert_eq!("hello", encoded);
/// let decoded = descriptor.decode(&encoded).unwrap();
/// assert_eq!("hello", *decoded.downcast::<String>().unwrap());
/// ```
///
/// An invalid ID fails during macro expansion:
///
/// ```compile_fail
/// # use core::convert::Infallible;
/// # use qubit_codec::{ValueDecoder, ValueEncoder};
/// # use qubit_codec::register_value_string_codec;
/// # #[derive(Default)] struct TextCodec;
/// # impl ValueEncoder<String> for TextCodec { type Output = String; type Error = Infallible; fn encode(&mut self, value: &String) -> Result<String, Infallible> { Ok(value.clone()) } }
/// # impl ValueDecoder<str> for TextCodec { type Output = String; type Error = Infallible; fn decode(&mut self, input: &str) -> Result<String, Infallible> { Ok(input.to_owned()) } }
/// register_value_string_codec!(id = "invalid-id", codec = TextCodec, value = String);
/// ```
///
/// A runtime ID cannot initialize the static registration:
///
/// ```compile_fail
/// # use core::convert::Infallible;
/// # use qubit_codec::{ValueDecoder, ValueEncoder};
/// # use qubit_codec::register_value_string_codec;
/// # #[derive(Default)] struct TextCodec;
/// # impl ValueEncoder<String> for TextCodec { type Output = String; type Error = Infallible; fn encode(&mut self, value: &String) -> Result<String, Infallible> { Ok(value.clone()) } }
/// # impl ValueDecoder<str> for TextCodec { type Output = String; type Error = Infallible; fn decode(&mut self, input: &str) -> Result<String, Infallible> { Ok(input.to_owned()) } }
/// fn runtime_id() -> &'static str { "example.text" }
/// register_value_string_codec!(id = runtime_id(), codec = TextCodec, value = String);
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
/// A registry-only example with a local codec:
///
/// ```
/// use core::convert::Infallible;
/// use qubit_codec::{ValueBytesCodecRegistry, ValueDecoder, ValueEncoder};
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
/// let registry = ValueBytesCodecRegistry::try_global().unwrap();
/// let descriptor = registry.get("example.bytes").unwrap().descriptor();
/// let encoded = descriptor.encode(&vec![1_u8, 2]).unwrap();
/// assert_eq!(vec![1_u8, 2], encoded);
/// let decoded = descriptor.decode(&encoded).unwrap();
/// assert_eq!(vec![1_u8, 2], *decoded.downcast::<Vec<u8>>().unwrap());
/// ```
///
/// An invalid ID fails during macro expansion:
///
/// ```compile_fail
/// # use core::convert::Infallible;
/// # use qubit_codec::{ValueDecoder, ValueEncoder};
/// # use qubit_codec::register_value_bytes_codec;
/// # #[derive(Default)] struct BytesCodec;
/// # impl ValueEncoder<Vec<u8>> for BytesCodec { type Output = Vec<u8>; type Error = Infallible; fn encode(&mut self, value: &Vec<u8>) -> Result<Vec<u8>, Infallible> { Ok(value.clone()) } }
/// # impl ValueDecoder<[u8]> for BytesCodec { type Output = Vec<u8>; type Error = Infallible; fn decode(&mut self, input: &[u8]) -> Result<Vec<u8>, Infallible> { Ok(input.to_vec()) } }
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
