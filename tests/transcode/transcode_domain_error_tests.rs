// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;
use std::io::Error as IoError;

use qubit_codec::TranscodeDomainError;

#[test]
fn test_domain_error_messages_and_error_source_expose_phase_context() {
    let reset = TranscodeDomainError::<IoError>::reset(IoError::other("reset detail"));
    let main = TranscodeDomainError::<IoError>::main(IoError::other("main detail"), 4);
    let finish = TranscodeDomainError::<IoError>::finish(IoError::other("finish detail"));

    assert_eq!(
        reset.to_string(),
        "codec reset error: reset detail",
        "the reset phase must name itself and forward the wrapped error message"
    );
    assert_eq!(
        main.to_string(),
        "codec main error at input index 4: main detail",
        "the main phase must name itself, the input index, and the wrapped error message"
    );
    assert_eq!(
        finish.to_string(),
        "codec finish error: finish detail",
        "the finish phase must name itself and forward the wrapped error message"
    );

    assert_eq!(
        Error::source(&reset).map(ToString::to_string).as_deref(),
        Some("reset detail"),
        "`#[source]` must expose the reset-phase domain error through the standard error chain"
    );
    assert_eq!(
        Error::source(&main).map(ToString::to_string).as_deref(),
        Some("main detail"),
        "`#[source]` must expose the main-phase domain error through the standard error chain"
    );
    assert_eq!(
        Error::source(&finish).map(ToString::to_string).as_deref(),
        Some("finish detail"),
        "`#[source]` must expose the finish-phase domain error through the standard error chain"
    );
}

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
