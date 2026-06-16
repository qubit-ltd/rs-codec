// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::num::NonZeroUsize;

pub(crate) fn nz(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).expect("test additional count must be non-zero")
}
