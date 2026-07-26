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
text、misc 和 I/O adapter crate 需要共享的小型 trait 与值类型，不放具体格式实现。

本库提供：

- 用于底层单值缓冲区编码解码的 `Codec` trait。
- 基于给定 `Codec` 显式适配 value 与 buffered 转换的
  `CodecValueEncoder`、`CodecValueDecoder`、`CodecTranscodeEncoder`、
  `CodecTranscodeDecoder` 和
  `CodecTranscodeConverter` adapter。
- 用于下游带策略 encoder 复用公共 buffered encode 循环的
  `TranscodeEncodeEngine`、`TranscodeEncodeHooks` 和 `EncodeContext`。
- 用于下游带策略 decoder 复用公共 buffered decode 循环的
  `TranscodeDecodeEngine`、`TranscodeDecodeHooks`、`DecodeInvalidAction` 和
  `DecodeContext`。
- 用于组合 decode side 与 encode side 的带策略 unit-to-unit 转换管线的
  `TranscodeConvertEngine` 和 `TranscodeConvertError`。
- 用于完整值便捷转换的 `ValueEncoder` 和 `ValueDecoder` trait。
- 用于调用方管理逻辑流缓冲区转换的 `Transcoder`、`TranscodeProgress`
  和 `TranscodeStatus`。
- 用于表达 transcoder 语义方向的 `TranscodeEncoder`、`TranscodeDecoder` 和
  `TranscodeConverter` marker trait。
- 供 binary 与 text codec 共享的 `ByteOrder`、`ByteOrderSpec`、
  `BigEndian`、`LittleEndian` 和 `NativeEndian`。
- 用于安全构造 `NonZeroUsize` 的 `nz`、`nz_const` 和 `nz!`。

具体 codec 位于相邻 crate，例如 `qubit-codec-binary`、
`qubit-codec-text` 和 `qubit-codec-misc`。

## 设计目标

- **分层边界清晰**：把领域无关 trait 与 binary、text、misc、stream 实现分开。
- **公开 API 小而稳定**：只暴露多个 codec crate 都需要共享的基础原语。
- **策略中立**：charset、畸形输入和线格式规则由领域 crate 自己定义。
- **零成本标记**：用可复制的类型和值标记表达字节序，不产生运行时分配。
- **稳定进度报告**：用 `TranscodeProgress` 和 `TranscodeStatus` 明确表达调用方管理缓冲区时的转换进度。

## 特性

### Cargo 特性

默认特性集为空。启用 `io` 特性可使用 `qubit-io` bridge，其中包括
`TranscodeDecodeInput` 和 `TranscodeEncodeOutput`。

### 核心转换 Trait

- **`Codec`**：在调用方管理的 unit 缓冲区中编码和解码一个值或 codec quantum。
- **`DecodeFailure`**：区分 `Codec::decode` 返回的 incomplete-prefix 流程控制
  与 codec-domain invalid input。
- **`TranscodeFailure`**：表达非法下标、输出不足、长度溢出、不完整输入和
  trailing input 等 framework failure。
- **`TranscodeDomainError<E>`**：为 codec 与策略错误附加 reset、main 或
  finish 阶段上下文。
- **`TranscodeEncodeError<E, V>` / `TranscodeDecodeError<E>` /
  `TranscodeConvertError<DE, EE, V>`**：组合 framework 与 domain failure 的
  方向性公开错误。
- **`ValueEncoder<Input>`**：把借用输入编码为自有输出。
- **`ValueDecoder<Input>`**：把借用的编码输入解码为自有输出。
- **`CodecValueEncoder<C>`**：把 `Codec` 包装为
  返回自有 `Vec<C::Unit>` 的 `ValueEncoder<C::Value>`。
- **`CodecValueDecoder<C>`**：把 `Codec` 包装为严格
  `ValueDecoder<[C::Unit]>`；decode reset 和 finish 不产生值时接收恰好一个
  完整编码值，lifecycle-aware 方法则分别保留 reset、主体和 finish 输出。
- **`DecodeLifecycleOutput<Value>` / `DecodeLifecycleProgress<Value>`**：
  分别表示完整 decode 生命周期的自有结果与调用方 scratch 结果。

### 缓冲区转换原语

- **`Transcoder`**：在调用方提供的缓冲区中把关联类型 `Input` 转换为关联类型 `Output`，并在调用方处理完不完整输入尾部后完成内部收尾输出。
- **`TranscodeEncoder`**：表示 value-to-unit 缓冲区编码的语义化
  `Transcoder` bound。
- **`TranscodeDecoder`**：表示 unit-to-value 缓冲区解码的语义化
  `Transcoder` bound。
- **`TranscodeConverter`**：表示 unit-to-unit 缓冲区转换的语义化
  `Transcoder` bound。
- **`CodecTranscodeEncoder<C>`**：把 `Codec` 包装为在调用方输出缓冲区上工作的
  `TranscodeEncoder`。
- **`TranscodeEncodeEngine<C, H>`**：持有 codec 与策略 hooks，并运行公共
  buffered encode 循环的可复用 engine。
- **`TranscodeEncodeHooks<C>`**：供带策略 codec-backed encoder
  共享公共循环时实现的 transcode/finalization 策略 hook trait。
- **`EncodeContext<'a, Value, Unit>`**：传给 encode hook 的输入值、输入索引、
  输出切片和游标上下文。
- **`CodecTranscodeDecoder<C>`**：把 `Codec` 包装为无策略的
  严格 `TranscodeDecoder`；engine 自己检测到的不完整尾部保留在调用方输入缓冲区中，
  codec 返回的 decode error 会被直接包装返回。
- **`TranscodeDecodeEngine<C, H>`**：持有 codec 与策略 hooks，并运行公共
  decode 循环的可复用 engine。
- **`TranscodeDecodeHooks<C>`**：供带策略 codec-backed decoder
  共享公共 decode 循环时实现的策略 hook trait。
- **`DecodeInvalidAction<Value>`**：decoder engine hook 针对非法输入返回的策略动作。
- **`CodecTranscodeConverter<D, E>`**：组合一个解码
  codec 和一个编码 codec，形成无策略的 `TranscodeConverter`。
- **`TranscodeConvertEngine<D, E, DH, EH>`**：组合 decode hooks、
  encode hooks 与公共 buffered conversion 循环的可复用 unit-to-unit
  converter engine。
- **`TranscodeDecodeInput<I>`**（需要 `io` 特性）：持有底层 unit
  `BufferedInput`；既支持严格及 lifecycle-aware 的单值解码，也可以通过
  `transcode_into` / `finish_transcode_into` 驱动调用方传入的 streaming decoder。
- **`TranscodeEncodeOutput<O>`**（需要 `io` 特性）：持有底层 unit
  `BufferedOutput`；普通 `flush` 只排空 unit buffer。状态化 streaming
  encoder 使用 `transcode_from` 和 `finish`。
- **`TranscodeProgress`**：报告相对读取和写入的单元数量。
- **`TranscodeStatus`**：区分转换完成、需要更多输入和需要更多输出空间。
- **`TranscodeFailure` / `CapacityError` / `TranscodeContractError`**：把
  framework 层的缓冲区、容量规划和错误进度契约失败，与 codec 或策略
  domain error 分开表达。

### 字节序标记

- **`ByteOrder`**：公共 API 中使用的运行时字节序枚举。
- **`ByteOrderSpec`**：热路径 codec 使用的类型级字节序 trait。
- **`BigEndian` / `LittleEndian`**：零大小字节序标记类型。

### 聚焦的公开 API

- **不包含具体格式**：binary、text 和 misc codec 发布在相邻 crate 中。

## 如何选择合适的抽象层

`qubit-codec` 提供了多个层次，因为真实的 codec 栈需求差异很大。按下面的决策
树挑选最小够用的那一层。

```text
你正在写什么？

├── 一个"单值"级别的 codec（一个 UTF-8 字符、一个 LEB128 整数、
│   一个 Base64 quantum、一个定宽标量……）
│       → 实现 Codec
│         （unchecked 的单值契约；所有上层都建立在它之上）
│
├── 一个"整串"型 codec，"单值"对它没有合理含义
│   （Base64 padding、带分隔符的 hex、percent 编码、C 字符串字面量……）
│       → 直接实现 ValueEncoder<Input> / ValueDecoder<Input>
│         （跳过 Codec；这两个 trait 同时也充当便利层）
│
├── 在已有 Codec 之上做严格透传的流式包装（codec 报什么错就原样返回）
│       → 用 CodecTranscodeDecoder<C> / CodecTranscodeEncoder<C>
│         / CodecTranscodeConverter<D, E>
│         （零代码；现成的 Transcoder 实现）
│
├── 在 Codec 之上做"拥有所有权"的便利包装（一次调用 → 一个 Vec<Unit>
│   或一个 Value）
│       → 用 CodecValueEncoder<C> / CodecValueDecoder<C>
│         （每次调用分配；便利层的 ValueEncoder/Decoder）
│
└── 需要对非法输入做策略决策的流式 codec：
    跳过、替换、计数、报错——不只是原样透传
        → 实现 TranscodeDecodeHooks<C> / TranscodeEncodeHooks<C>，
          再包装成 TranscodeDecodeEngine<C, H> / TranscodeEncodeEngine<C, H>
          （只需写策略；engine 负责缓冲循环、游标、NeedInput/NeedOutput
           报告、容量检查）

unit-to-unit 转换（如 UTF-8 字节 → UTF-16 字节）的写法是组合一个解码 codec
和一个编码 codec：
- 严格管线   → CodecTranscodeConverter<D, E>
- 带策略钩子 → TranscodeConvertEngine<D, E, DH, EH>
```

### 层次总览

```text
┌────────────────────────────────────────────────────────────────┐
│  qubit-io-binary / qubit-io-text             （具体 I/O）       │
├────────────────────────────────────────────────────────────────┤
│  TranscodeDecodeInput / TranscodeEncodeOutput（I/O bridge；需要 io）│
├────────────────────────────────────────────────────────────────┤
│  TranscodeXxxEngine + TranscodeXxxHooks       （策略 + 循环）   │
│  CodecTranscodeDecoder / Encoder / Converter  （严格 bridge）   │
├────────────────────────────────────────────────────────────────┤
│  Transcoder + TranscodeProgress + TranscodeStatus               │
│  ValueEncoder<Input> / ValueDecoder<Input>      （便利层）      │
├────────────────────────────────────────────────────────────────┤
│  Codec                              （单值、unchecked）         │
└────────────────────────────────────────────────────────────────┘
```

选择更上层并不意味着要重写下层：`CodecValueEncoder<C>` 与
`CodecTranscodeDecoder<C>` 之类的现成适配器能直接把任意 `Codec` 升级到
上层 trait。**只有当你真的需要对非法输入、替换输出、stateful finish 输出
做策略决策时，才下沉到 engine + hooks 这一层。**

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
qubit-codec = "0.11"
```

## 用户手册

[用户手册](doc/user_guide.zh_CN.md)说明抽象选择和正式的 `Codec`、`Transcoder`
实现者契约，包括进度、生命周期、EOF 和错误恢复规则。另有
[英文版](doc/user_guide.md)。

## 快速开始

```rust
use qubit_codec::{
    TranscodeProgress,
    TranscodeStatus,
    ValueEncoder,
};

struct StringEncoder;

impl ValueEncoder<str> for StringEncoder {
    type Output = String;
    type Error = core::convert::Infallible;

    fn encode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

let mut encoder = StringEncoder;
let encoded = ValueEncoder::<str>::encode(&mut encoder, "codec")?;
assert_eq!("codec", encoded);

let progress = TranscodeProgress::complete(3, 4);
assert_eq!(TranscodeStatus::Complete, progress.status());

# Ok::<(), core::convert::Infallible>(())
```

## API 参考

### 核心 Codec Trait

| Trait | 用途 | 典型实现者 |
|-------|------|------------|
| `Codec` | 在调用方缓冲区中编码/解码一个值或 quantum | 二进制标量、字符集字符、转义 byte、Base64 quantum |
| `ValueEncoder<Input>` | 把借用输入编码为自有输出 | 文本、二进制或 misc 便捷 helper |
| `ValueDecoder<Input>` | 把借用输入解码为自有输出 | 文本、二进制或 misc 便捷 helper |
| `TranscodeEncoder` | 把逻辑值编码进调用方提供的 unit 缓冲区 | Charset 或 binary buffered encoder |
| `TranscodeDecoder` | 把编码 unit 解码进调用方提供的 value 缓冲区 | Charset 或 binary buffered decoder |
| `TranscodeConverter` | 在两种编码 unit 表示之间转换 | Charset 或 binary buffered converter |

| 类型 | 用途 |
|------|------|
| `DecodeFailure<E>` | 底层 decode 结果，表示可见输入是 incomplete prefix，或 codec-domain invalid input |
| `TranscodeFailure` | framework failure，表示非法下标、输出不足、输出长度溢出、不完整输入或 trailing input |
| `TranscodeDomainError<E>` | 带 reset、main 或 finish 阶段上下文的 codec 或策略错误 |
| `TranscodeEncodeError<E, V>` | 组合 framework、不可编码值与 domain failure 的 encode-side 错误 |
| `TranscodeDecodeError<E>` | 组合 framework 与 domain failure 的 decode-side 错误 |
| `TranscodeConvertError<DE, EE, V>` | 保留 decode side、encode side、不可编码值与 framework failure 的 converter 错误 |
| `CapacityError` | 在分配或写入输出前返回的容量规划错误 |
| `TranscodeContractError` | 自定义 `Transcoder` 返回不一致进度时报告的错误 |

### Codec Adapter

| 类型 | 用途 |
|------|------|
| `CodecValueEncoder<C>` | 通过 `C: Codec` 把一个借用 `C::Value` 编码成自有 `Vec<C::Unit>`，不要求 `C::Value: Clone` |
| `CodecValueDecoder<C>` | 严格解码恰好一个借用 `[C::Unit]` slice，或通过 lifecycle-aware 方法保留 reset、主体和 finish 输出 |
| `DecodeLifecycleOutput<Value>` | 持有一次完整 decode 生命周期的 reset 输出、主体值和 finish 输出 |
| `DecodeLifecycleProgress<Value>` | 返回主体值，以及分别写入两块调用方缓冲区的 reset/finish 长度 |
| `CodecTranscodeEncoder<C>` | 通过 `C: Codec` 把 `C::Value` slice 编码进调用方提供的 `C::Unit` 缓冲区 |
| `CodecTranscodeDecoder<C>` | 通过 `C: Codec` 严格地把 `C::Unit` slice 解码进调用方提供的 `C::Value` 缓冲区 |
| `CodecTranscodeConverter<D, E>` | 先解码 `D::Unit` source unit，再用满足 `E::Value = D::Value` 的 `E` 编码 `E::Unit` target unit |

### I/O Adapter

| 类型 | 用途 |
|------|------|
| `TranscodeDecodeInput<I>`（需要 `io`） | 持有 `qubit_io::Input`，可严格解码单值、保留一次完整生命周期，或驱动 caller-owned streaming decoder |
| `TranscodeEncodeOutput<O>`（需要 `io`） | 持有 `qubit_io::Output`；普通 `flush` 排空缓冲单元；状态化 streaming encoder 使用 `transcode_from` 和 `finish` |

### Encoder Hooks 和 Engine

| 类型 | 用途 |
|------|------|
| `TranscodeEncodeEngine<C, H>` | 基于低层 `Codec` 与策略 hooks 的可复用 buffered encoder engine |
| `TranscodeEncodeHooks<C>` | 编码单个 value、reset 前清理并完成 encoded output 收尾的 hook 契约 |
| `TranscodeEncodeError<E, V>` / `TranscodeEncodeErrorOf<C>` | encode-side 错误及其 codec 派生别名 |
| `EncodeContext<'a, Value, Unit>` | 传递给 encode hook 的输入值、输入索引、输出切片和游标 |

### Decoder Hooks 和 Engine

| 类型 | 用途 |
|------|------|
| `TranscodeDecodeEngine<C, H>` | 基于低层 `Codec` 与策略 hooks 的可复用 buffered decoder engine |
| `TranscodeDecodeHooks<C>` | invalid-input decode 策略 hook 契约 |
| `TranscodeDecodeError<E>` / `TranscodeDecodeErrorOf<C>` | decode-side 错误及其 codec 派生别名 |
| `DecodeContext` | 传递给 decode policy hook 的上下文 |
| `DecodeInvalidAction<Value>` | 非法输入策略动作：跳过输入或输出替换值 |

### Converter Engine

| 类型 | 用途 |
|------|------|
| `TranscodeConvertEngine<D, E, DH, EH>` | 可复用 unit-to-unit converter，用 `D` 解码、用 `E` 编码，并应用 decode/encode hooks |
| `TranscodeConvertError<DE, EE, V>` / `TranscodeConvertErrorOf<D, E>` | converter 错误及其 codec 派生别名 |

### `Transcoder` 操作

| 方法 | 描述 |
|------|------|
| `max_transcode_output_len(input_len)` | 返回对所有可达瞬态都成立的 streaming 阶段输出长度上界 |
| `max_total_output_len(input_len)` | 返回完整 `reset -> transcode -> finish` 流程的输出长度上界 |
| `max_reset_output_len()` | 返回对所有可达瞬态都成立的 reset 输出长度上界 |
| `max_finish_output_len()` | 返回对所有可达瞬态都成立的 finish 收尾输出长度上界 |
| `reset()` | 保留配置并重置逻辑流状态 |
| `transcode(input, input_index, output, output_index)` | 把输入单元转换为输出单元 |
| `transcode_complete_into(input, output)` | 从传入 slice 起点运行一次完整 `reset -> transcode -> finish` 流程 |
| `finish(output, output_index)` | 完成内部收尾输出，例如 digest 或 trailer |

### `TranscodeStatus` 取值

| 状态 | 含义 |
|------|------|
| `Complete` | 当前转换步骤已完成 |
| `NeedInput` | 需要更多输入单元；不完整尾部仍留在调用方输入缓冲区中 |
| `NeedOutput` | 需要更多输出空间 |

### 契约说明

- `Codec::MIN_UNITS_PER_VALUE` 是调用 `Codec::decode` 的安全下界；
  `Codec::MAX_UNITS_PER_VALUE` 是单值输出/读取上界。checked adapter 在使用前会断言
  `min <= max`。
- `Codec::decode` 用 `DecodeFailure::Incomplete` 表示当前可见输入是合法前缀但还需要更多
  unit，用 `DecodeFailure::Invalid` 表示 codec-domain 的畸形、非规范或其他非法输入。
- `encode_len(value)` 必须等于同一 value 与 codec 状态下 `Codec::encode`
  实际写入的 unit 数量，并且不能超过 `Codec::MAX_UNITS_PER_VALUE`。
- 状态化自有单值编码使用 `CodecValueEncoder<C>`。decode 生命周期会产生输出的
  codec 应使用 `CodecValueDecoder::decode_lifecycle`；需要复用调用方缓冲区时使用
  `decode_lifecycle_with_scratch`。连续流应使用 streaming transcoder adapter。
- 方向性 adapter 暴露 `TranscodeEncodeError`、`TranscodeDecodeError` 或
  `TranscodeConvertError`；它们把 framework failure 与带阶段上下文的 codec、
  policy domain error 分开表达。
- `NeedInput` 表示被报告的不完整尾部未被消费，调用方重试时必须保留这段输入。
  它是 streaming 边界信号，不是 EOF 错误；`finish` 不会接收这段 source tail。
  调用方必须在 finalization 前自己应用 EOF 策略。
- 默认 codec-backed decoder 和 converter 适用于“值边界可由可见前缀加 codec 状态局部决定”的格式。
  如果格式需要 EOF-aware maximal-munch 解析、延迟边界决策，或在 EOF 时重新解释 pending prefix，
  应使用自定义 `Transcoder` 或 value-level facade 承载该策略。
- `NeedOutput` 表示输出切片到达容量边界，因此输入没有被完全消费。

### 字节序类型

| 类型 | 使用场景 |
|------|----------|
| `ByteOrder` | 公共 API 中运行时选择字节序 |
| `ByteOrderSpec` | 类型级字节序抽象 |
| `nz`、`nz!` | 安全构造 `NonZeroUsize` |
| `BigEndian` | 大端类型标记 |
| `LittleEndian` | 小端类型标记 |
| `NativeEndian` | 本机字节序类型标记 |

## 库边界

`qubit-codec` 不包含具体 binary 格式、字符集或 percent/Base64/hex codec。
启用 `io` 特性后，它面向 I/O 的公开面只保留供下游 stream crate 复用的低层
`qubit_io::Input` / `qubit_io::Output` bridge 类型。`std::io::Read` /
`std::io::Write` extension trait 和具体 reader/writer adapter 应放在领域
crate 中，让下游只依赖自己需要的层。

## 性能考虑

流式 trait 与 engine 主循环使用调用方提供的缓冲区。`BigEndian` 和
`LittleEndian` 是零大小类型，`ByteOrder` 是小型可复制枚举。
`CodecValueEncoder` 等 owned value adapter 会为 `Vec<Unit>` 结果分配内存；
codec lifecycle 输出或超过基础缓冲容量的 I/O 解码窗口也可能使用临时 scratch
存储。下游具体 codec 仍可能有额外分配行为。

## 依赖项

运行时依赖保持很少：

- `thiserror` 提供公共错误类型实现。
- 启用 `io` 特性后，`qubit-io` 提供 `TranscodeDecodeInput` 和
  `TranscodeEncodeOutput` 使用的 `BufferedInput` 与 `BufferedOutput`。

## 相关项目

- [qubit-codec-binary](https://github.com/qubit-ltd/rs-codec-binary)：二进制缓冲区级 codec。
- [qubit-codec-text](https://github.com/qubit-ltd/rs-codec-text)：charset 与 Unicode 缓冲区级 codec。
- [qubit-codec-misc](https://github.com/qubit-ltd/rs-codec-misc)：可复用的杂项字节与文本 codec。
- [qubit-io](https://github.com/qubit-ltd/rs-io)：通用 `std::io` helper。
- Qubit 旗下的更多 Rust 库发布在 GitHub 组织
  [qubit-ltd](https://github.com/qubit-ltd)。

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
