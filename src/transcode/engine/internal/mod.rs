// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal attempt outcomes reported by the buffered transcode engines.

mod decode_outcome;
mod encode_outcome;

pub(crate) use decode_outcome::DecodeOutcome;
pub(crate) use encode_outcome::EncodeOutcome;
