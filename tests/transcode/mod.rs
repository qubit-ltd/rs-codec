// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

mod adapter;
mod capacity_error_tests;
mod engine;
mod internal;
#[cfg(feature = "io")]
mod io;
mod transcode_contract_error_tests;
mod transcode_convert_error_tests;
mod transcode_converter_tests;
mod transcode_decode_error_tests;
mod transcode_decoder_tests;
mod transcode_domain_error_tests;
mod transcode_encode_error_tests;
mod transcode_encoder_tests;
mod transcode_failure_tests;
mod transcode_progress_tests;
mod transcode_status_tests;
mod transcoder_tests;
