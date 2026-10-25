// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Registration type for bytes-wire value codecs.

use crate::ValueBytesCodecDescriptor;
use crate::ValueCodecRegistration;

/// A registration for a value codec that encodes to and decodes from bytes.
///
/// # Examples
///
/// ```
/// # #[cfg(all(feature = "json", feature = "registry"))]
/// # {
/// use qubit_codec::JsonBytesValueCodec;
/// use qubit_codec::ValueBytesCodecDescriptor;
/// use qubit_codec::ValueBytesCodecRegistration;
/// use qubit_codec::ValueCodecId;
/// use qubit_codec::ValueCodecRegistration;
/// use qubit_codec::ValueCodecRegistrationSource;
///
/// static DESCRIPTOR: ValueBytesCodecDescriptor =
///     ValueBytesCodecDescriptor::of::<JsonBytesValueCodec<u32>, u32>();
/// let registration: ValueBytesCodecRegistration = ValueCodecRegistration::new(
///     ValueCodecId::new("example.u32"),
///     &DESCRIPTOR,
///     ValueCodecRegistrationSource::new("qubit-codec", "doctest", "src/example.rs", 1),
/// );
/// assert_eq!(registration.id().as_str(), "example.u32");
/// # }
/// ```
pub type ValueBytesCodecRegistration = ValueCodecRegistration<ValueBytesCodecDescriptor>;
