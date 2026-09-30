// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Framework-level failures reported by safe transcode APIs.

use thiserror::Error;

use super::capacity_error::CapacityError;
use super::transcode_contract_error::TranscodeContractError;
use crate::Codec;
use crate::codec::assert_unit_bounds;

/// Framework-level failure reported by a transcode operation.
///
/// These failures are part of the safe public transcode API. They describe
/// caller-supplied buffer ranges, capacity planning, complete-input shape, and
/// stream lifecycle misuse that the transcode layer can detect without
/// interpreting a concrete codec error.
///
/// # Examples
///
/// ```
/// use qubit_codec::TranscodeFailure;
///
/// let failure = TranscodeFailure::invalid_input_index(4, 3);
/// assert_eq!(failure.to_string(), "invalid input index 4 for input length 3");
/// assert_eq!(TranscodeFailure::ensure_input_index(3, 4), Err(failure));
/// ```
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
#[non_exhaustive]
#[must_use]
pub enum TranscodeFailure {
    /// The caller supplied an input index outside the input slice.
    #[error("invalid input index {index} for input length {input_len}")]
    InvalidInputIndex {
        /// Invalid input index supplied by the caller.
        index: usize,
        /// Length of the input slice.
        input_len: usize,
    },

    /// The caller supplied an output index outside the output slice.
    #[error("invalid output index {index} for output length {output_len}")]
    InvalidOutputIndex {
        /// Invalid output index supplied by the caller.
        index: usize,
        /// Length of the output slice.
        output_len: usize,
    },

    /// The caller supplied an output range extending beyond the output slice.
    #[error("invalid output range at index {output_index} with length {range_len} for output length {output_len}")]
    InvalidOutputRange {
        /// Absolute output index where the range starts.
        output_index: usize,
        /// Length of the caller-supplied output range.
        range_len: usize,
        /// Length of the output slice.
        output_len: usize,
    },

    /// The output slice cannot hold all required output.
    #[error("insufficient output at index {output_index}: required {required} units, available {available}")]
    InsufficientOutput {
        /// Absolute output index where writing would start.
        output_index: usize,
        /// Output units required from `output_index`.
        required: usize,
        /// Output units available from `output_index`.
        available: usize,
    },

    /// Output length arithmetic overflowed.
    #[error("output length arithmetic overflow")]
    OutputLengthOverflow,

    /// The allocator could not reserve the requested output storage.
    #[error("output allocation failed")]
    AllocationFailed,

    /// The complete input ended with an incomplete value.
    #[error(
        "incomplete input at index {input_index}: at least {required} units required to retry, {available} available"
    )]
    IncompleteInput {
        /// Absolute input index where the incomplete value starts.
        input_index: usize,
        /// Current minimum total input units required from `input_index`
        /// before retrying. A later retry may raise this lower bound.
        required: usize,
        /// Input units available from `input_index`.
        available: usize,
    },

    /// A transcoder returned progress inconsistent with the supplied buffers.
    #[error("invalid transcoder progress: {source}")]
    InvalidProgress {
        /// Contract error describing the invalid progress.
        #[source]
        source: TranscodeContractError,
    },

    /// The input contains exactly one decoded value plus trailing units.
    #[error("trailing input after value: consumed {consumed} units, remaining {remaining}")]
    TrailingInput {
        /// Units consumed by the decoded value.
        consumed: usize,
        /// Extra units left after the decoded value.
        remaining: usize,
    },

    /// A strict single-value decoder cannot expose codec lifecycle output.
    #[error(
        "strict single-value decoding does not support lifecycle output: reset bound {reset_bound}, finish bound {finish_bound}"
    )]
    UnsupportedDecodeLifecycleOutput {
        /// Maximum values the codec may emit while resetting decode state.
        reset_bound: usize,
        /// Maximum values the codec may emit while finishing decode state.
        finish_bound: usize,
    },

    /// `transcode` was called after the logical stream was finished.
    #[error("transcode called after finish without an intervening reset")]
    TranscodeAfterFinish,

    /// `transcode` was called before the first successful reset.
    #[error("transcode called before reset")]
    TranscodeBeforeReset,

    /// `finish` was called after the logical stream was already finished.
    #[error("finish called twice without an intervening reset")]
    FinishAfterFinish,

    /// `finish` was called before the first successful reset.
    #[error("finish called before reset")]
    FinishBeforeReset,

    /// Reset or finish failed after codec, hook, or buffered state may have
    /// changed. Continuing could repeat side effects or observe inconsistent
    /// state; a successful reset is required before further use.
    #[error("transcoder lifecycle is poisoned; a successful reset is required")]
    LifecyclePoisoned,
}

impl TranscodeFailure {
    /// Creates an invalid-input-index error.
    ///
    /// # Parameters
    ///
    /// - `index`: Absolute input index rejected by the caller boundary.
    /// - `len`: Total number of input units.
    ///
    /// # Returns
    ///
    /// Returns a failure retaining the supplied index and input length.
    #[inline]
    pub const fn invalid_input_index(index: usize, len: usize) -> Self {
        Self::InvalidInputIndex { index, input_len: len }
    }

    /// Creates an invalid-output-index error.
    ///
    /// # Parameters
    ///
    /// - `index`: Absolute output index rejected by the caller boundary.
    /// - `len`: Total number of writable output units.
    ///
    /// # Returns
    ///
    /// Returns a failure retaining the supplied index and output length.
    #[inline]
    pub const fn invalid_output_index(index: usize, len: usize) -> Self {
        Self::InvalidOutputIndex { index, output_len: len }
    }

    /// Creates an invalid-output-range error.
    ///
    /// # Parameters
    ///
    /// - `output_index`: Absolute start of the requested output range.
    /// - `range_len`: Requested range length.
    /// - `output_len`: Total output slice length.
    ///
    /// # Returns
    ///
    /// Returns a failure describing the range that exceeds the output slice.
    #[inline]
    pub const fn invalid_output_range(output_index: usize, range_len: usize, output_len: usize) -> Self {
        Self::InvalidOutputRange {
            output_index,
            range_len,
            output_len,
        }
    }

    /// Creates an insufficient-output error.
    ///
    /// # Parameters
    ///
    /// - `output_index`: Absolute start of the writable output.
    /// - `required`: Units needed from that index.
    /// - `available`: Units actually available from that index.
    ///
    /// # Returns
    ///
    /// Returns a failure retaining the required and available capacities.
    #[inline]
    pub const fn insufficient_output(output_index: usize, required: usize, available: usize) -> Self {
        Self::InsufficientOutput {
            output_index,
            required,
            available,
        }
    }

    /// Creates an output-length-overflow error.
    ///
    /// # Returns
    ///
    /// Returns the framework marker for unrepresentable output capacity.
    #[inline]
    pub const fn output_length_overflow() -> Self {
        Self::OutputLengthOverflow
    }

    /// Creates an output-allocation failure.
    ///
    /// # Returns
    ///
    /// Returns the framework marker for a failed output allocation.
    #[inline]
    pub const fn allocation_failed() -> Self {
        Self::AllocationFailed
    }

    /// Creates an incomplete-input error.
    ///
    /// # Parameters
    ///
    /// - `input_index`: Absolute index where the incomplete value starts.
    /// - `required`: Current minimum total units needed from that index.
    /// - `available`: Units currently available from that index.
    ///
    /// # Returns
    ///
    /// Returns a failure preserving the incomplete tail and retry requirement.
    #[inline]
    pub const fn incomplete_input(input_index: usize, required: usize, available: usize) -> Self {
        Self::IncompleteInput {
            input_index,
            required,
            available,
        }
    }

    /// Creates an invalid-progress error.
    ///
    /// # Parameters
    ///
    /// - `source`: Contract violation detected in a transcoder report.
    ///
    /// # Returns
    ///
    /// Returns a framework failure that retains the contract error as its
    /// source.
    #[inline]
    pub const fn invalid_progress(source: TranscodeContractError) -> Self {
        Self::InvalidProgress { source }
    }

    /// Creates a trailing-input error.
    ///
    /// # Parameters
    ///
    /// - `consumed`: Units used by the decoded value.
    /// - `remaining`: Extra units after that value.
    ///
    /// # Returns
    ///
    /// Returns a failure describing the otherwise successful single-value
    /// decode.
    #[inline]
    pub const fn trailing_input(consumed: usize, remaining: usize) -> Self {
        Self::TrailingInput { consumed, remaining }
    }

    /// Creates an unsupported decode-lifecycle-output error.
    ///
    /// # Parameters
    ///
    /// - `reset_bound`: Maximum values produced by decode reset.
    /// - `finish_bound`: Maximum values produced by decode finish.
    ///
    /// # Returns
    ///
    /// Returns a failure retaining the lifecycle output bounds rejected by
    /// strict decoding.
    #[inline]
    pub const fn unsupported_decode_lifecycle_output(reset_bound: usize, finish_bound: usize) -> Self {
        Self::UnsupportedDecodeLifecycleOutput {
            reset_bound,
            finish_bound,
        }
    }

    /// Validates that `input_index` is within an input slice.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Total input slice length.
    /// - `input_index`: Absolute start index, including the end boundary.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the index is at most the input length.
    ///
    /// # Errors
    ///
    /// Returns [`Self::InvalidInputIndex`] when the index exceeds the length.
    #[inline]
    pub fn ensure_input_index(input_len: usize, input_index: usize) -> Result<(), Self> {
        if input_index > input_len {
            return Err(Self::invalid_input_index(input_index, input_len));
        }
        Ok(())
    }

    /// Validates that enough input units are available from `input_index`.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Total input slice length.
    /// - `input_index`: Absolute start of the input region.
    /// - `min_required`: Minimum readable units required from that start.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the start index and minimum readable capacity are
    /// valid.
    ///
    /// # Errors
    ///
    /// Returns [`Self::InvalidInputIndex`] for an invalid start, or
    /// [`Self::IncompleteInput`] when fewer than `min_required` units remain.
    #[inline]
    pub fn ensure_min_input(input_len: usize, input_index: usize, min_required: usize) -> Result<(), Self> {
        Self::ensure_input_index(input_len, input_index)?;
        let available = input_len - input_index;
        if available < min_required {
            return Err(Self::incomplete_input(input_index, min_required, available));
        }
        Ok(())
    }

    /// Validates a consumed input count and rejects trailing input.
    ///
    /// # Parameters
    ///
    /// - `consumed`: Number of input units consumed by the decoded value.
    /// - `input_len`: Total number of input units supplied.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when `consumed == input_len`.
    ///
    /// # Errors
    ///
    /// Returns [`TranscodeFailure::InvalidInputIndex`] when `consumed` exceeds
    /// `input_len`, or [`TranscodeFailure::TrailingInput`] when unconsumed
    /// input remains.
    #[inline]
    pub fn ensure_no_trailing_input(consumed: usize, input_len: usize) -> Result<(), Self> {
        Self::ensure_input_index(input_len, consumed)?;
        let remaining = input_len - consumed;
        if remaining != 0 {
            return Err(Self::trailing_input(consumed, remaining));
        }
        Ok(())
    }

    /// Validates that `output_index` is within an output slice.
    ///
    /// # Parameters
    ///
    /// - `output_len`: Total output slice length.
    /// - `output_index`: Absolute start index, including the end boundary.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the index is at most the output length.
    ///
    /// # Errors
    ///
    /// Returns [`Self::InvalidOutputIndex`] when the index exceeds the length.
    #[inline]
    pub fn ensure_output_index(output_len: usize, output_index: usize) -> Result<(), Self> {
        if output_index > output_len {
            return Err(Self::invalid_output_index(output_index, output_len));
        }
        Ok(())
    }

    /// Validates input and output start indices for a transcode call.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Total input slice length.
    /// - `input_index`: Absolute starting input index.
    /// - `output_len`: Total output slice length.
    /// - `output_index`: Absolute starting output index.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when both indices lie within their slices or at their
    /// ends.
    ///
    /// # Errors
    ///
    /// Returns [`Self::InvalidInputIndex`] first for invalid input, otherwise
    /// [`Self::InvalidOutputIndex`] for invalid output.
    #[inline]
    pub fn ensure_transcode_indices(
        input_len: usize,
        input_index: usize,
        output_len: usize,
        output_index: usize,
    ) -> Result<(), Self> {
        Self::ensure_input_index(input_len, input_index)?;
        Self::ensure_output_index(output_len, output_index)
    }

    /// Validates that an output slice can hold one-shot finalization output.
    ///
    /// # Parameters
    ///
    /// - `output_len`: Total output slice length.
    /// - `output_index`: Absolute start of the writable region.
    /// - `required`: Minimum output units needed from that start.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the output region has at least `required` units.
    ///
    /// # Errors
    ///
    /// Returns [`Self::InvalidOutputIndex`] for an invalid start, or
    /// [`Self::InsufficientOutput`] when the remaining capacity is too small.
    #[inline]
    pub fn ensure_output_capacity(output_len: usize, output_index: usize, required: usize) -> Result<(), Self> {
        Self::ensure_output_index(output_len, output_index)?;
        let available = output_len - output_index;
        if available < required {
            return Err(Self::insufficient_output(output_index, required, available));
        }
        Ok(())
    }

    /// Validates an indexed output range and its minimum writable capacity.
    ///
    /// # Parameters
    ///
    /// - `output_len`: Total output slice length.
    /// - `output_index`: Absolute start of the writable range.
    /// - `range_len`: Length of the caller-selected writable range.
    /// - `required`: Minimum capacity required within the selected range.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the selected range fits and has enough capacity.
    ///
    /// # Errors
    ///
    /// Returns [`Self::InvalidOutputIndex`] for an invalid start,
    /// [`Self::InvalidOutputRange`] when the range exceeds the slice, or
    /// [`Self::InsufficientOutput`] when the selected range is too short.
    #[inline]
    pub fn ensure_output_range(
        output_len: usize,
        output_index: usize,
        range_len: usize,
        required: usize,
    ) -> Result<(), Self> {
        Self::ensure_output_index(output_len, output_index)?;
        let available = output_len - output_index;
        if range_len > available {
            return Err(Self::invalid_output_range(output_index, range_len, output_len));
        }
        if range_len < required {
            return Err(Self::insufficient_output(output_index, required, range_len));
        }
        Ok(())
    }

    /// Rejects lifecycle output for a strict single-value decode.
    ///
    /// # Type Parameters
    ///
    /// - `C`: Codec whose decode lifecycle bounds are checked.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when both decode lifecycle output bounds are zero.
    ///
    /// # Errors
    ///
    /// Returns [`Self::UnsupportedDecodeLifecycleOutput`] when reset or finish
    /// may emit values.
    #[inline]
    pub(crate) fn ensure_no_decode_lifecycle_output<C>() -> Result<(), Self>
    where
        C: Codec,
    {
        assert_unit_bounds::<C>();
        if C::MAX_DECODE_RESET_VALUES != 0 || C::MAX_DECODE_FINISH_VALUES != 0 {
            return Err(Self::unsupported_decode_lifecycle_output(
                C::MAX_DECODE_RESET_VALUES,
                C::MAX_DECODE_FINISH_VALUES,
            ));
        }
        Ok(())
    }
}

impl From<CapacityError> for TranscodeFailure {
    /// Converts capacity planning errors into transcode framework failures.
    ///
    /// # Parameters
    ///
    /// - `error`: Capacity-planning failure to translate.
    ///
    /// # Returns
    ///
    /// Returns the corresponding framework output-length-overflow failure.
    #[inline]
    fn from(error: CapacityError) -> Self {
        match error {
            CapacityError::OutputLengthOverflow => Self::OutputLengthOverflow,
        }
    }
}

impl From<TranscodeContractError> for TranscodeFailure {
    /// Converts a broken transcoder progress report into a framework failure.
    ///
    /// # Parameters
    ///
    /// - `error`: Progress-contract violation to retain as the error source.
    ///
    /// # Returns
    ///
    /// Returns [`Self::InvalidProgress`] carrying the original violation.
    #[inline]
    fn from(error: TranscodeContractError) -> Self {
        Self::invalid_progress(error)
    }
}
