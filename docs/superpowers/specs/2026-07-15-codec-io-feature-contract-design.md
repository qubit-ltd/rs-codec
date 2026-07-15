# Codec I/O Feature and Release Contract Design

## Goal

Map `Complete` progress with unread input to the existing trailing-input
failure, enforce every remaining invalid `TranscodeProgress` as a checked
contract violation in every build, and move the `qubit-io` integration behind
an opt-in `io` feature so the default `qubit-codec` build contains only codec
and transcode primitives.

## Release progress contract

`Transcoder::transcode_complete_into` currently validates returned progress
with `debug_assert!`. The validation therefore disappears from ordinary
release builds even though the helper relies on the reported counters and
status to interpret the result.

Before assertion validation, `Complete` progress that leaves unread input is
intentionally mapped to `TranscodeFailure::trailing_input`. That outcome is a
recoverable one-shot input failure, not a panic. The helper will then use
`assert!` for every remaining invalid progress state. A `Transcoder`
implementation that returns one of those remaining internally inconsistent
progress states will panic with `Transcoder::transcode returned invalid
progress` in debug and release builds. This remains a programmer-contract
violation rather than a recoverable domain or framework error, so no public
error type or method signature changes.

The existing over-reporting regression fixture and test will no longer be
compiled only under `debug_assertions`. The release test must demonstrate the
old behavior before the assertion is changed, then pass after the change.

## Cargo feature boundary

`qubit-codec` will declare:

```toml
[features]
default = []
io = ["dep:qubit-io"]
```

The `qubit-io` dependency will be optional. The `io` feature will control:

- the private `transcode::io` module;
- the `TranscodeDecodeInput` and `TranscodeEncodeOutput` re-exports; and
- the I/O bridge integration-test module.

No compatibility default, forwarding alias, deprecated export, or fallback
module will be provided. Building without `io` intentionally removes the two
I/O bridge types from the public API and removes `qubit-io` from the normal
dependency graph.

## Core `UncheckedSlice` removal

Three non-I/O production call sites currently keep `qubit-io` in the core
dependency graph. They will be replaced without changing their contracts:

1. `EncodeState::context_unchecked` will replace
   `UncheckedSlice::get(input, input_index)` with
   `input.get_unchecked(input_index)`. The caller already proves the cursor is
   in bounds with `state.has_input()`.
2. `TranscodeDecodeEngine::transcode` will replace
   `UncheckedSlice::write(output, output_index, value)` with assignment through
   `output.get_unchecked_mut(output_index)`. The preceding
   `state.needs_output()` check already proves a writable slot exists.
3. `TranscodeFailure::ensure_output_range` will replace
   `UncheckedSlice::range_fits` with a remaining-capacity comparison after
   `ensure_output_index`. Because `output_index <= output_len` is already
   established, `range_len <= output_len - output_index` is overflow-safe and
   equivalent to the existing checked-add range test.

The first two operations retain their unsafe caller contracts and compile to
unchecked indexed access in release builds. `UncheckedSlice` use inside the
feature-gated I/O bridge remains unchanged because that module is itself an
integration with `qubit-io`.

Test-only `qubit_io::nz!` references outside the I/O bridge tests will use the
existing crate-local non-zero helper or `NonZeroUsize` directly. This keeps the
core-only test and documentation build independent of `qubit-io`; no
unconditional development dependency will be added.

## Downstream manifests

`rs-io-binary` and `rs-io-text` are direct users of the bridge types. Their
`qubit-codec` dependencies will explicitly enable `features = ["io"]`.
Duplicate normal and development dependency declarations in `rs-io-binary`
will both state the feature for manifest clarity.

`rs-codec-binary`, `rs-codec-text`, and `rs-codec-misc` do not use the bridge
types and will continue depending on the default core-only feature set.

## Documentation and CI

The English and Chinese README files will describe the `io` feature and mark
the two bridge types as feature-gated. Crate-level documentation will likewise
avoid presenting the bridges as unconditionally available.

A `.rs-ci-cargo-matrix.json` configuration will verify both explicit feature
sets:

- core-only with `defaultFeatures: false`; and
- I/O integration with `defaultFeatures: false` and `features: ["io"]`.

Each matrix entry will run tests, rustdoc, and Clippy. The normal dependency
tree of the core-only build must not contain `qubit-io`.

## Compatibility and verification

This is an intentionally breaking change. Existing users importing
`TranscodeDecodeInput` or `TranscodeEncodeOutput` without enabling `io` will no
longer compile. Existing `Transcoder` implementations that return invalid
progress other than the deliberate `Complete`-with-unread-input mapping may
now panic in release builds instead of allowing inconsistent progress to
continue.

Verification will include targeted red-green cycles, default and `io` tests in
debug and release modes, rustdoc and Clippy for both feature sets, a normal
dependency-tree check, and complete tests and Clippy runs for `rs-io-binary`
and `rs-io-text` with their updated manifests. No Git commit is part of this
task unless separately requested.
