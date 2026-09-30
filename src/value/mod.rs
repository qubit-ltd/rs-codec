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
#[cfg(feature = "json")]
mod json_bytes_value_codec;
#[cfg(feature = "json")]
mod json_bytes_value_decoder;
#[cfg(feature = "json")]
mod json_bytes_value_encoder;
#[cfg(feature = "json")]
mod json_string_value_codec;
#[cfg(feature = "json")]
mod json_string_value_decoder;
#[cfg(feature = "json")]
mod json_string_value_encoder;
#[cfg(all(feature = "json", feature = "registry"))]
mod json_value_codec_registration;
#[cfg(feature = "registry")]
mod value_bytes_codec_descriptor;
#[cfg(feature = "registry")]
mod value_bytes_codec_registration_factory;
#[cfg(feature = "registry")]
mod value_codec_execution_error;
#[cfg(feature = "registry")]
mod value_codec_id;
#[cfg(feature = "registry")]
mod value_codec_id_error;
#[cfg(feature = "registry")]
mod value_codec_registration;
#[cfg(feature = "registry")]
mod value_codec_registration_source;
#[cfg(feature = "registry")]
mod value_codec_registry;
#[cfg(feature = "registry")]
mod value_codec_registry_error;
mod value_decoder;
mod value_encoder;
#[cfg(feature = "registry")]
mod value_string_codec_descriptor;
#[cfg(feature = "registry")]
mod value_string_codec_registration_factory;

pub use codec_value_decoder::CodecValueDecoder;
pub use codec_value_encoder::CodecValueEncoder;
pub use decode_lifecycle_output::DecodeLifecycleOutput;
pub use decode_lifecycle_progress::DecodeLifecycleProgress;
#[cfg(feature = "json")]
pub use json_bytes_value_codec::JsonBytesValueCodec;
#[cfg(feature = "json")]
pub use json_bytes_value_decoder::JsonBytesValueDecoder;
#[cfg(feature = "json")]
pub use json_bytes_value_encoder::JsonBytesValueEncoder;
#[cfg(feature = "json")]
pub use json_string_value_codec::JsonStringValueCodec;
#[cfg(feature = "json")]
pub use json_string_value_decoder::JsonStringValueDecoder;
#[cfg(feature = "json")]
pub use json_string_value_encoder::JsonStringValueEncoder;
#[cfg(all(feature = "json", feature = "registry"))]
pub use json_value_codec_registration::JSON_BYTES_VALUE_CODEC_ID;
#[cfg(all(feature = "json", feature = "registry"))]
pub use json_value_codec_registration::JSON_STRING_VALUE_CODEC_ID;
#[cfg(feature = "registry")]
pub use value_bytes_codec_descriptor::ValueBytesCodecDescriptor;
#[cfg(feature = "registry")]
#[doc(hidden)]
pub use value_bytes_codec_registration_factory::ValueBytesCodecRegistrationFactory;
#[cfg(feature = "registry")]
pub use value_codec_execution_error::ValueCodecExecutionError;
#[cfg(feature = "registry")]
pub use value_codec_id::ValueCodecId;
#[cfg(feature = "registry")]
pub use value_codec_id_error::ValueCodecIdError;
#[cfg(feature = "registry")]
pub use value_codec_registration::ValueBytesCodecRegistration;
#[cfg(feature = "registry")]
pub use value_codec_registration::ValueCodecRegistration;
#[cfg(feature = "registry")]
pub use value_codec_registration::ValueStringCodecRegistration;
#[cfg(feature = "registry")]
pub use value_codec_registration_source::ValueCodecRegistrationSource;
#[cfg(feature = "registry")]
pub use value_codec_registry::ValueBytesCodecRegistry;
#[cfg(feature = "registry")]
pub use value_codec_registry::ValueCodecRegistry;
#[cfg(feature = "registry")]
pub use value_codec_registry::ValueStringCodecRegistry;
#[cfg(feature = "registry")]
pub use value_codec_registry_error::ValueCodecRegistryError;
pub use value_decoder::ValueDecoder;
pub use value_encoder::ValueEncoder;
#[cfg(feature = "registry")]
pub use value_string_codec_descriptor::ValueStringCodecDescriptor;
#[cfg(feature = "registry")]
#[doc(hidden)]
pub use value_string_codec_registration_factory::ValueStringCodecRegistrationFactory;
