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
/// Registration macros create these values from codec descriptors and submit
/// them to the process-wide inventory. See
/// [`register_value_string_codec!`](crate::register_value_string_codec) and
/// [`register_value_bytes_codec!`](crate::register_value_bytes_codec) for
/// executable registration examples. For a local registry, this type can be
/// constructed directly from a validated ID, a static descriptor, and source
/// information as shown below.
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
