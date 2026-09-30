// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

mod codec_value_decoder_tests;
mod codec_value_encoder_tests;
mod codec_value_lifecycle_tests;
mod decode_lifecycle_output_tests;
mod decode_lifecycle_progress_tests;
mod internal;
#[cfg(feature = "json")]
mod json_bytes_value_codec_tests;
#[cfg(feature = "json")]
mod json_bytes_value_decoder_tests;
#[cfg(feature = "json")]
mod json_bytes_value_encoder_tests;
#[cfg(feature = "json")]
mod json_string_value_codec_tests;
#[cfg(feature = "json")]
mod json_string_value_decoder_tests;
#[cfg(feature = "json")]
mod json_string_value_encoder_tests;
#[cfg(all(feature = "json", feature = "registry"))]
mod json_value_codec_registration_tests;
#[cfg(feature = "registry")]
mod value_bytes_codec_descriptor_tests;
#[cfg(feature = "registry")]
mod value_bytes_codec_registration_factory_tests;
#[cfg(feature = "registry")]
mod value_codec_execution_error_tests;
#[cfg(feature = "registry")]
mod value_codec_id_error_tests;
#[cfg(feature = "registry")]
mod value_codec_id_tests;
#[cfg(feature = "registry")]
mod value_codec_registration_source_tests;
#[cfg(feature = "registry")]
mod value_codec_registration_tests;
#[cfg(feature = "registry")]
mod value_codec_registry_error_tests;
#[cfg(feature = "registry")]
mod value_codec_registry_tests;
mod value_decoder_tests;
mod value_encoder_tests;
#[cfg(feature = "registry")]
mod value_string_codec_descriptor_tests;
#[cfg(feature = "registry")]
mod value_string_codec_registration_factory_tests;
