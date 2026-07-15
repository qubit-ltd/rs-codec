# Codec I/O Feature and Release Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Map `Complete` progress with unread input to trailing-input failure, enforce all remaining invalid transcode progress contracts in release builds, and make the `qubit-io` bridge an opt-in, non-default `io` feature used explicitly by the two I/O downstream crates.

**Architecture:** Keep codec, value, and transcode engines in the default dependency-free core while compiling the existing I/O bridge module only with `io`. Replace three core-only `UncheckedSlice` calls with equivalent standard slice operations, and validate both feature sets plus downstream manifests independently.

**Tech Stack:** Rust 1.94, Cargo features, integration tests, rustdoc, Clippy, rs-ci Cargo feature matrix.

## Global Constraints

- `default = []`; `io` is never enabled implicitly.
- The change is intentionally breaking; add no compatibility aliases, default feature, or deprecated forwarding surface.
- `rs-io-binary` and `rs-io-text` must explicitly enable `features = ["io"]`.
- Leave the deferred generic-bound, convenience-adapter, and benchmark work untouched.
- Preserve all pre-existing user changes, especially changes outside the three target repositories.
- Do not run `git add`, `git commit`, or `git push`; replace commit checkpoints with status and diff reviews.
- Follow test-first red-green cycles for behavior changes.

---

### Task 1: Enforce Progress Validation in Release Builds

**Files:**
- Modify: `tests/transcode/transcoder_tests.rs`
- Modify: `src/transcode/transcoder.rs:20-64`

**Interfaces:**
- Consumes: `TranscodeProgress::validate` and the existing over-reporting test transcoder.
- Produces: `Transcoder::transcode_complete_into` returns
  `TranscodeFailure::trailing_input` for `Complete` progress with unread input,
  then panics on every other invalid progress state in every build profile.

**Progress classification:** Before assertion validation, deliberately map a
`Complete` status with unread input to `TranscodeFailure::trailing_input`.
This recoverable input failure does not panic. After that mapping, use the
release `assert!` for every remaining invalid progress state, which panics in
both debug and release builds.

- [ ] **Step 1: Make the existing regression fixture available in release tests**

Remove `#[cfg(debug_assertions)]` from `OverreportingCompleteTranscoder`, its `Transcoder` implementation, and the progress-validation test. Rename the test to:

```rust
#[test]
#[should_panic(expected = "Transcoder::transcode returned invalid progress")]
fn test_transcoder_transcode_complete_into_validates_progress() {
    let mut transcoder = OverreportingCompleteTranscoder;
    let mut output = [];

    let _ = transcoder.transcode_complete_into(b"", &mut output);
}
```

- [ ] **Step 2: Verify RED in release mode**

Run:

```bash
cargo test --release test_transcoder_transcode_complete_into_validates_progress
```

Expected: FAIL because the `#[should_panic]` test returns normally when the current `debug_assert!` is compiled out.

- [ ] **Step 3: Replace the debug-only assertion**

In `complete_progress_written`, change only the assertion kind:

```rust
assert!(
    progress
        .validate(
            0,
            input_len,
            output_index,
            output_len.saturating_sub(output_index),
        )
        .is_ok(),
    "Transcoder::transcode returned invalid progress",
);
```

- [ ] **Step 4: Verify GREEN in both profiles**

Run:

```bash
cargo test test_transcoder_transcode_complete_into_validates_progress
cargo test --release test_transcoder_transcode_complete_into_validates_progress
```

Expected: both targeted tests PASS by observing the expected panic.

- [ ] **Step 5: Review the Task 1 diff**

Run:

```bash
git --no-pager diff -- src/transcode/transcoder.rs tests/transcode/transcoder_tests.rs
git status --short
```

Expected: only the assertion and debug-only test gating/name have changed, plus the already approved untracked design and plan documents.

---

### Task 2: Isolate the Core Crate from `qubit-io`

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/lib.rs`
- Modify: `src/transcode/mod.rs`
- Modify: `src/transcode/internal/encode_state.rs`
- Modify: `src/transcode/engine/transcode_decode_engine.rs`
- Modify: `src/transcode/transcode_failure.rs`
- Modify: `src/transcode/transcoder.rs`
- Modify: `tests/transcode/mod.rs`
- Modify: `tests/codec/decode_failure_tests.rs`
- Modify: `tests/transcode/adapter/codec_transcode_converter_tests.rs`
- Modify: `tests/transcode/adapter/codec_transcode_decode_hooks_tests.rs`
- Modify: `tests/transcode/adapter/codec_transcode_decoder_tests.rs`
- Modify: `tests/transcode/engine/transcode_decode_engine_tests.rs`
- Modify: `tests/transcode/engine/decode_invalid_action_tests.rs`
- Modify: `tests/transcode/engine/encode_outcome_tests.rs`
- Modify: `tests/transcode/engine/decode_outcome_tests.rs`
- Modify: `tests/prelude_tests.rs`
- Modify: `tests/value/codec_value_decoder_tests.rs`

**Interfaces:**
- Consumes: existing `transcode::io` module and crate-local `tests::nz` helper.
- Produces: core-only `qubit-codec` with no normal `qubit-io` dependency, plus opt-in `TranscodeDecodeInput` and `TranscodeEncodeOutput` exports under `io`.

- [ ] **Step 1: Verify the requested feature does not yet exist**

Run:

```bash
cargo check --features io
```

Expected: FAIL with Cargo reporting that `qubit-codec` does not contain feature `io`.

- [ ] **Step 2: Declare the feature and optional dependency**

Add before `[dependencies]`:

```toml
[features]
default = []
io = ["dep:qubit-io"]
```

Change the dependency to:

```toml
qubit-io = { version = "0.13", optional = true }
```

Do not add an unconditional `dev-dependencies` entry.

- [ ] **Step 3: Gate the I/O module, exports, and tests**

In `src/transcode/mod.rs`:

```rust
#[cfg(feature = "io")]
mod io;

#[cfg(feature = "io")]
pub use io::{
    TranscodeDecodeInput,
    TranscodeEncodeOutput,
};
```

In `src/lib.rs`, remove the two bridge types from the unconditional grouped
`pub use transcode::{...}` and add:

```rust
#[cfg(feature = "io")]
pub use transcode::{
    TranscodeDecodeInput,
    TranscodeEncodeOutput,
};
```

In `tests/transcode/mod.rs`:

```rust
#[cfg(feature = "io")]
mod io;
```

- [ ] **Step 4: Replace the three core-only `UncheckedSlice` calls**

In `EncodeState::context_unchecked`:

```rust
let value = unsafe { input.get_unchecked(input_index) };
```

Remove `use qubit_io::UncheckedSlice;` from that file.

In the decode engine output closure:

```rust
unsafe {
    *output.get_unchecked_mut(output_index) = value;
}
```

Remove `use qubit_io::UncheckedSlice;` from that file.

In `TranscodeFailure::ensure_output_range`:

```rust
Self::ensure_output_index(output_len, output_index)?;
let available = output_len - output_index;
if range_len > available {
    return Err(Self::invalid_output_index(output_index, output_len));
}
```

Keep the existing `range_len < required` error behavior unchanged.

- [ ] **Step 5: Remove non-I/O test and doctest references to `qubit-io`**

In the listed integration-test files, mechanically replace each:

```rust
qubit_io::nz!(N)
```

with:

```rust
crate::nz(N)
```

In the `Transcoder` rustdoc example, replace both `qubit_io::nz!(2)` calls with:

```rust
NonZeroUsize::new(2).expect("two is non-zero")
```

Do not alter `qubit_io` references under `src/transcode/io` or
`tests/transcode/io`; those are part of the feature-gated bridge.

- [ ] **Step 6: Verify GREEN for core-only and I/O builds**

Run:

```bash
cargo test --no-default-features
cargo test --no-default-features --features io
cargo test --release --no-default-features
cargo test --release --no-default-features --features io
```

Expected: all four commands PASS; I/O bridge tests run only in the feature-enabled commands.

- [ ] **Step 7: Prove the core normal dependency graph excludes `qubit-io`**

Run:

```bash
if cargo tree --no-default-features -e normal --prefix none | rg -q '^qubit-io '; then
    exit 1
fi
```

Expected: exit 0 because no normal core-only dependency is named `qubit-io`.

- [ ] **Step 8: Review the Task 2 diff**

Run:

```bash
git --no-pager diff -- Cargo.toml src tests
git status --short
```

Expected: only feature gating, equivalent core slice operations, test helper references, and the release assertion work are present.

---

### Task 3: Document and Continuously Verify Both Feature Sets

**Files:**
- Create: `.rs-ci-cargo-matrix.json`
- Modify: `README.md`
- Modify: `README.zh_CN.md`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: rs-ci feature-matrix schema version 1.
- Produces: user-visible feature documentation and CI checks for core-only and I/O builds.

- [ ] **Step 1: Add the Cargo feature matrix**

Create `.rs-ci-cargo-matrix.json` with:

```json
{
  "version": 1,
  "checks": [
    {
      "name": "core-only",
      "defaultFeatures": false,
      "commands": ["test", "doc", "clippy"]
    },
    {
      "name": "io",
      "defaultFeatures": false,
      "features": ["io"],
      "commands": ["test", "doc", "clippy"]
    }
  ]
}
```

- [ ] **Step 2: Validate the new matrix configuration**

Run:

```bash
./.rs-ci/cargo-feature-check.sh validate
```

Expected: PASS with `Cargo feature matrix config is valid`.

- [ ] **Step 3: Update crate documentation**

In `src/lib.rs`, annotate the layer overview so the bridge layer reads:

```text
TranscodeDecodeInput / TranscodeEncodeOutput  [feature = "io"]
```

Do not introduce intra-doc links to feature-disabled items in unconditional documentation.

- [ ] **Step 4: Update both READMEs**

Add a Cargo Features section stating that the default set is empty and `io`
enables the `qubit-io` bridge. Mark every type-list, boundary, and dependency
description of `TranscodeDecodeInput` and `TranscodeEncodeOutput` as requiring
`io`. Keep the English and Chinese content semantically aligned.

- [ ] **Step 5: Verify docs and lint for both feature sets**

Run:

```bash
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --no-default-features
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --no-default-features --features io
cargo clippy --all-targets --no-default-features -- -D warnings
cargo clippy --all-targets --no-default-features --features io -- -D warnings
```

Expected: all four commands PASS with no warnings.

- [ ] **Step 6: Review the Task 3 diff**

Run:

```bash
git --no-pager diff -- .rs-ci-cargo-matrix.json README.md README.zh_CN.md src/lib.rs
git status --short
```

Expected: the matrix and feature documentation match the approved design without unrelated README edits.

---

### Task 4: Enable `io` in Direct I/O Consumers

**Files:**
- Modify: `../rs-io-binary/Cargo.toml`
- Modify: `../rs-io-text/Cargo.toml`

**Interfaces:**
- Consumes: `qubit-codec` feature `io`.
- Produces: both downstream crates continue importing the bridge types explicitly.

- [ ] **Step 1: Verify RED in each downstream crate before manifest updates**

After Task 2, run:

```bash
cargo check --manifest-path ../rs-io-binary/Cargo.toml
cargo check --manifest-path ../rs-io-text/Cargo.toml
```

Expected: each command FAILS with unresolved imports for
`TranscodeDecodeInput` and/or `TranscodeEncodeOutput` because `io` is not yet enabled.

- [ ] **Step 2: Enable `io` in `rs-io-binary`**

Update both its normal and development declarations to:

```toml
qubit-codec = { path = "../rs-codec", version = "0.10.0", features = ["io"] }
```

- [ ] **Step 3: Enable `io` in `rs-io-text`**

Update its declaration to:

```toml
qubit-codec = { path = "../rs-codec", version = "0.10.0", features = ["io"] }
```

- [ ] **Step 4: Verify GREEN in each downstream crate**

Run:

```bash
cargo test --manifest-path ../rs-io-binary/Cargo.toml
cargo clippy --manifest-path ../rs-io-binary/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path ../rs-io-text/Cargo.toml
cargo clippy --manifest-path ../rs-io-text/Cargo.toml --all-targets -- -D warnings
```

Expected: all four commands PASS.

- [ ] **Step 5: Review each downstream repository separately**

Run:

```bash
git -C ../rs-io-binary --no-pager diff -- Cargo.toml
git -C ../rs-io-binary status --short
git -C ../rs-io-text --no-pager diff -- Cargo.toml
git -C ../rs-io-text status --short
```

Expected: each repository contains only its own manifest change.

---

### Task 5: Full Verification and Final Review

**Files:**
- Verify all files changed by Tasks 1-4.

**Interfaces:**
- Consumes: completed implementation and updated manifests.
- Produces: fresh evidence for tests, formatting, lint, documentation, coverage, dependency isolation, and clean change scope.

- [ ] **Step 1: Check formatting**

Run in `rs-codec`:

```bash
cargo fmt --all --check
```

Run in each downstream repository:

```bash
cargo fmt --manifest-path ../rs-io-binary/Cargo.toml --all --check
cargo fmt --manifest-path ../rs-io-text/Cargo.toml --all --check
```

Expected: all commands PASS without modifying unrelated files.

- [ ] **Step 2: Re-run complete core verification**

Run:

```bash
cargo test --no-default-features
cargo test --no-default-features --features io
cargo test --release --no-default-features
cargo test --release --no-default-features --features io
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --no-default-features
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --no-default-features --features io
cargo clippy --all-targets --no-default-features -- -D warnings
cargo clippy --all-targets --no-default-features --features io -- -D warnings
```

Expected: every command exits 0 with no failed tests or warnings.

- [ ] **Step 3: Run the established coverage gate**

Run:

```bash
./coverage.sh json --clean
```

Expected: exit 0 with the repository's per-source function, line, and region thresholds satisfied.

- [ ] **Step 4: Re-run downstream verification**

Run:

```bash
cargo test --manifest-path ../rs-io-binary/Cargo.toml
cargo clippy --manifest-path ../rs-io-binary/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path ../rs-io-text/Cargo.toml
cargo clippy --manifest-path ../rs-io-text/Cargo.toml --all-targets -- -D warnings
```

Expected: all downstream tests and lint checks exit 0.

- [ ] **Step 5: Verify dependency isolation**

Run:

```bash
if cargo tree --no-default-features -e normal --prefix none | rg -q '^qubit-io '; then
    exit 1
fi
cargo tree --no-default-features --features io -e normal --prefix none | rg '^qubit-io '
```

Expected: core-only exits 0 without a match; the feature-enabled command prints exactly one `qubit-io` package line.

- [ ] **Step 6: Review final repository state without committing**

Run:

```bash
git --no-pager diff --check
git --no-pager diff --stat
git status --short
git -C ../rs-io-binary --no-pager diff --check
git -C ../rs-io-binary --no-pager diff --stat
git -C ../rs-io-binary status --short
git -C ../rs-io-text --no-pager diff --check
git -C ../rs-io-text --no-pager diff --stat
git -C ../rs-io-text status --short
```

Expected: no whitespace errors; only the approved `rs-codec`, `rs-io-binary`, and `rs-io-text` changes are present; no commit is created.
