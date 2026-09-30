// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Owned output from a complete single-value decode lifecycle.

/// Values produced by decode reset, the main decode, and decode finish.
///
/// This result keeps lifecycle output in phase order without allowing finish
/// output to overwrite reset output.
///
/// # Type Parameters
///
/// - `V`: Logical value type produced by the codec.
///
/// # Examples
///
/// A stateless identity codec produces one lifecycle whose reset and finish
/// phases stay empty:
///
/// ```
/// use core::convert::Infallible;
/// use core::num::NonZeroUsize;
///
/// use qubit_codec::Codec;
/// use qubit_codec::CodecValueDecoder;
/// use qubit_codec::DecodeFailure;
/// use qubit_codec::DecodeLifecycleOutput;
///
/// struct Identity;
///
/// impl Codec for Identity {
///     type Value = u8;
///     type Unit = u8;
///     type DecodeError = Infallible;
///     type EncodeError = Infallible;
///     const MIN_UNITS_PER_VALUE: usize = 1;
///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
///
///     unsafe fn decode(&mut self, input: &[u8], index: usize)
///         -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>>
///     {
///         Ok((input[index], NonZeroUsize::MIN))
///     }
///
///     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
///         -> Result<usize, Infallible>
///     {
///         output[index] = *value;
///         Ok(1)
///     }
/// }
///
/// let output: DecodeLifecycleOutput<u8> =
///     CodecValueDecoder::new(Identity).decode_lifecycle(&[7]).unwrap();
/// assert_eq!(*output.value(), 7);
/// assert!(output.reset().is_empty());
/// assert!(output.finish().is_empty());
///
/// let (reset, value, finish) = output.into_parts();
/// assert_eq!(value, 7);
/// assert!(reset.is_empty() && finish.is_empty());
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct DecodeLifecycleOutput<V> {
    /// Values emitted while resetting decode state.
    reset: Vec<V>,
    /// Main value decoded from the supplied input.
    value: V,
    /// Values emitted while finishing decode state.
    finish: Vec<V>,
}

impl<V> DecodeLifecycleOutput<V> {
    /// Creates owned output from the three decode lifecycle phases.
    ///
    /// # Parameters
    ///
    /// - `reset`: Values emitted while resetting decode state.
    /// - `value`: Main value decoded from the supplied input.
    /// - `finish`: Values emitted while finishing decode state.
    ///
    /// # Returns
    ///
    /// Returns output preserving all three lifecycle phases.
    #[inline]
    pub(crate) fn new(reset: Vec<V>, value: V, finish: Vec<V>) -> Self {
        Self { reset, value, finish }
    }

    /// Returns values emitted while resetting decode state.
    ///
    /// # Returns
    ///
    /// Returns the reset output in emission order.
    #[inline]
    #[must_use]
    pub fn reset(&self) -> &[V] {
        &self.reset
    }

    /// Returns the main value decoded from the supplied input.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the main decoded value.
    #[inline]
    #[must_use]
    pub const fn value(&self) -> &V {
        &self.value
    }

    /// Returns values emitted while finishing decode state.
    ///
    /// # Returns
    ///
    /// Returns the finish output in emission order.
    #[inline]
    #[must_use]
    pub fn finish(&self) -> &[V] {
        &self.finish
    }

    /// Consumes this result and returns all lifecycle phases.
    ///
    /// # Returns
    ///
    /// Returns reset values, the main decoded value, and finish values.
    #[inline]
    #[must_use]
    pub fn into_parts(self) -> (Vec<V>, V, Vec<V>) {
        (self.reset, self.value, self.finish)
    }
}
