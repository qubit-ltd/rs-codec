# Codec Lifecycle and Capacity Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> `superpowers:executing-plans` to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking. Repository instructions prohibit
> subagent delegation and do not authorize commits, so execute inline and leave
> changes uncommitted.

**Goal:** Correct stateful complete-value encoding, make capacity bounds global
across transient states, fix converter reset ordering, pass the configured
coverage gate, and synchronize public documentation and tests.

**Architecture:** Complete-value adapters reserve codec constants before reset
and perform exact validation afterward. `Transcoder` and hook bounds describe
the maximum for every reachable transient state; engines compose those bounds,
and converters explicitly account for one retained decoded value. Converter
reset initializes the target stream before converting source reset values.

**Tech Stack:** Rust 2024, cargo 1.94.0, nightly rustfmt/clippy, cargo-llvm-cov,
project integration tests under `tests/`.

## Global Constraints

- Preserve the user-owned `.rs-ci` submodule update.
- Do not change public method signatures or configured coverage thresholds.
- Put all Rust tests under `tests/`; do not add source-local test modules.
- Use explicit imports and document every added helper function.
- Do not run `git add`, `git commit`, or `git push`.

---

### Task 1: Make complete value encoding reset-state-correct

**Files:**

- Modify: `tests/value/codec_value_encoder_tests.rs`
- Modify: `tests/transcode/io/transcode_encode_output_tests.rs`
- Modify: `src/value/codec_value_lifecycle.rs`
- Modify: `src/value/codec_value_encoder.rs`
- Modify: `src/transcode/io/transcode_encode_output.rs`
- Modify: `src/codec/codec.rs`

**Interfaces:**

- Consumes: `Codec::{encode_reset, can_encode_value, encode_len, encode,
  encode_finish}` and codec maximum constants.
- Produces: `encode_complete_value_into_reserved` that accepts a conservative
  reserved length and queries domain/width only after reset.

- [ ] **Step 1: Add reset-sensitive codec regressions**

  Add test codecs whose pre-reset width/domain differ from their reset state.
  The core regression has `MAX_UNITS_PER_VALUE = 2`, reports width `1` before
  reset and width `2` afterward, and writes two units. Add equivalent assertions
  through `CodecValueEncoder::encode_into` and
  `TranscodeEncodeOutput::write_encoded_with`.

- [ ] **Step 2: Run the focused tests and verify RED**

  Run:

  ```bash
  cargo +1.94.0 test --all-features \
    value::codec_value_encoder_tests::test_codec_value_encoder_queries_width_after_reset
  cargo +1.94.0 test --all-features \
    transcode::io::transcode_encode_output_tests::test_write_encoded_with_queries_width_after_reset
  ```

  Expected: the old pre-reset exact reservation panics or rejects the value.

- [ ] **Step 3: Replace pre-reset exact sizing with conservative reservation**

  Remove `complete_encode_len`. Make both adapters call
  `max_complete_encode_units::<C>()`, reserve that many units, and pass the
  conservative bound to the lifecycle helper. Inside the helper run reset,
  then `can_encode_value`, then `encode_len`, assert that the exact width is no
  greater than `C::MAX_UNITS_PER_VALUE`, encode, finish, and return actual
  output. Update safety comments and `Codec` documentation accordingly.

- [ ] **Step 4: Run focused tests and verify GREEN**

  Run both commands from Step 2 plus:

  ```bash
  cargo +1.94.0 test --all-features value::codec_value_encoder_tests
  cargo +1.94.0 test --all-features \
    transcode::io::transcode_encode_output_tests
  ```

  Expected: all selected tests pass with no warnings.

### Task 2: Make capacity bounds global across transient states

**Files:**

- Modify: `src/transcode/transcoder.rs`
- Modify: `src/transcode/engine/transcode_decode_hooks.rs`
- Modify: `src/transcode/engine/transcode_encode_hooks.rs`
- Modify: `src/transcode/engine/transcode_decode_engine.rs`
- Modify: `src/transcode/engine/transcode_encode_engine.rs`
- Modify: `src/transcode/engine/transcode_convert_engine.rs`
- Modify: `src/transcode/adapter/codec_transcode_converter.rs`
- Modify: `src/transcode/adapter/codec_transcode_decoder.rs`
- Modify: `src/transcode/adapter/codec_transcode_encoder.rs`
- Modify: `tests/transcode/transcoder_tests.rs`
- Modify: `tests/transcode/engine/transcode_decode_engine_tests.rs`
- Modify: `tests/transcode/engine/transcode_encode_engine_tests.rs`
- Modify: `tests/transcode/engine/transcode_convert_engine_tests.rs`

**Interfaces:**

- Consumes: existing capacity method signatures.
- Produces: configuration-dependent, transient-state-independent upper bounds;
  `max_total_output_len` remains a checked sum and becomes valid before a
  complete stream from any prior state.

- [ ] **Step 1: Add global-bound regressions**

  Add a stateful test transcoder that may retain a one-byte checksum. Assert
  `max_finish_output_len() == Ok(1)` before input, while pending, and after
  finish. Update stateful hook fixtures so their maximum methods return their
  configured maximum rather than `usize::from(current_pending)` or a remaining
  count. Add a converter assertion that streaming and finish bounds include one
  possible retained decoded value even when `pending` is currently empty.

- [ ] **Step 2: Run focused tests and verify RED**

  Run:

  ```bash
  cargo +1.94.0 test --all-features \
    transcode::transcoder_tests::test_transcoder_capacity_bounds_are_global
  cargo +1.94.0 test --all-features \
    transcode::engine::transcode_convert_engine_tests::test_buffered_convert_engine_bounds_include_possible_pending_value
  ```

  Expected: converter bounds omit the possible pending value under the old
  state-dependent implementation.

- [ ] **Step 3: Implement and document global bounds**

  Rewrite the trait and hook documentation to prohibit transient-state-
  dependent bounds. In `TranscodeConvertEngine`, replace
  `pending_output_len()` in capacity calculations with a checked global bound
  for one possible pending target value. Bound source reset/finish values by
  checked multiplication with `E::MAX_UNITS_PER_VALUE`, then add target
  reset/finish bounds. Make streaming conversion include one possible pending
  value for every state. Keep actual write counts state-sensitive.

- [ ] **Step 4: Run engine and trait tests and verify GREEN**

  Run:

  ```bash
  cargo +1.94.0 test --all-features transcode::transcoder_tests
  cargo +1.94.0 test --all-features \
    transcode::engine::transcode_decode_engine_tests
  cargo +1.94.0 test --all-features \
    transcode::engine::transcode_encode_engine_tests
  cargo +1.94.0 test --all-features \
    transcode::engine::transcode_convert_engine_tests
  ```

  Expected: all selected tests pass and overflow assertions still return
  `CapacityError::OutputLengthOverflow`.

### Task 3: Initialize target state before converting source reset values

**Files:**

- Modify: `tests/transcode/engine/transcode_convert_engine_tests.rs`
- Modify: `src/transcode/engine/transcode_convert_engine.rs`

**Interfaces:**

- Consumes: `TranscodeEncodeEngine::reset` and `drain_decoder_reset`.
- Produces: converter reset output ordered as target reset output followed by
  encoded source reset values.

- [ ] **Step 1: Add a combined reset-order regression**

  Add a source codec that emits one reset value and a target codec that writes
  a reset marker and changes the encoding of that value after reset. Assert the
  output starts with the target marker and that the source reset value uses the
  reset target state. Replace the old test that explicitly expected decode
  reset before encode reset.

- [ ] **Step 2: Run the focused test and verify RED**

  Run:

  ```bash
  cargo +1.94.0 test --all-features \
    transcode::engine::transcode_convert_engine_tests::test_buffered_convert_engine_resets_target_before_encoding_source_reset_values
  ```

  Expected: output order or target-state assertion fails.

- [ ] **Step 3: Reorder converter reset**

  After clearing pending state, create `ConvertState`, reset the target encode
  engine at the initial output cursor, advance by its actual output, and then
  call `drain_decoder_reset`. Update lifecycle documentation and comments.

- [ ] **Step 4: Run the focused and converter suites and verify GREEN**

  Run the Step 2 command and the complete converter-engine test module.

### Task 4: Close the enforced per-source coverage gaps

**Files:**

- Modify: `tests/transcode/transcoder_tests.rs`
- Modify: `tests/transcode/engine/transcode_convert_engine_tests.rs`
- Modify: `tests/transcode/engine/transcode_decode_engine_tests.rs`
- Modify: `tests/transcode/io/transcode_decode_input_tests.rs`
- Modify: `tests/transcode/io/transcode_encode_output_tests.rs`
- Modify: `tests/value/codec_value_decoder_tests.rs`
- Modify: `tests/value/codec_value_encoder_tests.rs`

**Interfaces:**

- Consumes: public APIs and existing test fixtures only.
- Produces: behavior-based coverage above every configured per-source
  threshold without exclusions or threshold changes.

- [ ] **Step 1: Generate the post-refactor coverage report**

  Run:

  ```bash
  env COVERAGE_OPEN_HTML=0 ./coverage.sh json
  cargo +1.94.0 llvm-cov report --text --show-instantiations \
    --show-missing-lines --output-path target/llvm-cov/missing.txt
  ```

  Expected: the command identifies the remaining failing source files and
  unexecuted generic/error branches.

- [ ] **Step 2: Add targeted behavioral cases**

  Cover the remaining concrete paths: default trait reset/finish and each
  checked-add failure in `Transcoder`; converter reset/finish capacity and
  pending paths; decode/encode I/O direct-buffer and scratch fallback paths,
  invalid indices, EOF/incomplete progress, flush/seek failures, and error
  mapping; value-adapter append/rollback and lifecycle contract paths. Reuse
  existing in-memory I/O fixtures and assert exact errors and cursor/output
  state.

- [ ] **Step 3: Re-run coverage until the configured gate passes**

  Run `./coverage.sh json` after each focused batch. Completion requires every
  source to report functions `>= 100%`, lines `> 95%`, and regions `> 95%`.

### Task 5: Synchronize README content and remove no-op tests

**Files:**

- Modify: `README.md`
- Modify: `README.zh_CN.md`
- Modify: `tests/transcode/mod.rs`
- Modify: `tests/transcode/engine/encode_context_tests.rs`
- Modify: `tests/transcode/engine/mod.rs`
- Modify: `tests/transcode/internal/mod.rs`
- Delete: `tests/transcode/capacity_error_tests.rs`
- Delete: `tests/transcode/internal/convert_state_tests.rs`
- Delete: `tests/transcode/internal/decode_state_tests.rs`
- Delete: `tests/transcode/internal/encode_state_tests.rs`
- Delete: `tests/transcode/internal/pending_value_tests.rs`
- Delete: `tests/transcode/internal/pending_value_slot_tests.rs`

**Interfaces:**

- Consumes: exports from `src/lib.rs`, `src/transcode/mod.rs`, and
  `src/value/mod.rs`.
- Produces: README type tables/examples that compile conceptually against the
  real API and a test module tree containing only behavioral tests.

- [ ] **Step 1: Replace stale error names and hierarchy descriptions**

  Replace `TranscodeError`, `ConvertError`, `CodecValueEncodeError`, and
  `CodecValueDecodeError` references with the actual framework/domain and
  directional error families. Keep English and Chinese READMEs structurally
  aligned.

- [ ] **Step 2: Remove no-op tests**

  Delete the six files whose only test is `test_module_compiles`, remove their
  module declarations, and remove only the no-op function from
  `encode_context_tests.rs`, preserving its behavioral test.

- [ ] **Step 3: Verify documentation and test discovery**

  Run:

  ```bash
  rg -n "ConvertError|TranscodeError|CodecValueEncodeError|CodecValueDecodeError|test_module_compiles" \
    README.md README.zh_CN.md tests
  RUSTDOCFLAGS="-D warnings" cargo +1.94.0 doc --no-deps
  cargo +1.94.0 test --all-features --no-fail-fast
  ```

  Expected: `rg` finds no stale standalone names or no-op tests; docs and tests
  succeed.

### Task 6: Run complete and downstream verification

**Files:**

- Verify only; do not modify unrelated downstream sources.

**Interfaces:**

- Consumes: the completed `rs-codec` implementation.
- Produces: fresh evidence for formatting, lint, build, tests, coverage, docs,
  and direct downstream compatibility.

- [ ] **Step 1: Run the complete crate CI**

  Run `./ci-check.sh`. Expected: all 11 checks pass, including the per-source
  coverage gate.

- [ ] **Step 2: Run direct downstream all-feature tests**

  From each sibling crate, run `cargo +1.94.0 test --all-features`. Required
  crates are `rs-codec-binary`, `rs-codec-text`, `rs-codec-misc`,
  `rs-io-binary`, and `rs-io-text`.

- [ ] **Step 3: Inspect the final diff and preserve user changes**

  Run `git status --short` and `git --no-pager diff -- . ':!.rs-ci'`, then
  separately inspect `git --no-pager diff --submodule=short -- .rs-ci`.
  Confirm the existing `.rs-ci` pointer update remains unchanged and every
  other change maps to this plan.
