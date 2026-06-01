use qubit_codec::TranscodeStatus;

#[test]
fn test_transcoder_status_variants_are_distinct() {
    assert_ne!(
        TranscodeStatus::Complete,
        TranscodeStatus::NeedInput {
            input_index: 0,
            additional: super::nz(1),
            available: 0
        }
    );
    assert_ne!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            additional: super::nz(1),
            available: 0
        },
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: super::nz(1),
            available: 0,
        }
    );
}

#[test]
fn test_transcoder_status_constructors_create_expected_variants() {
    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 4,
            additional: super::nz(2),
            available: 1,
        },
        TranscodeStatus::need_input(4, super::nz(2), 1),
    );
    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 7,
            additional: super::nz(3),
            available: 0,
        },
        TranscodeStatus::need_output(7, super::nz(3), 0),
    );
}
