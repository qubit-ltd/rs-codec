# Qubit Codec

[![Rust CI](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-codec/coverage-badge.json)](https://qubit-ltd.github.io/rs-codec/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-codec.svg?color=blue)](https://crates.io/crates/qubit-codec)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

面向 Rust 的核心 codec trait 和缓冲区转换原语。

## 概述

Qubit Codec 是 Qubit codec 系列 crate 的领域无关基础层。它只放 binary、
text、misc 和 I/O adapter crate 需要共享的小型 trait 与值类型，不引入
`std::io` stream helper，也不放具体格式实现。

本库提供：

- 用于完整值转换的 `Encoder`、`Decoder` 和 `Codec` trait。
- 用于调用方管理缓冲区转换的 `Coder`、`CoderProgress` 和 `CoderStatus`。
- 供 binary 与 text codec 共享的 `ByteOrder`、`ByteOrderSpec`、
  `BigEndian` 和 `LittleEndian`。

具体 codec 位于相邻 crate，例如 `qubit-codec-binary`、
`qubit-codec-text` 和 `qubit-codec-misc`。

## 安装

```toml
[dependencies]
qubit-codec = "0.1"
```

## 快速示例

```rust
use qubit_codec::{
    CoderProgress,
    CoderStatus,
    Encoder,
};

struct StringEncoder;

impl Encoder<str> for StringEncoder {
    type Output = String;
    type Error = core::convert::Infallible;

    fn encode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

let encoded = Encoder::<str>::encode(&StringEncoder, "codec")?;
assert_eq!("codec", encoded);

let progress = CoderProgress::complete(3, 4);
assert_eq!(CoderStatus::Complete, progress.status());

# Ok::<(), core::convert::Infallible>(())
```

## 库边界

`qubit-codec` 不包含具体 binary 格式、字符集、percent/Base64/hex codec，
也不包含 `std::io` reader/writer adapter。这些能力应放在领域 crate 中，
让下游只依赖自己需要的层。

## 开发

```bash
./align-ci.sh
RS_CI_SKIP_TOOLCHAIN_UPDATE=1 ./ci-check.sh
```

## 许可证

根据 Apache License 2.0 授权。完整许可证文本见 [LICENSE](LICENSE)。
