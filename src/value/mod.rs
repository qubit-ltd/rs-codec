// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Owned value conversion traits and adapters.

mod codec_value_decoder;
mod codec_value_encoder;
pub(crate) mod codec_value_lifecycle;
mod decode_lifecycle_output;
mod decode_lifecycle_progress;
#[cfg(feature = "registry")]
mod value_codec_descriptor;
#[cfg(feature = "registry")]
mod value_codec_execution_error;
#[cfg(feature = "registry")]
mod value_codec_id;
#[cfg(feature = "registry")]
mod value_codec_id_error;
#[cfg(feature = "registry")]
mod value_codec_registration;
#[cfg(feature = "registry")]
mod value_codec_registration_factory;
#[cfg(feature = "registry")]
mod value_codec_registration_source;
#[cfg(feature = "registry")]
mod value_codec_registry;
#[cfg(feature = "registry")]
mod value_codec_registry_error;
mod value_decoder;
mod value_encoder;

pub use codec_value_decoder::CodecValueDecoder;
pub use codec_value_encoder::CodecValueEncoder;
pub use decode_lifecycle_output::DecodeLifecycleOutput;
pub use decode_lifecycle_progress::DecodeLifecycleProgress;
#[cfg(feature = "registry")]
pub use value_codec_descriptor::ValueCodecDescriptor;
#[cfg(feature = "registry")]
pub use value_codec_execution_error::ValueCodecExecutionError;
#[cfg(feature = "registry")]
pub use value_codec_id::ValueCodecId;
#[cfg(feature = "registry")]
pub use value_codec_id_error::ValueCodecIdError;
#[cfg(feature = "registry")]
pub use value_codec_registration::ValueCodecRegistration;
#[cfg(feature = "registry")]
#[doc(hidden)]
pub use value_codec_registration_factory::ValueCodecRegistrationFactory;
#[cfg(feature = "registry")]
pub use value_codec_registration_source::ValueCodecRegistrationSource;
#[cfg(feature = "registry")]
pub use value_codec_registry::ValueCodecRegistry;
#[cfg(feature = "registry")]
pub use value_codec_registry_error::ValueCodecRegistryError;
pub use value_decoder::ValueDecoder;
pub use value_encoder::ValueEncoder;
