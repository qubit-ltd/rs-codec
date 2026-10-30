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
///
/// # Type Parameters
///
/// - `'a`: Lifetime of the mutable borrow of the shared input buffer.
/// - `I`: Input implementation whose items are the codec's decode units.
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
    /// # Type Parameters
    ///
    /// - `C`: Codec that decodes units from the shared input buffer.
    /// - `M`: Callback that converts codec errors to I/O errors.
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
            let decode_result = Self::decode_window(codec, units, end_of_input);
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
                        return self.handle_eof_incomplete::<C, M>(source, required_total, map_error);
                    }
                    if !self.refill_after_incomplete(required_total, available)? {
                        return self.retry_decode_at_eof::<C, M>(codec, map_error);
                    }
                }
                Err(DecodeFailure::Invalid { source, consumed }) => {
                    return self.reject::<C, M>(source, consumed, available, map_error);
                }
            }
        }
    }

    /// Decodes the prepared window using the entry point selected by EOF state.
    ///
    /// # Parameters
    ///
    /// - `codec`: Codec that decodes the prepared unit window.
    /// - `units`: Readable units starting at the current value boundary.
    /// - `end_of_input`: Whether the underlying input has been exhausted.
    ///
    /// # Returns
    ///
    /// Returns the codec's decode result without changing the buffered input.
    fn decode_window<C>(
        codec: &mut C,
        units: &[I::Item],
        end_of_input: bool,
    ) -> core::result::Result<(C::Value, NonZeroUsize), DecodeFailure<C::DecodeError>>
    where
        C: Codec<Unit = I::Item>,
    {
        // SAFETY: the caller prepared at least the codec minimum number of
        // readable units, satisfying either decode entry point's preconditions.
        unsafe {
            if end_of_input {
                codec.decode_eof(units, 0)
            } else {
                codec.decode(units, 0)
            }
        }
    }

    /// Retries decoding with EOF semantics after refill reports exhaustion.
    ///
    /// # Type Parameters
    ///
    /// - `C`: Codec that decodes the final buffered window.
    /// - `M`: Callback that converts codec errors to I/O errors.
    ///
    /// # Parameters
    ///
    /// - `codec`: Codec whose EOF-aware entry point decodes the final window.
    /// - `map_error`: Converts codec errors into reported I/O errors.
    ///
    /// # Errors
    ///
    /// Returns `ErrorKind::UnexpectedEof` for an unresolved incomplete value
    /// without codec context, the mapped codec error when context is present,
    /// or `ErrorKind::InvalidData` when the codec reports an invalid
    /// consumption count or an already available required width.
    fn retry_decode_at_eof<C, M>(&mut self, codec: &mut C, map_error: &mut M) -> Result<C::Value>
    where
        C: Codec<Unit = I::Item>,
        M: FnMut(C::DecodeError) -> Error,
    {
        let available = self.input.unread_len();
        let units = &self.input.unread()[..available];
        // SAFETY: the failed streaming decode established that at least the
        // codec minimum units remain readable in this unchanged buffer.
        let eof_result = unsafe { codec.decode_eof(units, 0) };
        match eof_result {
            Ok((value, consumed)) => self.accept(value, consumed, available),
            Err(DecodeFailure::Invalid { source, consumed }) => {
                self.reject::<C, M>(source, consumed, available, map_error)
            }
            Err(DecodeFailure::Incomplete { source, required_total }) => {
                self.handle_eof_incomplete::<C, M>(source, required_total, map_error)
            }
        }
    }

    /// Prepares the buffered window for one codec decode attempt.
    ///
    /// The buffer is filled only up to the codec minimum before decoding.
    /// Additional units are read only when the codec reports that its current
    /// window is incomplete. All unread units are consumed before reporting
    /// the unexpected end of input, so the caller never observes a partially
    /// drained window.
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
        Ok((self.input.unread_len().min(max_units_per_value), end_of_input))
    }

    /// Accepts a decoded value and consumes its source units.
    ///
    /// # Type Parameters
    ///
    /// - `Value`: Decoded value returned after consuming its source units.
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

    /// Consumes the unread tail and reports an incomplete value at EOF.
    ///
    /// # Type Parameters
    ///
    /// - `C`: Codec whose incomplete decode report is being handled.
    /// - `M`: Callback that converts a present codec error to an I/O error.
    ///
    /// # Parameters
    ///
    /// - `source`: Optional codec error associated with the incomplete value.
    /// - `required_total`: Total unit count the codec needs to decode a value.
    /// - `map_error`: Converts a present codec error into an I/O error.
    ///
    /// # Errors
    ///
    /// Returns `ErrorKind::InvalidData` if the codec's required width is
    /// already available, `ErrorKind::UnexpectedEof` when no codec error is
    /// present, or the mapped codec error otherwise.
    ///
    /// # Panics
    ///
    /// Panics when `required_total` exceeds the codec's declared maximum.
    fn handle_eof_incomplete<C, M>(
        &mut self,
        source: Option<C::DecodeError>,
        required_total: NonZeroUsize,
        map_error: &mut M,
    ) -> Result<C::Value>
    where
        C: Codec<Unit = I::Item>,
        M: FnMut(C::DecodeError) -> Error,
    {
        assert!(
            required_total.get() <= C::MAX_DECODE_UNITS_PER_VALUE,
            "Codec::decode_eof incomplete required_total exceeded Codec::MAX_DECODE_UNITS_PER_VALUE",
        );
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
        match source {
            Some(source) => Err(map_error(source)),
            None => Err(Error::new(ErrorKind::UnexpectedEof, "failed to decode complete value")),
        }
    }

    /// Rejects invalid codec input and applies its consumption hint.
    ///
    /// The optional consumption hint is honored first so the mapped error
    /// describes a window that already advanced past the rejected units.
    ///
    /// # Type Parameters
    ///
    /// - `C`: Codec that reported invalid input.
    /// - `M`: Callback that converts the codec error to an I/O error.
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
