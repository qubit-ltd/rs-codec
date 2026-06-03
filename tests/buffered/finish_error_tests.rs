/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
use qubit_codec::{
    CapacityError,
    FinishError,
};

#[test]
fn test_finish_error_constructors_create_expected_variants() {
    assert_eq!(
        FinishError::<&'static str>::Capacity {
            source: CapacityError::OutputLengthOverflow,
        },
        FinishError::capacity(CapacityError::OutputLengthOverflow),
    );
    assert_eq!(
        FinishError::<&'static str>::InvalidOutputIndex { index: 3, len: 2 },
        FinishError::invalid_output_index(3, 2),
    );
    assert_eq!(
        FinishError::<&'static str>::InsufficientOutput {
            output_index: 2,
            required: 4,
            available: 1,
        },
        FinishError::insufficient_output(2, 4, 1),
    );
    assert_eq!(FinishError::Source { source: "finish" }, FinishError::source("finish"));
}

#[test]
fn test_finish_error_maps_source_only_for_semantic_errors() {
    let mapped = FinishError::source("finish").map_source(str::len);
    assert_eq!(FinishError::Source { source: 6 }, mapped);

    let mapped = FinishError::<&'static str>::capacity(CapacityError::OutputLengthOverflow).map_source(str::len);
    assert_eq!(
        FinishError::<usize>::Capacity {
            source: CapacityError::OutputLengthOverflow,
        },
        mapped,
    );

    let mapped = FinishError::<&'static str>::invalid_output_index(3, 2).map_source(str::len);
    assert_eq!(FinishError::<usize>::InvalidOutputIndex { index: 3, len: 2 }, mapped);

    let mapped = FinishError::<&'static str>::insufficient_output(2, 4, 1).map_source(str::len);
    assert_eq!(
        FinishError::<usize>::InsufficientOutput {
            output_index: 2,
            required: 4,
            available: 1,
        },
        mapped,
    );
}

#[test]
fn test_finish_error_ensure_output_capacity_checks_index_and_remaining_len() {
    assert_eq!(Ok(()), FinishError::<()>::ensure_output_capacity(4, 4, 0));
    assert_eq!(Ok(()), FinishError::<()>::ensure_output_capacity(4, 1, 3));
    assert_eq!(
        Err(FinishError::InvalidOutputIndex { index: 5, len: 4 }),
        FinishError::<()>::ensure_output_capacity(4, 5, 0),
    );
    assert_eq!(
        Err(FinishError::InsufficientOutput {
            output_index: 2,
            required: 3,
            available: 2,
        }),
        FinishError::<()>::ensure_output_capacity(4, 2, 3),
    );
}
