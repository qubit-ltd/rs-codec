// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
use qubit_codec::{
    copy_nonoverlapping_unchecked, has_units, mut_unchecked, read_unchecked, ref_unchecked,
    required_index, write_unchecked,
};

#[test]
fn read_unchecked_reads_value() {
    let input = [1_u8, 2, 3];
    assert_eq!(unsafe { read_unchecked(&input, 1) }, 2);
}

#[test]
fn write_unchecked_writes_value() {
    let mut output = [1_u8, 2, 3];
    unsafe { write_unchecked(&mut output, 1, 9) };
    assert_eq!(output, [1, 9, 3]);
}

#[test]
fn ref_unchecked_returns_reference() {
    let input = [4_u16, 5, 6];
    assert_eq!(unsafe { *ref_unchecked(&input, 2) }, 6);
}

#[test]
fn mut_unchecked_writes_reference() {
    let mut output = [10_u32, 20, 30];
    unsafe {
        *mut_unchecked(&mut output, 0) = 12_345;
    }
    assert_eq!(output[0], 12_345);
}

#[test]
fn has_units_checks_range() {
    assert!(has_units(8, 2, 6));
    assert!(!has_units(8, 3, 6));
}

#[test]
fn required_index_handles_overflow() {
    assert_eq!(required_index(10, 2), 12);
    assert_eq!(required_index(usize::MAX, 1), usize::MAX);
}

#[test]
fn copy_nonoverlapping_unchecked_copies_slice() {
    let source = [1_u8, 2, 3, 4];
    let mut destination = [0_u8, 0, 0, 0];
    unsafe {
        copy_nonoverlapping_unchecked(&source, 0, &mut destination, 0, 4);
    }
    assert_eq!(destination, source);
}
