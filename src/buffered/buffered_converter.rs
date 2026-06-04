/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Semantic marker trait for buffered converters.

use super::BufferedTranscoder;

/// Converts encoded units of one representation into encoded units of another.
///
/// `BufferedConverter` refines [`BufferedTranscoder`] for implementations whose input
/// and output are both encoded unit streams. Any intermediate logical values
/// are implementation details of the concrete converter.
///
/// The trait adds no methods. It exists to make generic bounds distinguish
/// unit-to-unit conversion from value-to-unit encoding and unit-to-value
/// decoding.
///
/// # Type Parameters
///
/// - `InputUnit`: Encoded input unit type accepted by the converter.
/// - `OutputUnit`: Encoded output unit type produced by the converter.
pub trait BufferedConverter<InputUnit, OutputUnit>: BufferedTranscoder<InputUnit, OutputUnit> {}
