// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Distributed value-codec registrations.

use crate::ValueCodecDescriptor;
use crate::ValueCodecId;
use crate::ValueCodecRegistrationSource;

/// One statically linked value-codec implementation.
#[derive(Clone, Copy, Debug)]
pub struct ValueCodecRegistration {
    id: ValueCodecId,
    descriptor: &'static ValueCodecDescriptor,
    source: ValueCodecRegistrationSource,
}

impl ValueCodecRegistration {
    /// Creates a registration from validated static facts.
    #[doc(hidden)]
    #[must_use]
    pub const fn new(
        id: ValueCodecId,
        descriptor: &'static ValueCodecDescriptor,
        source: ValueCodecRegistrationSource,
    ) -> Self {
        Self { id, descriptor, source }
    }

    /// Returns the stable value-codec ID.
    #[must_use]
    pub const fn id(&self) -> ValueCodecId {
        self.id
    }

    /// Returns the executable descriptor.
    #[must_use]
    pub const fn descriptor(&self) -> &'static ValueCodecDescriptor {
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
macro_rules! register_value_codec {
    (id = $id:literal, codec = $codec:ty, value = $value:ty $(,)?) => {
        const _: () = {
            static DESCRIPTOR: $crate::ValueCodecDescriptor = $crate::ValueCodecDescriptor::of::<$codec, $value>();

            fn registration() -> $crate::ValueCodecRegistration {
                $crate::ValueCodecRegistration::new(
                    $crate::ValueCodecId::new($id),
                    &DESCRIPTOR,
                    $crate::ValueCodecRegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
                )
            }

            $crate::__private::inventory::submit! {
                $crate::ValueCodecRegistrationFactory(registration)
            }
        };
    };
}
