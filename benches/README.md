# Transcode benchmarks

Run the safe/unchecked indexing A/B benchmark with:

```bash
cargo bench --manifest-path rs-codec/Cargo.toml --bench transcode -- safe_vs_unchecked
```

The benchmark compares the same copy loop implemented with safe indexing and
explicit `get_unchecked` access. It is a code-generation probe, not
a correctness test; the safe version is the default choice unless assembly
shows a measurable regression in a real hot path.

Emit optimized assembly for inspection with:

```bash
cargo rustc --manifest-path rs-codec/Cargo.toml --release --bench transcode -- --emit=asm
```

Inspect the generated `target/release/deps/transcode-*.s` files and compare the
`copy_safe` and `copy_unchecked` symbols. On toolchains with `cargo-asm`, the
equivalent focused commands are:

```bash
cargo asm --manifest-path rs-codec/Cargo.toml --bench transcode copy_safe
cargo asm --manifest-path rs-codec/Cargo.toml --bench transcode copy_unchecked
```
