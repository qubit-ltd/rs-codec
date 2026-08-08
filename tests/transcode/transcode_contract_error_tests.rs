// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use qubit_codec::TranscodeContractError;

#[test]
fn test_transcode_contract_error_display_formats_all_variants() {
    assert_eq!(
        "transcoder consumed 3 units but only 2 were available",
        TranscodeContractError::OverRead {
            read: 3,
            available: 2,
        }
        .to_string(),
    );
    assert_eq!(
        "transcoder wrote 5 units but only 4 output slots were available",
        TranscodeContractError::OverWritten {
            written: 5,
            available: 4,
        }
        .to_string(),
    );
    assert_eq!(
        "transcoder progress overflow: index 7 plus advanced 8",
        TranscodeContractError::ProgressIndexOverflow {
            index: 7,
            advanced: 8,
        }
        .to_string(),
    );
    assert_eq!(
        "transcoder reported required 2 with available 2",
        TranscodeContractError::SatisfiedNeed {
            required: 2,
            available: 2,
        }
        .to_string(),
    );
    assert_eq!(
        "transcoder reported Complete after consuming 1 of 2 available input units",
        TranscodeContractError::CompleteWithRemainingInput {
            read: 1,
            available: 2,
        }
        .to_string(),
    );
}

#[test]
fn test_transcode_contract_error_is_copy_hashable_and_has_no_source() {
    let error = TranscodeContractError::OverRead {
        read: 3,
        available: 2,
    };
    let copied = error;
    assert_eq!(error, copied);
    assert!(error.source().is_none());

    let mut first = DefaultHasher::new();
    error.hash(&mut first);
    let mut second = DefaultHasher::new();
    copied.hash(&mut second);
    assert_eq!(first.finish(), second.finish());
}
