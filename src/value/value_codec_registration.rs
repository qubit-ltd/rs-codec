// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Registration type definitions are colocated with their public module.

use crate::ValueCodecId;
use crate::ValueCodecRegistrationSource;

/// One statically linked value-codec implementation.
///
/// A registration bundles the stable identifier used for lookup and duplicate
/// detection, a borrowed executable descriptor, and the link-time location the
/// registration was declared at. It is produced by
/// [`register_value_string_codec!`](crate::register_value_string_codec) or
/// [`register_value_bytes_codec!`](crate::register_value_bytes_codec)
/// rather than built by hand. Applications may also construct a registration
/// manually from static data and pass it to a local
/// [`ValueCodecRegistry`](crate::ValueCodecRegistry).
///
/// The ID has already been validated by [`ValueCodecId`]. The descriptor must
/// live in static storage because registrations borrow it with a `'static`
/// lifetime. The source records where the registration came from for duplicate
/// ID diagnostics; it does not affect those static requirements.
///
/// The registration macros validate codecs through the matching descriptor.
/// Encoder and decoder error types must implement `Error + Send + Sync +
/// 'static`; codec and value types do not need those bounds.
///
/// # Registration Macro Examples
///
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
/// # Type Parameters
///
/// - `D`: Static descriptor type that executes the registered codec.
///
/// # Examples
///
/// ```
/// use qubit_codec::ValueCodecId;
/// use qubit_codec::ValueCodecRegistration;
/// use qubit_codec::ValueCodecRegistrationSource;
///
/// static DESCRIPTOR: u8 = 1;
/// let registration = ValueCodecRegistration::new(
///     ValueCodecId::new("example.u8"),
///     &DESCRIPTOR,
///     ValueCodecRegistrationSource::new("qubit-codec", "example", "src/example.rs", 1),
/// );
/// assert_eq!(registration.id().as_str(), "example.u8");
/// assert_eq!(*registration.descriptor(), 1);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ValueCodecRegistration<D: 'static> {
    /// Stable identifier used by registry lookups and duplicate detection.
    id: ValueCodecId,
    /// Executable descriptor borrowed from static storage for the registration
    /// lifetime.
    descriptor: &'static D,
    /// Link-time location identifying where the registration macro was invoked.
    source: ValueCodecRegistrationSource,
}
impl<D: 'static> ValueCodecRegistration<D> {
    /// Creates a registration from the validated facts captured by a
    /// registration macro.
    ///
    /// Use the registration macros to submit a codec to the process-wide
    /// inventory. For a local registry, construct this value from static data
    /// and pass a reference to
    /// [`ValueCodecRegistry::from_registrations`](crate::ValueCodecRegistry::from_registrations).
    /// The ID must already be validated, and the descriptor must have static
    /// storage. The source provides location details for duplicate-ID
    /// diagnostics.
    ///
    /// # Parameters
    ///
    /// - `id`: Stable identifier used for registry lookup and duplicate
    ///   detection.
    /// - `descriptor`: Descriptor in static storage that performs codec
    ///   operations.
    /// - `source`: Declaration location captured by the registration macro.
    ///
    /// # Returns
    ///
    /// A copyable registration borrowing the static descriptor.
    #[inline]
    #[must_use]
    pub const fn new(id: ValueCodecId, descriptor: &'static D, source: ValueCodecRegistrationSource) -> Self {
        Self { id, descriptor, source }
    }

    /// Returns the stable value-codec ID used by registry lookups.
    ///
    /// # Returns
    ///
    /// The registration's validated, copyable identifier.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> ValueCodecId {
        self.id
    }

    /// Returns the descriptor used to execute this codec.
    ///
    /// # Returns
    ///
    /// A static borrow of the descriptor associated with this registration.
    #[inline]
    #[must_use = "the codec descriptor is needed to perform codec operations"]
    pub const fn descriptor(&self) -> &'static D {
        self.descriptor
    }

    /// Returns the source location captured at the registration declaration.
    ///
    /// # Returns
    ///
    /// The copyable package, module, file, and line information for this
    /// registration.
    #[inline]
    #[must_use]
    pub const fn source(&self) -> ValueCodecRegistrationSource {
        self.source
    }
}
