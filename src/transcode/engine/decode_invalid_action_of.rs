// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Codec-associated invalid-decode action type.

use super::DecodeInvalidAction;
use crate::Codec;

/// Invalid-decode action type used by a codec-backed decode hook.
///
/// This alias binds the action's replacement value to the codec's decoded
/// value type, so hook implementations can name their result in terms of the
/// codec they serve.
///
/// # Type Parameters
///
/// - `C`: Codec whose decoded value is carried by replacement actions.
pub type DecodeInvalidActionOf<C> = DecodeInvalidAction<<C as Codec>::Value>;
