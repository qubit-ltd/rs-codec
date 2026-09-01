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
mod value_codec_descriptor;
mod value_codec_execution_error;
mod value_codec_id;
mod value_codec_id_error;
mod value_codec_registration;
mod value_codec_registration_factory;
mod value_codec_registration_source;
mod value_codec_registry;
mod value_codec_registry_error;
mod value_decoder;
mod value_encoder;

pub use codec_value_decoder::CodecValueDecoder;
pub use codec_value_encoder::CodecValueEncoder;
pub use decode_lifecycle_output::DecodeLifecycleOutput;
pub use decode_lifecycle_progress::DecodeLifecycleProgress;
pub use value_codec_descriptor::ValueCodecDescriptor;
pub use value_codec_execution_error::ValueCodecExecutionError;
pub use value_codec_id::ValueCodecId;
pub use value_codec_id_error::ValueCodecIdError;
pub use value_codec_registration::ValueCodecRegistration;
#[doc(hidden)]
pub use value_codec_registration_factory::ValueCodecRegistrationFactory;
pub use value_codec_registration_source::ValueCodecRegistrationSource;
pub use value_codec_registry::ValueCodecRegistry;
pub use value_codec_registry_error::ValueCodecRegistryError;
pub use value_decoder::ValueDecoder;
pub use value_encoder::ValueEncoder;
