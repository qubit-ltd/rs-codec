# Qubit Codec（`rs-codec`）

[![Rust CI](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-codec/coverage-badge.json)](https://qubit-ltd.github.io/rs-codec/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-codec.svg?color=blue)](https://crates.io/crates/qubit-codec)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-codec` 帮助 Rust 格式 crate 与 adapter crate 在实现「逻辑 value ↔ 编码 unit」
规则之后，仍能用同一套表示层公开自有输出 API、调用方缓冲区流式转换，以及可选的策略
hooks。若每一层各自处理下标、容量上界、进度和 reset/finish 生命周期，同一格式很容易
长出互不兼容的公开面。本库统一这些机制，领域编码与错误仍由格式 crate 拥有。本库不
提供现成的 JSON、hex、Base64、percent encoding 或通用 `std::io` 扩展。

## 二进制格式 crate 实战场景

某协议 crate 需要：`0x1234` 编码为 `[0x12, 0x34]`，反向解码一致；只剩 1 个字节时应
报告不完整，而不是进入 unsafe decode；还要能把一批 value 写入调用方拥有的缓冲区——且
只实现一次 `Codec`。只需单次自有转换的调用方走 `CodecValueEncoder` /
`CodecValueDecoder`；自己管理目标缓冲区的调用方走 `CodecTranscodeEncoder`，无须重复
推导容量算术。

## 安装

```toml
[dependencies]
qubit-codec = "0.16"
```

默认 feature 集为空。只有使用 `qubit-io` 缓冲 bridge 时才启用 `io`；全局 value codec
注册表和 `register_value_string_codec!` / `register_value_bytes_codec!` 宏需要启用 `registry`。两项 feature 相互独立：

```toml
[dependencies]
qubit-io = "0.17"
qubit-codec = { version = "0.16", features = ["io"] }
```

```toml
[dependencies]
qubit-codec = { version = "0.16", features = ["registry"] }
```

## 快速开始

表示规则只写一次，再暴露自有编码：

```rust
use core::{convert::Infallible, num::NonZeroUsize};
use qubit_codec::{Codec, CodecValueEncoder, DecodeFailure, ValueEncoder};

#[derive(Default)]
struct U16BeCodec;

impl Codec for U16BeCodec {
    type Value = u16;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u16, NonZeroUsize), DecodeFailure<Infallible>> {
        debug_assert!(input_index + 2 <= input.len());
        let value = u16::from_be_bytes([
            input[input_index],
            input[input_index + 1],
        ]);
        Ok((value, NonZeroUsize::new(2).expect("two is non-zero")))
    }

    unsafe fn encode(
        &mut self,
        value: &u16,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Infallible> {
        debug_assert!(output_index + 2 <= output.len());
        output[output_index..output_index + 2]
            .copy_from_slice(&value.to_be_bytes());
        Ok(2)
    }
}

let mut encoder = CodecValueEncoder::new(U16BeCodec);
let bytes = encoder.encode(&0x1234).expect("encoding is infallible");
assert_eq!(vec![0x12, 0x34], bytes);
```

当 codec 在 reset 或 finish 阶段也会输出 value 时，严格单值 decode 不够用；可运行
示例见 [`examples/decode_lifecycle.rs`](examples/decode_lifecycle.rs)。定宽场景的自有
decode、不完整输入、增量 transcode、策略 hooks 与可选 `qubit-io` 集成见
[用户手册](doc/user_guide.zh_CN.md)。

下面按模块划分同一 `U16BeCodec` 的三种公开方式：核心规则、自有单值 API、批量写入
调用方缓冲区。

核心 codec 定义表示规则，供各层 adapter 复用：

```rust
// src/u16_be.rs
use core::{convert::Infallible, num::NonZeroUsize};
use qubit_codec::{Codec, DecodeFailure};

#[derive(Clone, Copy, Debug, Default)]
pub struct U16BeCodec;

impl Codec for U16BeCodec {
    type Value = u16;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u16, NonZeroUsize), DecodeFailure<Infallible>> {
        debug_assert!(input_index + 2 <= input.len());
        let value = u16::from_be_bytes([
            input[input_index],
            input[input_index + 1],
        ]);
        Ok((value, NonZeroUsize::new(2).expect("two is non-zero")))
    }

    unsafe fn encode(
        &mut self,
        value: &u16,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Infallible> {
        debug_assert!(output_index + 2 <= output.len());
        output[output_index..output_index + 2]
            .copy_from_slice(&value.to_be_bytes());
        Ok(2)
    }
}
```

自有输出层包装 checked facade，适合「给我一个 `Vec<u8>` / 一个 `u16`」的调用方：

```rust
// src/owned.rs
use qubit_codec::{CodecValueDecoder, CodecValueEncoder, ValueDecoder, ValueEncoder};

use crate::u16_be::U16BeCodec;

pub fn encode_u16(value: u16) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut encoder = CodecValueEncoder::new(U16BeCodec);
    Ok(encoder.encode(&value)?)
}

pub fn decode_u16(bytes: &[u8]) -> Result<u16, Box<dyn std::error::Error>> {
    let mut decoder = CodecValueDecoder::new(U16BeCodec);
    Ok(decoder.decode(bytes)?)
}
```

流式层在调用方缓冲区上完成整段生命周期，适合连续多个 value：

```rust
// src/batch.rs
use qubit_codec::{CodecTranscodeEncoder, Transcoder};

use crate::u16_be::U16BeCodec;

pub fn encode_batch(values: &[u16]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut encoder = CodecTranscodeEncoder::new(U16BeCodec);
    let capacity = encoder
        .max_total_output_len(values.len())
        .expect("capacity arithmetic should not overflow");
    let mut output = vec![0_u8; capacity];
    let written = encoder.transcode_complete_into(values, &mut output)?;
    output.truncate(written);
    Ok(output)
}
```

checked adapter 会在进入 unsafe `Codec` 入口前建立下标与容量前置条件，因此 1 字节输入
会表现为不完整或 transcode 失败，而不是未定义行为。增量 `transcode`、EOF 处理以及带
hooks 的 engine 见[用户手册](doc/user_guide.zh_CN.md)。

## 能力与边界

- 底层 value/unit 契约：`Codec`、`DecodeFailure`，以及供 checked adapter 使用的显式
  encode/decode 上界。
- 自有完整值 facade：`ValueEncoder`、`ValueDecoder`、`CodecValueEncoder`、
  `CodecValueDecoder`；声明 reset/finish value 时支持 lifecycle-aware decode。
- 严格的调用方缓冲区转换：`Transcoder`、`CodecTranscodeEncoder`、
  `CodecTranscodeDecoder`、`CodecTranscodeConverter`，以 `TranscodeProgress` 与
  `TranscodeStatus` 表达背压。
- 带策略的 engine：`engine::TranscodeEncodeEngine`、`engine::TranscodeDecodeEngine`、
  `engine::TranscodeConvertEngine`，以及确认 EOF 后对不完整尾部选择 `Reject` /
  `Skip` / `Emit` 的 hooks（如 `DecodeIncompleteAction`）。
- 字节序辅助：`ByteOrder`、`ByteOrderSpec`、`BigEndian`、`LittleEndian`、
  `NativeEndian`。
- 启用 `registry` 时：经校验的稳定 ID、`ValueStringCodecDescriptor`、
  `ValueBytesCodecDescriptor`、`ValueStringCodecRegistry`、`ValueBytesCodecRegistry`，
  以及 `register_value_string_codec!` / `register_value_bytes_codec!`；只擦除注册时已固定
  value 类型的双向 whole-value codec（字符串或 `Vec<u8>` wire），执行前核对实际输入类型。
- 启用 `io` 时：基于 `qubit-io` 的同步与部分 I/O 异步 transcode 输入/输出 bridge。

本库不实现具体领域格式或字符集表。`Codec::decode` 通过 `DecodeFailure` 区分可见输入
不完整与领域内非法；不完整前缀本身不构成 EOF 决策。相同 codec 状态下，
`Codec::encode_len` 必须与随后成功的 `Codec::encode` 一致，包括有意的零输出缓冲。
`Transcoder` 遵循 `reset -> transcode/transcode_eof -> finish`：`NeedInput` 将尾部留给
调用方；`NeedOutput` 要求扩大目标容量；`Complete` 表示本次调用起点之后的可见输入均已
消费。容量上界须覆盖所有可达瞬态；自有 adapter 可能分配内存，流式 API 使用调用方缓冲
区。

## 延伸阅读

- [用户手册](doc/user_guide.zh_CN.md)
- [API 文档](https://docs.rs/qubit-codec)
- [生命周期感知 decode 示例](examples/decode_lifecycle.rs)
- [English README](README.md) · [English user guide](doc/user_guide.md)

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
./.infra/bin/ci-check.sh

# 检查代码覆盖率
./.infra/bin/coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./.infra/bin/align-ci.sh` 格式化代码，运行 `./.infra/bin/ci-check.sh` 对齐 CI 要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-codec](https://github.com/qubit-ltd/rs-codec)
