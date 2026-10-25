// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Codec-bound incomplete-decode action aliases.

use super::DecodeIncompleteAction;
use crate::Codec;

/// Incomplete-decode action for a codec-backed decode hook, using that codec's
/// logical value type for replacements.
///
/// This alias keeps the hook's action type tied to its codec, so replacement
/// values have the same type as values produced by decoding that codec.
///
/// # Type Parameters
///
/// - `C`: Codec whose logical value type is emitted by the decode hook.
pub type DecodeIncompleteActionOf<C> = DecodeIncompleteAction<<C as Codec>::Value>;
