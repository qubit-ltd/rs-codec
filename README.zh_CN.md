# Qubit Codec

[![Rust CI](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-codec/coverage-badge.json)](https://qubit-ltd.github.io/rs-codec/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-codec.svg?color=blue)](https://crates.io/crates/qubit-codec)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-codec` 为 Rust codec 与 adapter 作者提供一套统一契约，覆盖单值 codec、
自有输出便捷 API 和调用方管理缓冲区的流式转换。格式 crate 只需保留自己的二进制、
文本或字符集规则，即可复用本库经过检查的容量管理、显式生命周期和支持策略扩展的
转换循环。

## 安装

```toml
[dependencies]
qubit-codec = "0.12"
```

默认 feature 集为空。只有使用 `qubit-io` 缓冲 bridge 时才启用 `io`：

```toml
[dependencies]
qubit-codec = { version = "0.12", features = ["io"] }
```

最低支持的 Rust 版本为 1.94。

## 快速开始

先根据格式特点选择最小够用的公开契约：

| 格式需求 | 起点 |
| --- | --- |
| 一个有意义的逻辑 value 对应若干编码 unit | 实现 `Codec` |
| 只有完整输入才构成有意义的操作 | 直接实现 `ValueEncoder` / `ValueDecoder` |
| 已有 `Codec`，需要自有输出 | 使用 `CodecValueEncoder` / `CodecValueDecoder` |
| 已有 `Codec`，需要严格的缓冲流 | 使用 `CodecTranscode*` adapter |
| 非法或不可编码 value 需要策略 | 使用 transcode engine 与 hooks |
| EOF 或 framing 行为不适合现有 engine | 实现 `Transcoder` |

例如，一个提供定宽大端 `u16` codec 的 crate 只需实现一次 `Codec`，随后便能提供
自有输出操作，无需重复容量和生命周期处理：

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

[用户手册](doc/user_guide.zh_CN.md)沿用这一场景，继续讲解自有输出 decode、调用方
缓冲区流式转换、不完整输入、生命周期输出与策略 hooks。

## 为什么需要这个项目

一个格式 crate 通常从 value-to-unit 操作开始，随后逐步增加自有输出 helper、流式
adapter、畸形输入策略和 I/O 集成。若每层各自实现下标、容量、EOF 以及 reset/finish
行为，同一格式很容易产生互不兼容的契约。`qubit-codec` 统一这些机制，同时让领域规则
留在拥有它们的格式 crate 中。

## 核心能力

| 需求 | 公开 API |
| --- | --- |
| 底层 value/unit 契约 | `Codec`、`DecodeFailure` |
| 自有完整值转换 | `ValueEncoder`、`ValueDecoder`、`CodecValueEncoder`、`CodecValueDecoder` |
| 严格的调用方缓冲区转换 | `Transcoder`、`CodecTranscodeEncoder`、`CodecTranscodeDecoder`、`CodecTranscodeConverter` |
| 带策略的转换 | `engine::TranscodeEncodeEngine`、`engine::TranscodeDecodeEngine`、`engine::TranscodeConvertEngine` 与 hooks（包括 `DecodeIncompleteAction`）|
| 进度与背压 | `TranscodeProgress`、`TranscodeStatus` |
| 运行时或静态字节序 | `ByteOrder`、`ByteOrderSpec`、`BigEndian`、`LittleEndian`、`NativeEndian` |
| 启用 `io` 后的 `qubit-io` bridge | 同步及部分 I/O 异步 transcode input/output adapter |

## 边界与保证

- 本库不实现具体二进制格式、字符集、Base64、hex、percent encoding 或
  `std::io` extension。
- `Codec::decode` 通过 `DecodeFailure` 区分可见输入不完整与领域输入非法；不完整
  前缀本身不是 EOF 决策。
- 在相同状态下，`Codec::encode_len` 必须等于随后成功的 `Codec::encode` 写入量，
  包括有意的零输出缓冲。
- `Transcoder` 遵循 `reset -> transcode/transcode_eof -> finish`。`NeedInput`
  将尾部留给调用方；`NeedOutput` 要求更多目标容量；`Complete` 表示所有可见输入均
  已消费。
- `TranscodeDecodeHooks::handle_incomplete_decode` 仅在确认 EOF 后为不完整尾部选择
  `Reject`、`Skip` 或 `Emit`。
- 容量上界必须覆盖所有可达瞬态。自有输出 adapter 可能分配内存；流式 API 使用
  调用方提供的缓冲区。

## 延伸阅读

- [用户手册](doc/user_guide.zh_CN.md)：实现 codec、暴露 adapter、驱动流并诊断失败。
- [English user guide](doc/user_guide.md)
- [API 文档](https://docs.rs/qubit-codec)
- [生命周期感知 decode 示例](examples/decode_lifecycle.rs)
- [English README](README.md)

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
./ci-check.sh

# 检查代码覆盖率
./coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./align-ci.sh`格式化代码，运行`./ci-check.sh`对齐CI要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-codec](https://github.com/qubit-ltd/rs-codec)
