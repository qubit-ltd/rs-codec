// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private driver for decoding one codec value from buffered input.

use core::num::NonZeroUsize;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;

use qubit_io::BufferedInput;
use qubit_io::Input;
use qubit_utils::allocation_error;

use crate::Codec;
use crate::DecodeFailure;

/// Drives one codec decode operation against persistent buffered input.
pub(in crate::transcode::io) struct CodecDecodeDriver<'a, I>
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
    ///
    /// # Parameters
    ///
    /// - `input`: Buffered unit input shared with the public adapter. The
    ///   borrow keeps every unit this driver consumes inside that buffer.
    ///
    /// # Returns
    ///
    /// Returns a driver that decodes exactly one value per `read_one` call.
    #[inline]
    #[must_use]
    pub(in crate::transcode::io) const fn new(input: &'a mut BufferedInput<I>) -> Self {
        Self { input }
    }

    /// Reads one decoded value after codec lifecycle reset completed.
    ///
    /// The driver fills the shared buffer until the codec minimum is
    /// available, retries after an incomplete report, and switches to
    /// `Codec::decode_eof` once the underlying input is exhausted.
    ///
    /// # Parameters
    ///
    /// - `codec`: Codec whose decode entry points read the prepared window.
    /// - `map_error`: Converts a codec decode error into the reported I/O
    ///   error; it is called at most once per call.
    ///
    /// # Returns
    ///
    /// Returns the decoded value after consuming exactly the units the codec
    /// reported as read.
    ///
    /// # Errors
    ///
    /// Returns `ErrorKind::UnexpectedEof` when the input ends before one
    /// complete value is available, `ErrorKind::InvalidData` when the codec
    /// reports consumption beyond the unread window or reports incomplete
    /// input inside a window that is already large enough, the allocation
    /// error when the buffer cannot grow, and the mapped codec error when the
    /// codec rejects the input.
    ///
    /// # Panics
    ///
    /// Panics when a codec reports `DecodeFailure::Incomplete` with a
    /// `required_total` above `Codec::MAX_DECODE_UNITS_PER_VALUE`, which
    /// violates the codec contract.
    pub(in crate::transcode::io) fn read_one<C, M>(&mut self, codec: &mut C, map_error: &mut M) -> Result<C::Value>
    where
        C: Codec<Unit = I::Item>,
        M: FnMut(C::DecodeError) -> Error,
    {
        let min_units_per_value = C::MIN_UNITS_PER_VALUE;
        let max_units_per_value = C::MAX_DECODE_UNITS_PER_VALUE.max(min_units_per_value);
        self.input
            .try_reserve_capacity(min_units_per_value)
            .map_err(allocation_error)?;
        loop {
            let (available, end_of_input) = self.prepare_buffered_window(min_units_per_value, max_units_per_value)?;
            let units = &self.input.unread()[..available];
            debug_assert!(units.len() >= min_units_per_value);
            let decode_result = unsafe {
                // SAFETY: `min_units_per_value <= units.len()` guarantees
                // the selected decode entry point's preconditions for this
                // slice.
                if end_of_input {
                    codec.decode_eof(units, 0)
                } else {
                    codec.decode(units, 0)
                }
            };
            match decode_result {
                Ok((value, consumed)) => {
                    return self.accept(value, consumed, available);
                }
                Err(DecodeFailure::Incomplete { source, required_total }) => {
                    assert!(
                        required_total.get() <= C::MAX_DECODE_UNITS_PER_VALUE,
                        "Codec::decode incomplete required_total exceeded Codec::MAX_DECODE_UNITS_PER_VALUE",
                    );
                    if end_of_input {
                        let available = self.input.unread_len();
                        if required_total.get() <= available {
                            return Err(Error::new(
                                ErrorKind::InvalidData,
                                "codec reported incomplete input within available window",
                            ));
                        }
                        // SAFETY: `available` is the current unread length.
                        unsafe {
                            self.input.consume(available);
                        }
                        return match source {
                            Some(source) => Err(map_error(source)),
                            None => Err(Error::new(ErrorKind::UnexpectedEof, "failed to decode complete value")),
                        };
                    }
                    if !self.refill_after_incomplete(required_total, available)? {
                        let available = self.input.unread_len();
                        let units = &self.input.unread()[..available];
                        let eof_result = unsafe {
                            // SAFETY: the original decode established that at
                            // least the codec minimum units remain available.
                            codec.decode_eof(units, 0)
                        };
                        match eof_result {
                            Ok((value, consumed)) => {
                                return self.accept(value, consumed, available);
                            }
                            Err(DecodeFailure::Invalid { source, consumed }) => {
                                return self.reject::<C, M>(source, consumed, available, map_error);
                            }
                            Err(DecodeFailure::Incomplete { source, required_total }) => {
                                assert!(
                                    required_total.get() <= C::MAX_DECODE_UNITS_PER_VALUE,
                                    "Codec::decode_eof incomplete required_total exceeded Codec::MAX_DECODE_UNITS_PER_VALUE",
                                );
                                if required_total.get() <= available {
                                    return Err(Error::new(
                                        ErrorKind::InvalidData,
                                        "codec reported incomplete input within available window",
                                    ));
                                }
                                unsafe {
                                    self.input.consume(available);
                                }
                                return match source {
                                    Some(source) => Err(map_error(source)),
                                    None => {
                                        Err(Error::new(ErrorKind::UnexpectedEof, "failed to decode complete value"))
                                    }
                                };
                            }
                        }
                    }
                }
                Err(DecodeFailure::Invalid { source, consumed }) => {
                    return self.reject::<C, M>(source, consumed, available, map_error);
                }
            }
        }
    }

    /// Prepares the buffered window for one codec decode attempt.
    ///
    /// The buffer is filled up to the codec minimum first, then
    /// opportunistically up to the codec maximum without exceeding the
    /// existing buffer capacity. All unread units are consumed before
    /// reporting the unexpected end of input, so the caller never observes
    /// a partially drained window.
    ///
    /// # Parameters
    ///
    /// - `min_units_per_value`: Smallest unit count one codec attempt may read.
    /// - `max_units_per_value`: Largest unit count one codec attempt may read.
    ///
    /// # Returns
    ///
    /// Returns the number of leading unread units the codec may read together
    /// with whether the underlying input is exhausted, which selects
    /// `Codec::decode` or `Codec::decode_eof`.
    ///
    /// # Errors
    ///
    /// Returns `ErrorKind::UnexpectedEof` when fewer than `min_units_per_value`
    /// units remain after refilling, and propagates buffer fill or allocation
    /// failures from the underlying input.
    fn prepare_buffered_window(
        &mut self,
        min_units_per_value: usize,
        max_units_per_value: usize,
    ) -> Result<(usize, bool)> {
        let mut end_of_input = false;
        let available = self.input.unread_len();
        if available < min_units_per_value && !self.input.fill_until(min_units_per_value)? {
            end_of_input = true;
        }
        if self.input.unread_len() < min_units_per_value {
            let available = self.input.unread_len();
            // SAFETY: `available` is the current unread length.
            unsafe {
                self.input.consume(available);
            }
            return Err(Error::new(ErrorKind::UnexpectedEof, "failed to decode complete value"));
        }

        if self.input.unread_len() < max_units_per_value
            && max_units_per_value <= self.input.capacity()
            && !self.input.fill_until(max_units_per_value)?
        {
            end_of_input = true;
        }
        Ok((self.input.unread_len().min(max_units_per_value), end_of_input))
    }

    /// Accepts a decoded value and consumes its source units.
    ///
    /// # Parameters
    ///
    /// - `value`: Value produced by the codec for the prepared window.
    /// - `consumed`: Non-zero unit count the codec reported as read.
    /// - `available`: Unit count the codec was allowed to read.
    ///
    /// # Returns
    ///
    /// Returns `value` after the consumed units are removed from the buffer.
    ///
    /// # Errors
    ///
    /// Returns `ErrorKind::InvalidData` when `consumed` exceeds `available`,
    /// because that consumption would move the window past its own bounds.
    fn accept<Value>(&mut self, value: Value, consumed: NonZeroUsize, available: usize) -> Result<Value> {
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
    ///
    /// # Parameters
    ///
    /// - `required_total`: Total unit count the codec needs to make progress.
    /// - `available`: Unit count already readable before this refill.
    ///
    /// # Returns
    ///
    /// Returns `true` when the buffer now holds `required_total` units and the
    /// decode attempt should be retried, and `false` when the input is
    /// exhausted first.
    ///
    /// # Errors
    ///
    /// Returns `ErrorKind::InvalidData` when the codec already had
    /// `required_total` units readable, which contradicts an incomplete report,
    /// and propagates buffer growth and fill failures from the underlying
    /// input.
    fn refill_after_incomplete(&mut self, required_total: NonZeroUsize, available: usize) -> Result<bool> {
        let required_total = required_total.get();
        if available >= required_total {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "codec reported incomplete input within available window",
            ));
        }
        self.input
            .try_reserve_capacity(required_total)
            .map_err(allocation_error)?;
        self.input.fill_until(required_total)
    }

    /// Rejects invalid codec input and applies its consumption hint.
    ///
    /// The optional consumption hint is honored first so the mapped error
    /// describes a window that already advanced past the rejected units.
    ///
    /// # Parameters
    ///
    /// - `source`: Codec decode error reported for the prepared window.
    /// - `consumed`: Optional unit count the codec reported as read before
    ///   failing.
    /// - `available`: Unit count the codec was allowed to read.
    /// - `map_error`: Converts `source` into the reported I/O error.
    ///
    /// # Errors
    ///
    /// Always returns an error. Returns `ErrorKind::InvalidData` when the
    /// consumption hint exceeds `available` without calling `map_error`,
    /// otherwise the error produced by `map_error`.
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
