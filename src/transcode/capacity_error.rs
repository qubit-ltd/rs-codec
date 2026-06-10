// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use thiserror::Error;

/// Error reported by output-capacity planning APIs.
///
/// Capacity planning is separate from actual transcoding. This error means the
/// requested upper bound cannot be represented as a `usize`; callers should
/// reject the one-shot allocation request or switch to chunked streaming.
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
pub enum CapacityError {
    /// The computed output length overflowed `usize`.
    #[error("output length overflow")]
    OutputLengthOverflow,
}
