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
mod value_decoder;
mod value_encoder;

pub use codec_value_decoder::CodecValueDecoder;
pub use codec_value_encoder::CodecValueEncoder;
pub use decode_lifecycle_output::DecodeLifecycleOutput;
pub use decode_lifecycle_progress::DecodeLifecycleProgress;
pub use value_decoder::ValueDecoder;
pub use value_encoder::ValueEncoder;
