// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

mod decode_context;
mod decode_incomplete_action;
mod decode_invalid_action;
mod decode_outcome;
mod encode_context;
mod encode_outcome;
mod encode_unencodable_action;
mod transcode_convert_engine;
mod transcode_decode_engine;
mod transcode_decode_hooks;
mod transcode_encode_engine;
mod transcode_encode_hooks;

pub use decode_context::DecodeContext;
pub use decode_incomplete_action::{
    DecodeIncompleteAction,
    DecodeIncompleteActionOf,
};
pub use decode_invalid_action::{
    DecodeInvalidAction,
    DecodeInvalidActionOf,
};
pub(crate) use decode_outcome::DecodeOutcome;
pub use encode_context::EncodeContext;
pub(crate) use encode_outcome::EncodeOutcome;
pub use encode_unencodable_action::{
    EncodeUnencodableAction,
    EncodeUnencodableActionOf,
};
pub use transcode_convert_engine::TranscodeConvertEngine;
pub use transcode_decode_engine::TranscodeDecodeEngine;
pub use transcode_decode_hooks::TranscodeDecodeHooks;
pub use transcode_encode_engine::TranscodeEncodeEngine;
pub use transcode_encode_hooks::TranscodeEncodeHooks;
