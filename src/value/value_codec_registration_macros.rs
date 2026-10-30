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
/// ```compile_fail
/// use qubit_codec::JsonStringValueCodec;
/// use qubit_codec::register_value_string_codec;
///
/// fn invalid_id() -> &'static str { "example.u32" }
/// register_value_string_codec!(id = invalid_id(), codec = JsonStringValueCodec<u32>, value = u32);
/// ```
///
/// ```compile_fail
/// use qubit_codec::JsonStringValueCodec;
/// use qubit_codec::register_value_string_codec;
/// register_value_string_codec!(id = "invalid-id", codec = JsonStringValueCodec<u32>, value = u32);
/// ```
///
/// ```
/// # #[cfg(feature = "json")]
/// # {
/// use qubit_codec::JsonStringValueCodec;
/// use qubit_codec::register_value_string_codec;
///
/// register_value_string_codec!(
///     id = "example.u32",
///     codec = JsonStringValueCodec<u32>,
///     value = u32,
/// );
/// # }
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
/// ```compile_fail
/// use qubit_codec::JsonBytesValueCodec;
/// use qubit_codec::register_value_bytes_codec;
/// register_value_bytes_codec!(id = "invalid-id", codec = JsonBytesValueCodec<u32>, value = u32);
/// ```
///
/// ```
/// # #[cfg(feature = "json")]
/// # {
/// use qubit_codec::JsonBytesValueCodec;
/// use qubit_codec::register_value_bytes_codec;
///
/// register_value_bytes_codec!(
///     id = "example.u32",
///     codec = JsonBytesValueCodec<u32>,
///     value = u32,
/// );
/// # }
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
