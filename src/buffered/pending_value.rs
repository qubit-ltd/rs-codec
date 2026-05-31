/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Pending decoded value retained by buffered converters.

/// Decoded value retained after source input has been consumed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct PendingValue<Value> {
    /// Decoded logical value.
    pub(super) value: Value,
    /// Source input index that produced this value.
    pub(super) input_index: usize,
}
