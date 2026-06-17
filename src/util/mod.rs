// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal utilities shared across the crate.

mod nz;
mod slice;

pub use nz::{
    nz,
    nz_const,
};
pub use slice::{
    copy_nonoverlapping_unchecked,
    mut_unchecked,
    range_fits,
    read_ne_unaligned_unchecked,
    read_unchecked,
    ref_unchecked,
    write_ne_unaligned_unchecked,
    write_unchecked,
};
