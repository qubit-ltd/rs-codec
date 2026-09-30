# Qubit Codec 用户手册

[中文 README](../README.zh_CN.md) · [English user guide](user_guide.md) · [API 文档](https://docs.rs/qubit-codec)

本文适用于 `qubit-codec` 0.16.0，要求 Rust 1.94 或更高版本。它面向需要在本地保留格式规则、同时复用容量规划、生命周期和流式转换机制的 codec 与 adapter crate 作者。若你只想直接使用现成的 hex、Base64 或字符集实现，应选用对应的领域 crate，而不是本库。读到[检查转换结果](#检查转换结果)，就能发布一个带自有输出与调用方缓冲区 adapter 的定宽 `Codec`；后面章节供你按需查阅增量流、生命周期 decode、策略 hooks、可选 I/O bridge 以及 `registry` feature。

## 目录

- [它解决什么问题](#它解决什么问题)
- [从哪里开始](#从哪里开始)
- [发布一个定宽 codec](#发布一个定宽-codec)
  - [只实现一次表示规则](#只实现一次表示规则)
  - [声明 Codec 上界](#声明-codec-上界)
  - [提供自有单值操作](#提供自有单值操作)
  - [把多个 value 编码到调用方缓冲区](#把多个-value-编码到调用方缓冲区)
  - [涉及的核心类型](#涉及的核心类型)
- [检查转换结果](#检查转换结果)
  - [成功时能看到什么](#成功时能看到什么)
  - [不完整输入与非法输入](#不完整输入与非法输入)
- [驱动增量流](#驱动增量流)
- [处理生命周期 decode 输出](#处理生命周期-decode-输出)
- [为畸形输入或 EOF 配置策略](#为畸形输入或-eof-配置策略)
- [选择字节序辅助类型](#选择字节序辅助类型)
- [接入缓冲 I/O](#接入缓冲-io)
- [在链接期注册 codec 供运行时查找](#在链接期注册-codec-供运行时查找)
- [错误、诊断与排障](#错误诊断与排障)
- [边界与实践清单](#边界与实践清单)
- [延伸阅读](#延伸阅读)

## 它解决什么问题

以一个二进制协议 crate 为例。它要把 `u16` 字段编码成两个大端字节供报文使用，还要对外提供简单的 `encode(&u16) -> Vec<u8>` 供测试和一次性调用；之后还要从 socket 按块读入字节，不能把“只收到一个字节”误判成数据损坏。如果每一层各自实现下标运算、容量规划、reset/finish 生命周期，以及“还需要更多字节”和“字节本身不合法”的区分，同一格式很容易积累互不兼容的契约。

使用 `qubit-codec` 时，格式 crate 在 `Codec` 上实现表示规则一次；checked adapter 提供自有输出（`CodecValueEncoder` / `CodecValueDecoder`）、严格的调用方缓冲区转换（`CodecTranscode*`），以及可选的带 hooks 的策略 engine。本库**不**提供 hex、percent encoding、UTF-8 等具体格式，这些仍由领域 crate 负责。本库也**不**替调用方决定 EOF：在调用方确认输入结束或采用文档化的 EOF 策略之前，合法但不完整的 prefix 只会被报告为不完整，而不是非法。

## 从哪里开始

1. [发布一个定宽 codec](#发布一个定宽-codec) 介绍 `U16BeCodec`、自有 encode/decode，以及批量写入调用方缓冲区。
2. [检查转换结果](#检查转换结果) 说明成功时的可观察结果，以及不完整输入与非法输入的区别。基础接入到这里结束。
3. 按需要查阅[驱动增量流](#驱动增量流)、[处理生命周期 decode 输出](#处理生命周期-decode-输出)、[为畸形输入或 EOF 配置策略](#为畸形输入或-eof-配置策略)、[接入缓冲 I/O](#接入缓冲-io)或[在链接期注册 codec 供运行时查找](#在链接期注册-codec-供运行时查找)。

可选 feature `io` 与 `registry` 相互独立，只在 crate 实际使用时启用。

## 发布一个定宽 codec

本场景的成功标准如下：

1. `0x1234` 编码为 `[0x12, 0x34]`；
2. `[0x12, 0x34]` 解码为 `0x1234`；
3. 只有一个字节的输入会在进入 unsafe `Codec::decode` 之前被判定为不完整；
4. 同一 codec 可以把多个 value 编码进一块调用方拥有的缓冲区，而无需重复实现容量计算。

### 只实现一次表示规则

```rust
use core::{convert::Infallible, num::NonZeroUsize};
use qubit_codec::{Codec, DecodeFailure};

#[derive(Clone, Copy, Debug, Default)]
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
```

定宽整数或码元通常有合理的单值 `Codec` 边界。格式化 hex 字符串、percent-encoded 文本或 C 字符串字面量往往没有；这类格式直接实现 `ValueEncoder` / `ValueDecoder` 通常更清晰。

### 声明 Codec 上界

三个必需常量是公开的安全与容量契约，不是性能提示：

| 契约 | 含义 |
| --- | --- |
| `MIN_UNITS_PER_VALUE` | 可能容纳一个 decoded value 的最小可读输入，必须非零。 |
| `MAX_DECODE_UNITS_PER_VALUE` | 成功 decode 的最大消费量或不完整重试需求，必须非零且不小于最小值。 |
| `MAX_ENCODE_UNITS_PER_VALUE` | 主 encode 阶段与 value 无关的输出上界；只有刻意缓冲时才允许为零。 |

默认 `encode_len` 返回 `MAX_ENCODE_UNITS_PER_VALUE`，对本场景的定宽编码是精确值。变宽或有状态 codec 必须覆盖该方法。在相同 value 与 codec 状态下，成功的 `encode` 必须恰好写入 `encode_len` 报告的长度。当 `Value` 包含编码域之外的值时，应覆盖 `can_encode_value`；checked encoder 会在 `encode_len` 和 unsafe `encode` 之前调用它。

checked adapter 会在调用 unsafe `Codec` 方法前建立下标与容量前置条件。实现仍须只在前置条件允许范围内读写；成功 decode 时返回非零消费量且不超过 decode 上界；出错时保持状态一致；并稳定区分 `DecodeFailure::Incomplete` 与 `DecodeFailure::Invalid`。应像上面一样在入口处用 `debug_assert!` 标明假定的范围。

### 提供自有单值操作

`CodecValueEncoder` 与 `CodecValueDecoder` 执行完整 codec 生命周期并返回自有输出：

```rust
use qubit_codec::{CodecValueDecoder, CodecValueEncoder, ValueEncoder};

let mut encoder = CodecValueEncoder::new(U16BeCodec);
let encoded = encoder.encode(&0x1234).expect("encoding is infallible");
assert_eq!(vec![0x12, 0x34], encoded);

let mut decoder = CodecValueDecoder::new(U16BeCodec);
let decoded = decoder.decode(&encoded).expect("input contains one u16");
assert_eq!(0x1234, decoded);
```

严格单值 decode 只允许一个 main value；其后若还有额外 unit，会产生 `TranscodeFailure::TrailingInput`。

### 把多个 value 编码到调用方缓冲区

`CodecTranscodeEncoder` 对一组 value 复用同一 codec。one-shot helper 会计算容量并执行完整生命周期，缓冲区仍由调用方持有：

```rust
use qubit_codec::{CodecTranscodeEncoder, Transcoder};

let values = [0x1234, 0xabcd];
let mut encoder = CodecTranscodeEncoder::new(U16BeCodec);
let capacity = encoder
    .max_total_output_len(values.len())
    .expect("capacity arithmetic should not overflow");
let mut output = vec![0_u8; capacity];
let written = encoder
    .transcode_complete_into(&values, &mut output)
    .expect("encoding is infallible");
output.truncate(written);

assert_eq!(vec![0x12, 0x34, 0xab, 0xcd], output);
```

unit 到 value 使用 `CodecTranscodeDecoder`；严格的 decode 再 encode 管线使用 `CodecTranscodeConverter`。只有某一侧确实需要策略决策、而不只是返回领域错误时，才选用带 hooks 的 transcode engine。

### 涉及的核心类型

| 类型 | 作用 |
| --- | --- |
| `Codec` | 底层 value/unit 契约；带声明上界的 unsafe `encode` / `decode`。 |
| `DecodeFailure` | 在 `Codec::decode` 内区分可见输入不完整与领域输入非法。 |
| `CodecValueEncoder` / `CodecValueDecoder` | 基于单个 `Codec` 的自有完整值 facade。 |
| `CodecTranscodeEncoder` | 带 reset/finish 生命周期与进度报告的调用方缓冲区 encode 路径。 |
| `TranscodeFailure` | adapter 层的调用方误用、形状错误、分配限制或生命周期顺序问题。 |
| `Transcoder` | 由 `CodecTranscode*` adapter 与自定义流式 engine 实现的 trait。 |

概念上，格式自有规则落在 `Codec` 上；adapter 在不重复表示规则的前提下提供 checked 容量、生命周期与进度：

```text
格式自有规则
    |
  Codec --------------> CodecValueEncoder / CodecValueDecoder
    |                              自有输出
    |
    +-----------------> CodecTranscodeEncoder / Decoder / Converter
    |                              严格的缓冲区转换
    |
    +-----------------> Transcode*Engine + hooks
                                   带策略的转换

ValueEncoder / ValueDecoder        没有合理单值 codec quantum 的完整值格式

Transcoder                         自定义流式或 EOF/framing 行为
```

## 检查转换结果

自有 encode/decode 得到预期字节与整数，说明定宽场景在完整值层已接入。若还对外提供流式或批量 API，需要分别再验收。

### 成功时能看到什么

完成前面步骤后，测试或 REPL 中应能通过：

```rust
use qubit_codec::{CodecValueDecoder, CodecValueEncoder, ValueEncoder};

let mut encoder = CodecValueEncoder::new(U16BeCodec);
assert_eq!(encoder.encode(&0x1234).unwrap(), vec![0x12, 0x34]);

let mut decoder = CodecValueDecoder::new(U16BeCodec);
assert_eq!(decoder.decode(&[0x12, 0x34]).unwrap(), 0x1234);
```

这验证了表示规则、自有 adapter 与严格单值形状。它**不能**证明增量流、EOF 策略或生命周期 decode 行为；这些 API 对外公开时应单独加测试。

### 不完整输入与非法输入

checked decoder 会在调用 unsafe `decode` 之前拒绝只有一个字节的缓冲区：

```rust
use qubit_codec::{CodecValueDecoder, TranscodeFailure};

let mut decoder = CodecValueDecoder::new(U16BeCodec);
let error = decoder.decode(&[0x12]).expect_err("one byte is incomplete");
assert!(matches!(
    error.failure_ref(),
    Some(TranscodeFailure::IncompleteInput {
        input_index: 0,
        required: 2,
        available: 1,
    })
));
```

`Codec::decode` 内的 `DecodeFailure::Incomplete` 表示可见 prefix 在补充更多 unit 后仍可能合法；`DecodeFailure::Invalid` 表示领域拒绝该输入。adapter 会把这些区分映射为 `TranscodeFailure` 或领域错误；除非所有下游调用方对两种情况的处理确实相同，否则不要把不完整与非法压平为一种错误。

定宽 codec 的基础路径到这里结束。下面各节按需查阅：先是增量流生命周期，再是生命周期 decode 输出、策略 hooks、字节序辅助类型、I/O bridge 与注册表查找。

## 驱动增量流

显式 transcoder 生命周期如下：

```text
计算 reset 输出 -> reset
                    |
                    v
保留输入尾部 <- transcode/transcode_eof -> 排空或扩展输出
                    |
                    v
              计算 finish 输出 -> finish
```

`TranscodeProgress::read()` 与 `written()` 都相对于本次调用传入的下标。重试前必须推进两个游标。

| 状态 | 含义 | 调用方动作 |
| --- | --- | --- |
| `Complete` | 从 `input_index` 起的全部可见输入均已消费。 | 提供下一段，或在 EOF 时进入 finish。 |
| `NeedInput` | 不完整尾部未被消费。 | 保留尾部并补充输入；EOF 时采用格式明确规定的策略（`transcode_eof` 或 hooks）。 |
| `NeedOutput` | 转换在超过输出容量之前停止。 | 排空或扩展输出，并按报告的进度继续。 |

只有调用方确认不会再收到源 unit 后才调用 `transcode_eof`。默认实现会把仍存在的 `NeedInput` 转为 `TranscodeFailure::IncompleteInput`。`finish` 不接收源输入尾部，因此无法重新解释它。

内置 transcode engine 新建时尚未初始化。第一次成功操作必须是 `reset`；成功 `finish` 后，复用前必须再次 `reset`。生命周期操作失败可能使实例进入 poisoned 状态，直到下一次成功 `reset`。

## 处理生命周期 decode 输出

无状态 codec 的 reset/finish 上界为零。若 decoder 会从 `decode_reset` 或 `decode_finish` 输出 value，必须声明 `MAX_DECODE_RESET_VALUES` 或 `MAX_DECODE_FINISH_VALUES`。严格的 `CodecValueDecoder::decode` 会拒绝会产出生命周期 value 的输入；此时应改用 `decode_lifecycle` 或 `decode_lifecycle_with_scratch`。

可运行的[生命周期示例](../examples/decode_lifecycle.rs) 演示了一个在 decode 状态打开和关闭时输出 marker value 的单字节 codec。调用 `decode_lifecycle` 后，应分别检查 reset、main 与 finish 部分，而不是把它们合并成一个 main value。

## 为畸形输入或 EOF 配置策略

严格的 `CodecTranscode*` adapter 会直接返回领域失败。格式需要替换、跳过、计数或分阶段报告时，应保留共享循环并实现 hooks：

- `TranscodeEncodeEngine` 与 `TranscodeEncodeHooks` 处理不可编码 value 以及 encode reset/finish 策略；
- `TranscodeDecodeEngine` 与 `TranscodeDecodeHooks` 处理非法输入，以及 EOF 后的不完整输入；
- `TranscodeConvertEngine` 组合上述两组 hooks。

只有在输入源确认 EOF 后才调用 `transcode_eof`。对 `TranscodeDecodeEngine`，此时会调用 hook 的 `handle_incomplete_decode`。默认动作为 `Reject`，在 codec 提供不完整输入的领域错误时会保留该错误。需要恢复时可以返回：

```rust
use qubit_codec::engine::DecodeIncompleteAction;

let skip = DecodeIncompleteAction::<char>::Skip;
let replacement = DecodeIncompleteAction::Emit { value: '\u{fffd}' };
```

`Skip` 会消费整个剩余尾部但不产生 value。`Emit` 同样消费整个尾部并写入一个替代 value，因此调用方必须提供输出槽位。若 codec 已被调用并报告输入不完整，hook 收到 `Some(source)`；若尾部短于 `Codec::MIN_UNITS_PER_VALUE`，则收到 `None`。

当 framing、流状态或 EOF 行为无法用 `Codec` 加 hooks 表达时，应实现自定义 `Transcoder`。

## 选择字节序辅助类型

运行时配置使用 `ByteOrder`；静态选择有价值时使用 `ByteOrderSpec` 与 `BigEndian`、`LittleEndian` 或 `NativeEndian`。这些类型描述字节序策略，不实现具体整数 codec。前面的 `U16BeCodec` 场景把端序直接写在表示规则里。

## 接入缓冲 I/O

只有使用 `qubit-io` bridge 时才启用 `io` feature：

```toml
[dependencies]
qubit-io = "0.17"
qubit-codec = { version = "0.16", features = ["io"] }
```

`TranscodeDecodeInput` 与 `TranscodeEncodeOutput` 对接缓冲式 `qubit-io` trait。`TranscodeDecodeInput::transcode` 在 refill 确认 EOF 后，会把保留尾部交给 `transcode_eof`；需要逐步控制时使用 `TranscodeDecodeInput::transcode_eof_step`。异步对应物会报告 `AsyncTranscodeDecodeStep::EndOfInput`，随后在 `finish` 前调用 `AsyncTranscodeDecodeInput::transcode_eof_step`。

## 在链接期注册 codec 供运行时查找

当多个 crate 需要按稳定 ID 在链接期发现双向 value codec（字符串或字节 whole-value）时，启用 `registry` feature：

```toml
[dependencies]
qubit-codec = { version = "0.16", features = ["registry"] }
```

为固定 value 类型成对实现 `ValueEncoder` 与 `ValueDecoder` 后注册。字符串 wire 与字节 wire 使用不同的 codec 类型与注册宏，各自进入独立的进程级注册表；同一 `ValueCodecId` 可以同时出现在两张表中。

字符串 wire 示例：

```rust
use qubit_codec::{ValueDecoder, ValueEncoder, register_value_string_codec};

#[derive(Default)]
struct U32StringCodec;

impl ValueEncoder<u32> for U32StringCodec {
    type Output = String;
    type Error = std::io::Error;

    fn encode(&mut self, input: &u32) -> Result<Self::Output, Self::Error> {
        Ok(input.to_string())
    }
}

impl ValueDecoder<str> for U32StringCodec {
    type Output = u32;
    type Error = std::io::Error;

    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        input.parse().map_err(std::io::Error::other)
    }
}

register_value_string_codec!(id = "example.u32", codec = U32StringCodec, value = u32,);
```

字节 whole-value wire 示例（encode 产出 `Vec<u8>`，decode 输入 `&[u8]`）：

```rust
use qubit_codec::{ValueDecoder, ValueEncoder, register_value_bytes_codec};

#[derive(Default)]
struct U32BeBytesCodec;

impl ValueEncoder<u32> for U32BeBytesCodec {
    type Output = Vec<u8>;
    type Error = std::io::Error;

    fn encode(&mut self, input: &u32) -> Result<Self::Output, Self::Error> {
        Ok(input.to_be_bytes().to_vec())
    }
}

impl ValueDecoder<[u8]> for U32BeBytesCodec {
    type Output = u32;
    type Error = std::io::Error;

    fn decode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        let bytes: [u8; 4] = input.try_into().map_err(|_| std::io::Error::other("expected four bytes"))?;
        Ok(u32::from_be_bytes(bytes))
    }
}

register_value_bytes_codec!(id = "example.u32", codec = U32BeBytesCodec, value = u32,);
```

`ValueStringCodecRegistry::global()` 与 `ValueBytesCodecRegistry::global()`（需要错误值时用各自的 `try_global()`）在每个进程内分别收集一次已链接的注册项。查找使用经校验的 `ValueCodecId`；执行前会先核对擦除输入的实际类型。重复 ID 在同一注册表内属于注册错误，不会静默覆盖。该路径只擦除注册时 value 类型已固定的双向 whole-value codec，不能替代 `Codec` 流式 adapter。

## 错误、诊断与排障

| 错误 | 边界 | 恢复方式 |
| --- | --- | --- |
| `DecodeFailure::Incomplete` | 开放流 codec 输入是合法前缀，但长度不足。 | 保留尾部并重试，或显式做 EOF 决策。 |
| `DecodeFailure::Invalid` | unit 畸形、非规范或不可映射。 | 采用领域策略或返回 codec 错误。 |
| `TranscodeFailure` | 下标、容量、完整输入形状、分配或生命周期用法非法。 | 修正调用方状态并检查结构化 variant。 |
| `CapacityError` | 容量算术无法产生有效的 `usize` 上界。 | 在分配或写入前拒绝本次规划。 |
| `TranscodeDomainError<E>` | codec 或 hook 在 reset、main 或 finish 阶段失败。 | 报告时保留阶段和领域 source。 |
| `TranscodeContractError` | 自定义 transcoder 返回了不一致进度。 | 修复 transcoder；不是可恢复输入。 |
| `ValueCodecExecutionError` | 注册表分发：类型不匹配或 encode/decode 失败。 | 核对 value 类型与 ID，检查领域错误来源。 |

| 现象 | 检查顺序 |
| --- | --- |
| 文件末尾出现 `NeedInput` | 确认已保留尾部；调用 `transcode_eof`；在 `finish` 前采用格式专属 EOF 规则。 |
| `NeedOutput` 持续重复 | 推进两个进度计数；提供报告的容量；核对自定义上界。 |
| 自有 decode 拒绝看似有效的输入 | 检查 trailing unit 或 decode reset/finish 输出；必要时使用生命周期感知 decode。 |
| 转换前容量被拒绝 | 纳入 reset、main 与 finish 上界，并检查算术溢出。 |
| `TranscodeBeforeReset` 或 `TranscodeAfterFinish` | 在第一条流及复用前调用 `reset`。 |
| 替换逻辑正在演变成第二套循环 | 把决策移入 engine hooks。 |
| 注册表查找失败或执行类型不对 | 确认对应 `register_value_string_codec!` / `register_value_bytes_codec!` 已链接、在该注册表内 ID 唯一、输入类型与注册一致。 |

## 边界与实践清单

- 具体格式、字符集和高层 reader/writer adapter 应留在领域 crate，而不是放进 `qubit-codec`。
- `NeedInput` 是流式信号，不是最终 EOF；只有调用方或显式 EOF hook 才能决定输入结束时不完整尾部的含义。
- 容量方法必须覆盖所有可达瞬态，不能只看典型输出大小。
- 自有输出 adapter 可能分配 `Vec`；分配所有权很重要时使用调用方缓冲区 API。
- 通过 checked 公开 adapter 测试成功、不完整、非法、边界和有状态生命周期行为，不要只测 unsafe `Codec` 方法。
- 只有实际使用对应集成路径的 crate 才启用 `io` 与 `registry`。

## 延伸阅读

- [中文 README](../README.zh_CN.md) · [English README](../README.md) · [API 文档](https://docs.rs/qubit-codec)
- [生命周期感知 decode 示例](../examples/decode_lifecycle.rs)
