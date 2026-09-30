// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Distributed value-codec registrations.

use crate::ValueBytesCodecDescriptor;
use crate::ValueCodecId;
use crate::ValueCodecRegistrationSource;
use crate::ValueStringCodecDescriptor;

/// One statically linked value-codec implementation.
#[derive(Clone, Copy, Debug)]
pub struct ValueCodecRegistration<D: 'static> {
    id: ValueCodecId,
    descriptor: &'static D,
    source: ValueCodecRegistrationSource,
}

/// A string-wire value-codec registration.
pub type ValueStringCodecRegistration = ValueCodecRegistration<ValueStringCodecDescriptor>;

/// A bytes-wire value-codec registration.
pub type ValueBytesCodecRegistration = ValueCodecRegistration<ValueBytesCodecDescriptor>;

impl<D: 'static> ValueCodecRegistration<D> {
    /// Creates a registration from validated static facts.
    #[doc(hidden)]
    #[must_use]
    pub const fn new(id: ValueCodecId, descriptor: &'static D, source: ValueCodecRegistrationSource) -> Self {
        Self { id, descriptor, source }
    }

    /// Returns the stable value-codec ID.
    #[must_use]
    pub const fn id(&self) -> ValueCodecId {
        self.id
    }

    /// Returns the executable descriptor.
    #[must_use]
    pub const fn descriptor(&self) -> &'static D {
        self.descriptor
    }

    /// Returns the linked source location.
    #[must_use]
    pub const fn source(&self) -> ValueCodecRegistrationSource {
        self.source
    }
}

/// Registers a default-constructible bidirectional string codec.
#[macro_export]
macro_rules! register_value_string_codec {
    (id = $id:literal, codec = $codec:ty, value = $value:ty $(,)?) => {
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
#[macro_export]
macro_rules! register_value_bytes_codec {
    (id = $id:literal, codec = $codec:ty, value = $value:ty $(,)?) => {
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
