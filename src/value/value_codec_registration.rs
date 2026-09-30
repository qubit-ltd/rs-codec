// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Distributed value-codec registrations.

use crate::ValueBytesCodecDescriptor;
use crate::ValueCodecId;
use crate::ValueCodecRegistrationSource;
use crate::ValueStringCodecDescriptor;

/// One statically linked value-codec implementation.
///
/// A registration bundles the stable identifier used for lookup and duplicate
/// detection, a borrowed executable descriptor, and the link-time location the
/// registration was declared at. It is produced by
/// [`register_value_string_codec!`](crate::register_value_string_codec) or
/// [`register_value_bytes_codec!`](crate::register_value_bytes_codec) rather
/// than built by hand.
///
/// # Type Parameters
///
/// - `D`: Executable codec descriptor type; `'static` lets the registry keep a
///   borrowed `&'static D` for the whole program run.
///
/// # Examples
///
/// ```
/// use qubit_codec::JsonStringValueCodec;
/// use qubit_codec::ValueCodecId;
/// use qubit_codec::ValueCodecRegistration;
/// use qubit_codec::ValueCodecRegistrationSource;
/// use qubit_codec::ValueStringCodecDescriptor;
/// use qubit_codec::ValueStringCodecRegistration;
///
/// static DESCRIPTOR: ValueStringCodecDescriptor =
///     ValueStringCodecDescriptor::of::<JsonStringValueCodec<u32>, u32>();
///
/// let registration: ValueStringCodecRegistration = ValueCodecRegistration::new(
///     ValueCodecId::new("example.u32"),
///     &DESCRIPTOR,
///     ValueCodecRegistrationSource::new(
///         "qubit-codec",
///         "doctest",
///         "src/value/value_codec_registration.rs",
///         1,
///     ),
/// );
/// assert_eq!(registration.id().as_str(), "example.u32");
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ValueCodecRegistration<D: 'static> {
    /// Stable identifier shared with registry lookups and duplicate detection.
    id: ValueCodecId,
    /// Executable descriptor borrowed from static storage.
    descriptor: &'static D,
    /// Link-time location the registration was declared at.
    source: ValueCodecRegistrationSource,
}

/// A string-wire value-codec registration.
pub type ValueStringCodecRegistration = ValueCodecRegistration<ValueStringCodecDescriptor>;

/// A bytes-wire value-codec registration.
pub type ValueBytesCodecRegistration = ValueCodecRegistration<ValueBytesCodecDescriptor>;

impl<D: 'static> ValueCodecRegistration<D> {
    /// Creates a registration from validated static facts.
    ///
    /// # Parameters
    ///
    /// - `id`: Stable identifier for registry lookups.
    /// - `descriptor`: Executable descriptor borrowed from static storage.
    /// - `source`: Link-time location recorded for diagnostics.
    ///
    /// # Returns
    ///
    /// Returns a registration that borrows `descriptor` for the whole run.
    ///
    /// Prefer [`register_value_string_codec!
    /// `](crate::register_value_string_codec)
    /// or [`register_value_bytes_codec!`](crate::register_value_bytes_codec)
    /// over calling this directly.
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub const fn new(id: ValueCodecId, descriptor: &'static D, source: ValueCodecRegistrationSource) -> Self {
        Self { id, descriptor, source }
    }

    /// Returns the stable value-codec ID.
    ///
    /// # Returns
    ///
    /// Returns the identifier this registration was filed under.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> ValueCodecId {
        self.id
    }

    /// Returns the executable descriptor.
    ///
    /// # Returns
    ///
    /// Returns the borrowed static descriptor, valid for the whole program run.
    #[inline]
    #[must_use]
    pub const fn descriptor(&self) -> &'static D {
        self.descriptor
    }

    /// Returns the linked source location.
    ///
    /// # Returns
    ///
    /// Returns the link-time crate, module, file, and line of the declaration.
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
///
/// # Examples
///
/// ```
/// use qubit_codec::JsonStringValueCodec;
/// use qubit_codec::register_value_string_codec;
///
/// register_value_string_codec!(
///     id = "example.u32",
///     codec = JsonStringValueCodec<u32>,
///     value = u32,
/// );
/// ```
#[macro_export]
macro_rules! register_value_string_codec {
    (id = $id:expr, codec = $codec:ty, value = $value:ty $(,)?) => {
        const _: () = {
            static DESCRIPTOR: $crate::ValueStringCodecDescriptor =
                $crate::ValueStringCodecDescriptor::of::<$codec, $value>();

            fn registration() -> $crate::ValueStringCodecRegistration {
                $crate::ValueCodecRegistration::new(
                    $crate::ValueCodecId::new($id),
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
///
/// # Examples
///
/// ```
/// use qubit_codec::JsonBytesValueCodec;
/// use qubit_codec::register_value_bytes_codec;
///
/// register_value_bytes_codec!(
///     id = "example.u32",
///     codec = JsonBytesValueCodec<u32>,
///     value = u32,
/// );
/// ```
#[macro_export]
macro_rules! register_value_bytes_codec {
    (id = $id:expr, codec = $codec:ty, value = $value:ty $(,)?) => {
        const _: () = {
            static DESCRIPTOR: $crate::ValueBytesCodecDescriptor =
                $crate::ValueBytesCodecDescriptor::of::<$codec, $value>();

            fn registration() -> $crate::ValueBytesCodecRegistration {
                $crate::ValueCodecRegistration::new(
                    $crate::ValueCodecId::new($id),
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
