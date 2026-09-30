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
#[cfg(feature = "json")]
mod json_bytes_value_decoder_tests;
#[cfg(feature = "json")]
mod json_bytes_value_encoder_tests;
#[cfg(feature = "json")]
mod json_string_value_decoder_tests;
#[cfg(feature = "json")]
mod json_string_value_encoder_tests;
#[cfg(all(feature = "json", feature = "registry"))]
mod json_value_codec_registration_tests;
#[cfg(feature = "registry")]
mod value_codec_registry_tests;
mod value_decoder_tests;
mod value_encoder_tests;
