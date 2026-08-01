// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal state machines and step types for transcode engines.

pub(in crate::transcode) mod applied_decode_invalid_action;
pub(in crate::transcode) mod convert_state;
pub(in crate::transcode) mod decode_state;
pub(in crate::transcode) mod encode_attempt;
pub(in crate::transcode) mod encode_state;
pub(in crate::transcode) mod lifecycle_guard;
pub(in crate::transcode) mod lifecycle_phase;
pub(in crate::transcode) mod pending_value;
pub(in crate::transcode) mod pending_value_slot;
pub(in crate::transcode) mod transcode_state;
