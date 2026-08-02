// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private driver for decoding one codec value from buffered input.

use core::num::NonZeroUsize;
use std::io::{Error, ErrorKind, Result};

use qubit_io::{BufferedInput, Input};

use crate::{Codec, DecodeFailure};

/// Drives one codec decode operation against persistent buffered input.
pub(super) struct CodecDecodeDriver<'a, I>
where
    I: Input,
    I::Item: Copy + Default,
{
    /// Buffered unit input shared with the public adapter.
    input: &'a mut BufferedInput<I>,
}

impl<'a, I> CodecDecodeDriver<'a, I>
where
    I: Input,
    I::Item: Copy + Default,
{
    /// Creates a one-value decode driver over `input`.
    #[inline(always)]
    pub(super) const fn new(input: &'a mut BufferedInput<I>) -> Self {
        Self { input }
    }

    /// Reads one decoded value after codec lifecycle reset completed.
    pub(super) fn read_one<C, M>(&mut self, codec: &mut C, map_error: &mut M) -> Result<C::Value>
    where
        C: Codec<Unit = I::Item>,
        M: FnMut(C::DecodeError) -> Error,
    {
        let min_units_per_value = C::MIN_UNITS_PER_VALUE;
        let max_units_per_value = C::MAX_DECODE_UNITS_PER_VALUE.max(min_units_per_value);
        self.input
            .try_reserve_capacity(min_units_per_value)
            .map_err(|error| Error::new(ErrorKind::OutOfMemory, error))?;

        loop {
            let available =
                self.prepare_buffered_window(min_units_per_value, max_units_per_value)?;
            let units = &self.input.unread()[..available];
            debug_assert!(units.len() >= min_units_per_value);
            let decode_result = unsafe {
                // SAFETY: `min_units_per_value <= units.len()` guarantees
                // `decode` preconditions for this slice.
                codec.decode(units, 0)
            };
            match decode_result {
                Ok((value, consumed)) => {
                    return self.accept(value, consumed, available);
                }
                Err(DecodeFailure::Incomplete { required_total, .. }) => {
                    assert!(
                        required_total.get() <= C::MAX_DECODE_UNITS_PER_VALUE,
                        "Codec::decode incomplete required_total exceeded Codec::MAX_DECODE_UNITS_PER_VALUE",
                    );
                    self.refill_after_incomplete(required_total, available)?;
                }
                Err(DecodeFailure::Invalid { source, consumed }) => {
                    return self.reject::<C, M>(source, consumed, available, map_error);
                }
            }
        }
    }

    /// Prepares the buffered window for one codec decode attempt.
    fn prepare_buffered_window(
        &mut self,
        min_units_per_value: usize,
        max_units_per_value: usize,
    ) -> Result<usize> {
        let available = self.input.unread_len();
        if available < min_units_per_value && !self.input.fill_until(min_units_per_value)? {
            let available = self.input.unread_len();
            // SAFETY: `available` is the current unread length.
            unsafe {
                self.input.consume(available);
            }
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "failed to decode complete value",
            ));
        }

        if self.input.unread_len() < max_units_per_value
            && max_units_per_value <= self.input.capacity()
        {
            let _ = self.input.fill_until(max_units_per_value)?;
        }
        Ok(self.input.unread_len().min(max_units_per_value))
    }

    /// Accepts a decoded value and consumes its source units.
    fn accept<Value>(
        &mut self,
        value: Value,
        consumed: NonZeroUsize,
        available: usize,
    ) -> Result<Value> {
        if consumed.get() > available {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "codec consumed units exceed unread window",
            ));
        }
        // SAFETY: The check above proves `consumed <= available`, and
        // `available` came from the current unread window.
        unsafe {
            self.input.consume(consumed.get());
        }
        Ok(value)
    }

    /// Refills after the codec reports incomplete input.
    fn refill_after_incomplete(
        &mut self,
        required_total: NonZeroUsize,
        available: usize,
    ) -> Result<()> {
        let required_total = required_total.get();
        if available >= required_total {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "codec reported incomplete input within available window",
            ));
        }
        self.input
            .try_reserve_capacity(required_total)
            .map_err(|error| Error::new(ErrorKind::OutOfMemory, error))?;
        if !self.input.fill_until(required_total)? {
            let available = self.input.unread_len();
            // SAFETY: `available` is the current unread length.
            unsafe {
                self.input.consume(available);
            }
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "failed to decode complete value",
            ));
        }
        Ok(())
    }

    /// Rejects invalid codec input and applies its consumption hint.
    fn reject<C, M>(
        &mut self,
        source: C::DecodeError,
        consumed: Option<NonZeroUsize>,
        available: usize,
        map_error: &mut M,
    ) -> Result<C::Value>
    where
        C: Codec<Unit = I::Item>,
        M: FnMut(C::DecodeError) -> Error,
    {
        if let Some(consumed) = consumed {
            if consumed.get() > available {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "decode error consumed units exceed unread window",
                ));
            }
            // SAFETY: The check above proves `consumed <= available`, and
            // `available` came from the current unread window.
            unsafe {
                self.input.consume(consumed.get());
            }
        }
        Err(map_error(source))
    }
}
