// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Low-level unchecked slice access helpers for hot paths.
//!
//! All helpers in this module avoid bound checks and are intended for call
//! sites that already validate index and range safety in their own protocol.

/// Returns whether a slice has at least `count` readable/writable units from
/// `start`.
///
/// # Parameters
///
/// - `len`: Slice length.
/// - `start`: Start index in the slice.
/// - `count`: Number of requested units after `start`.
///
/// # Returns
///
/// `true` if `start + count <= len` and no overflow occurs.
#[inline(always)]
pub const fn range_fits(len: usize, start: usize, count: usize) -> bool {
    match start.checked_add(count) {
        Some(end) => len >= end,
        None => false,
    }
}

/// Reads one value from an unchecked slice index.
///
/// # Parameters
///
/// - `input`: Source slice.
/// - `index`: Start index that must be valid for reading one unit.
///
/// # Safety
///
/// The caller must guarantee that `index < input.len()`.
#[inline(always)]
pub unsafe fn read_unchecked<T: Copy>(input: &[T], index: usize) -> T {
    // SAFETY: The caller guarantees that `index` is in-bounds.
    unsafe { *input.as_ptr().add(index) }
}

/// Writes one value to an unchecked mutable slice index.
///
/// # Parameters
///
/// - `output`: Destination slice.
/// - `index`: Start index that must be valid for writing one unit.
/// - `value`: Value to write.
///
/// # Safety
///
/// The caller must guarantee that `index < output.len()`.
#[inline(always)]
pub unsafe fn write_unchecked<T: Copy>(
    output: &mut [T],
    index: usize,
    value: T,
) {
    // SAFETY: The caller guarantees that `index` is in-bounds.
    unsafe {
        *output.as_mut_ptr().add(index) = value;
    }
}

/// Returns an immutable reference to one value at an unchecked slice index.
///
/// # Parameters
///
/// - `input`: Source slice.
/// - `index`: Start index that must be valid for reading one unit.
///
/// # Safety
///
/// The caller must guarantee that `index < input.len()`.
#[inline(always)]
pub unsafe fn ref_unchecked<T>(input: &[T], index: usize) -> &T {
    // SAFETY: The caller guarantees that `index` is in-bounds.
    unsafe { &*input.as_ptr().add(index) }
}

/// Returns a mutable reference to one value at an unchecked mutable slice
/// index.
///
/// # Parameters
///
/// - `output`: Destination slice.
/// - `index`: Start index that must be valid for writing one unit.
///
/// # Safety
///
/// The caller must guarantee that `index < output.len()`.
#[inline(always)]
pub unsafe fn mut_unchecked<T>(output: &mut [T], index: usize) -> &mut T {
    // SAFETY: The caller guarantees that `index` is in-bounds.
    unsafe { &mut *output.as_mut_ptr().add(index) }
}

/// Copies `count` values between unchecked slice offsets.
///
/// # Parameters
///
/// - `source`: Source slice.
/// - `source_index`: Source offset, must be valid for `count` units.
/// - `destination`: Destination slice.
/// - `destination_index`: Destination offset, must be valid for `count` units.
/// - `count`: Number of units to copy.
///
/// # Safety
///
/// The caller must guarantee that both source and destination ranges are valid
/// for `count` elements and the copy does not overflow pointer arithmetic.
#[inline(always)]
pub unsafe fn copy_nonoverlapping_unchecked<T: Copy>(
    source: &[T],
    source_index: usize,
    destination: &mut [T],
    destination_index: usize,
    count: usize,
) {
    // SAFETY: The caller guarantees both ranges are valid and non-overlapping.
    unsafe {
        core::ptr::copy_nonoverlapping(
            source.as_ptr().add(source_index),
            destination.as_mut_ptr().add(destination_index),
            count,
        );
    }
}

/// Reads one value from an unchecked unaligned byte slice offset.
///
/// # Parameters
///
/// - `input`: Source byte buffer.
/// - `index`: Byte offset in `input`.
///
/// # Safety
///
/// The caller must guarantee that `index` points to a valid unaligned region
/// capable of holding one `T`.
#[inline(always)]
pub unsafe fn read_ne_unaligned_unchecked<T: Copy>(
    input: &[u8],
    index: usize,
) -> T {
    // SAFETY: The caller guarantees byte-level validity for this unaligned
    // load.
    unsafe { core::ptr::read_unaligned(input.as_ptr().add(index).cast::<T>()) }
}

/// Writes one value to an unchecked unaligned byte slice offset.
///
/// # Parameters
///
/// - `output`: Destination byte buffer.
/// - `index`: Byte offset in `output`.
/// - `value`: Value to write.
///
/// # Safety
///
/// The caller must guarantee that `index` points to a valid unaligned region
/// capable of holding one `T`.
#[inline(always)]
pub unsafe fn write_ne_unaligned_unchecked<T: Copy>(
    output: &mut [u8],
    index: usize,
    value: T,
) {
    // SAFETY: The caller guarantees byte-level validity for this unaligned
    // store.
    unsafe {
        core::ptr::write_unaligned(
            output.as_mut_ptr().add(index).cast::<T>(),
            value,
        );
    }
}
