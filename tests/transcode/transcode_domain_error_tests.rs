// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::TranscodeDomainError;

#[test]
fn test_domain_error_accessors_and_mapping_cover_all_phases() {
    let reset = TranscodeDomainError::reset("reset");
    let main = TranscodeDomainError::main_with_consumed("main", 4, Some(crate::nonzero(2)));
    let finish = TranscodeDomainError::finish("finish");

    assert_eq!("reset", *reset.source());
    assert_eq!("main", *main.source());
    assert_eq!("reset", reset.into_source());
    assert_eq!(Some(4), main.input_index());
    assert_eq!(Some(crate::nonzero(2)), main.input_consumed());
    assert_eq!(None, finish.input_index());
    assert_eq!(None, finish.input_consumed());
    assert_eq!("finish", finish.into_source());

    assert_eq!(
        TranscodeDomainError::Reset { source: 5 },
        TranscodeDomainError::reset("reset").map_source(str::len),
    );
    assert_eq!(
        TranscodeDomainError::Main {
            source: 4,
            input_index: 4,
            input_consumed: Some(crate::nonzero(2)),
        },
        main.map_source(str::len),
    );
    assert_eq!(
        TranscodeDomainError::Finish { source: 6 },
        TranscodeDomainError::finish("finish").map_source(str::len),
    );
}
