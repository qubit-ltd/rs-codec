// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Registration type for string-wire value codecs.

use crate::ValueCodecRegistration;
use crate::ValueStringCodecDescriptor;

/// A registration for a value codec that encodes to and decodes from strings.
///
/// # Examples
///
/// ```
/// # #[cfg(all(feature = "json", feature = "registry"))]
/// # {
/// use qubit_codec::JsonStringValueCodec;
/// use qubit_codec::ValueCodecId;
/// use qubit_codec::ValueCodecRegistration;
/// use qubit_codec::ValueCodecRegistrationSource;
/// use qubit_codec::ValueStringCodecDescriptor;
/// use qubit_codec::ValueStringCodecRegistration;
///
/// static DESCRIPTOR: ValueStringCodecDescriptor =
///     ValueStringCodecDescriptor::of::<JsonStringValueCodec<u32>, u32>();
/// let registration: ValueStringCodecRegistration = ValueCodecRegistration::new(
///     ValueCodecId::new("example.u32"),
///     &DESCRIPTOR,
///     ValueCodecRegistrationSource::new("qubit-codec", "doctest", "src/example.rs", 1),
/// );
/// assert_eq!(registration.id().as_str(), "example.u32");
/// # }
/// ```
pub type ValueStringCodecRegistration = ValueCodecRegistration<ValueStringCodecDescriptor>;
