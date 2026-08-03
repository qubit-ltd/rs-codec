# Qubit Codec 用户手册

[English](user_guide.md) · [README](../README.zh_CN.md) · [API 文档](https://docs.rs/qubit-codec)

本手册适用于 `qubit-codec` 0.11 和 Rust 1.94 及以上版本，面向 codec 与 adapter
crate 作者，而不是寻找某种具体文件格式或字符集实现的应用开发者。

## 手册目标与读者

当格式 crate 需要以下一种或多种可复用边界时，应使用 `qubit-codec`：

- 处理一个逻辑 value 或 codec quantum 的底层契约；
- 基于该契约的自有输出 facade；
- 显式报告进度的调用方缓冲区流式 adapter；
- 处理非法或不可编码输入的策略 hooks；
- 基于 `qubit-io` 的缓冲 I/O 集成。

格式 crate 继续拥有表示规则和领域错误；`qubit-codec` 负责共享机制，包括下标、容量
规划、进度、reset/finish 生命周期，以及不完整输入与非法输入之间的区分。

## 概念模型

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

应选择能保持格式真实边界的最小层次：

| 需求 | 推荐 API |
| --- | --- |
| 一个逻辑 value 对应若干编码 unit | 实现 `Codec` |
| 只有完整输入才有实用含义 | 直接实现 `ValueEncoder` / `ValueDecoder` |
| 已有 `Codec`，需要自有结果 | `CodecValueEncoder` / `CodecValueDecoder` |
| 已有 `Codec`，需要严格流式处理 | `CodecTranscode*` adapter |
| 非法或不可编码输入需要替换、跳过或报告 | transcode engine 与 hooks |
| 流式规则需要自定义 EOF 或 framing 决策 | 实现 `Transcoder` |

定宽整数或字符通常有合理的 `Codec` value 边界。格式化 hex 字符串、percent-encoded
字符串或 C 字符串字面量通常没有；这些格式直接采用 value-level 实现更清晰。

## 场景：以两个层次发布一个定宽 codec

假设一个二进制格式 crate 需要将 `u16` 编码为两个大端字节。它的成功标准很具体：

1. `0x1234` 编码为 `[0x12, 0x34]`；
2. `[0x12, 0x34]` 解码为 `0x1234`；
3. 一个输入字节被报告为不完整，而不会传入 unsafe codec 入口；
4. 同一个 codec 可以把多个 value 编码到调用方拥有的缓冲区。

这个 crate 只需实现一次表示规则：

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

checked adapter 随即提供两个发布层次；格式 crate 无需重复实现容量或生命周期代码。

## 安装与最小配置

```toml
[dependencies]
qubit-codec = "0.11"
```

默认 feature 集为空，上述场景不需要任何 feature。只有使用 `qubit-io` bridge 时才
启用 `io`：

```toml
[dependencies]
qubit-codec = { version = "0.11", features = ["io"] }
```

## 核心工作流

### 1. 声明精确的 `Codec` 上界

三个必需常量是公开的安全与容量契约，而不是性能提示：

| 契约 | 含义 |
| --- | --- |
| `MIN_UNITS_PER_VALUE` | 可能容纳一个 decoded value 的最小可读输入，必须非零。 |
| `MAX_DECODE_UNITS_PER_VALUE` | 成功 decode 的最大消费量或不完整重试需求，必须非零且不小于最小值。 |
| `MAX_ENCODE_UNITS_PER_VALUE` | 主 encode 阶段与 value 无关的输出上界；只有刻意缓冲时才允许为零。 |

默认 `encode_len` 返回 `MAX_ENCODE_UNITS_PER_VALUE`，对本场景的定宽编码是精确值。
变宽或有状态 codec 必须覆盖该方法。在相同 value 与 codec 状态下，成功的 `encode`
必须恰好写入它报告的长度。

当 `Value` 包含编码域之外的值时，应覆盖 `can_encode_value`。checked encoder 会在
`encode_len` 和 unsafe `encode` 之前调用它。

### 2. 保持 unsafe 入口短小

checked adapter 会在进入 `Codec::encode` 或 `Codec::decode` 前建立文档规定的下标和
容量前置条件。实现仍必须：

- 只在前置条件允许的范围内读写；
- 成功 decode 时返回非零消费量，且不超过 decode 上界；
- 出错时保持状态一致；
- 对需要更多 unit 的合法开放流前缀返回 `DecodeFailure::Incomplete`，对领域内畸形
  输入返回 `DecodeFailure::Invalid`；
- 将 reset、main 和 finish 输出限制在声明的上界内。

应像贯穿场景一样，在入口处用 `debug_assert!` 明确假定的范围。

### 3. 提供自有单值操作

`CodecValueEncoder` 与 `CodecValueDecoder` 会执行完整 codec 生命周期并返回自有输出：

```rust
use qubit_codec::{CodecValueDecoder, CodecValueEncoder, ValueEncoder};

let mut encoder = CodecValueEncoder::new(U16BeCodec);
let encoded = encoder.encode(&0x1234).expect("encoding is infallible");
assert_eq!(vec![0x12, 0x34], encoded);

let mut decoder = CodecValueDecoder::new(U16BeCodec);
let decoded = decoder.decode(&encoded).expect("input contains one u16");
assert_eq!(0x1234, decoded);
```

严格单值 decode 只允许一个 main value。额外 unit 会产生
`TranscodeFailure::TrailingInput`。如果 codec 声明从 `decode_reset` 或
`decode_finish` 输出 value，则必须改用 `decode_lifecycle` 或
`decode_lifecycle_with_scratch`；可运行的[生命周期示例](../examples/decode_lifecycle.rs)
会分别保留 reset、main 与 finish 输出。

### 4. 提供调用方缓冲区转换

`CodecTranscodeEncoder` 将相同 codec 应用于一组 value。one-shot helper 会计算容量并
执行完整生命周期，同时让调用方拥有缓冲区：

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

unit-to-value 使用 `CodecTranscodeDecoder`；严格的 decode-encode 管线使用
`CodecTranscodeConverter`。只有一侧确实需要策略决策时才选择 engine。

### 5. 正确驱动增量流

显式生命周期如下：

```text
计算 reset 输出 -> reset
                    |
                    v
保留输入尾部 <- transcode/transcode_eof -> 排空或扩展输出
                    |
                    v
              计算 finish 输出 -> finish
```

`TranscodeProgress::read()` 与 `written()` 都相对于本次调用传入的下标。重试前必须
推进两个游标。

| 状态 | 含义 | 调用方动作 |
| --- | --- | --- |
| `Complete` | 从 `input_index` 起的全部可见输入均已消费。 | 提供下一段，或在 EOF 时 finish。 |
| `NeedInput` | 不完整尾部未被消费。 | 保留尾部并补充输入；EOF 时采用格式明确规定的策略。 |
| `NeedOutput` | 转换在超过输出容量之前停止。 | 排空或扩展输出，并按报告的进度继续。 |

只有调用方确认不会再收到源 unit 后才调用 `transcode_eof`。其默认实现会把仍存在的
`NeedInput` 转为 `TranscodeFailure::IncompleteInput`。`finish` 不接收源输入尾部，
因此无法重新解释它。

内置 transcode engine 新建时尚未初始化。第一次成功操作必须是 `reset`；成功
`finish` 后，复用前必须调用 `reset`。生命周期操作失败可能使实例进入 poisoned 状态，
直到下一次成功 reset。

## 进阶用法

### 生命周期输出

无状态 codec 使用零 reset/finish 上界。有状态 encoder 写入 header 或 trailer 时声明
`MAX_ENCODE_RESET_UNITS` 或 `MAX_ENCODE_FINISH_UNITS`；decoder 输出生命周期 value
时声明 `MAX_DECODE_RESET_VALUES` 或 `MAX_DECODE_FINISH_VALUES`。上界必须覆盖每个
可达瞬态，而不只是当前状态。

### 策略 hooks

严格的 `CodecTranscode*` adapter 会返回领域失败。格式需要替换、跳过、计数或分阶段
报告时，应保留共享循环并实现 hooks：

- `TranscodeEncodeEngine` 与 `TranscodeEncodeHooks` 处理不可编码 value 以及 encode
  reset/finish 策略；
- `TranscodeDecodeEngine` 与 `TranscodeDecodeHooks` 处理非法输入，以及 EOF 后的
  不完整输入；
- `TranscodeConvertEngine` 用两组 hooks 组合 decode 与 encode engine。

对于 codec 与 hooks 无法表达的延迟 framing、专用流状态或 EOF 行为，应实现自定义
`Transcoder`。

### EOF 不完整 decode 策略

只有在输入源确认 EOF 后才调用 `transcode_eof`。对于
`TranscodeDecodeEngine`，它随后会调用 hook 的
`handle_incomplete_decode`。默认动作为 `Reject`：若 codec 提供不完整输入的领域错误，
该错误会被保留；否则返回框架错误 `TranscodeFailure::IncompleteInput`。

需要恢复的 hook 可返回其他公开动作：

```rust
use qubit_codec::engine::DecodeIncompleteAction;

let skip = DecodeIncompleteAction::<char>::Skip;
let replacement = DecodeIncompleteAction::Emit { value: '\u{fffd}' };
```

`Skip` 会消费整个剩余尾部但不产生 value。`Emit` 同样消费整个尾部，并写入一个替代
value，因此调用方必须提供一个输出槽位。若 codec 已被调用并报告输入不完整，hook 收到
`Some(source)`；若尾部短于 `Codec::MIN_UNITS_PER_VALUE`，则收到 `None`。因此格式
crate 可以为两种情形应用同一个明确的 EOF 策略，而无需把开放流中的 `NeedInput` 视为错误。

### 字节序与 I/O

运行时配置使用 `ByteOrder`；静态选择有价值时使用 `ByteOrderSpec` 与 `BigEndian`、
`LittleEndian` 或 `NativeEndian`。这些类型描述字节序策略，但不实现具体整数 codec。

启用 `io` feature 后，`TranscodeDecodeInput` 与 `TranscodeEncodeOutput` 桥接缓冲式
`qubit-io` trait。`TranscodeDecodeInput::transcode` 在 refill 确认 EOF 后，会将保留
尾部交给 `transcode_eof`；需要显式逐步控制时使用
`TranscodeDecodeInput::transcode_eof_step`。异步对应物会报告
`AsyncTranscodeDecodeStep::EndOfInput`，随后在 `finish` 前调用
`AsyncTranscodeDecodeInput::transcode_eof_step`。

## 错误与诊断

| 错误 | 边界 | 恢复方式 |
| --- | --- | --- |
| `DecodeFailure::Incomplete` | 开放流 codec 输入是合法前缀，但长度不足。 | 保留尾部并重试，或显式做 EOF 决策。 |
| `DecodeFailure::Invalid` | unit 畸形、非规范或不可映射。 | 采用领域策略或返回 codec 错误。 |
| `TranscodeFailure` | 下标、容量、完整输入形状、分配或生命周期用法非法。 | 修正调用方状态并检查结构化 variant。 |
| `CapacityError` | 容量算术无法产生有效的 `usize` 上界。 | 在分配或写入前拒绝本次规划。 |
| `TranscodeDomainError<E>` | codec 或 hook 在 reset、main 或 finish 阶段失败。 | 报告时保留阶段和领域 source。 |
| 方向性 transcode error | encode、decode 或 conversion 失败。 | 保留方向；encode/conversion 错误可能携带不可编码 value。 |
| `TranscodeContractError` | 自定义 transcoder 返回了不一致进度。 | 修复 transcoder 实现；这不是可恢复输入。 |

在贯穿场景中，checked decoder 会在调用 unsafe `decode` 之前拒绝只有一个字节的完整
输入：

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

除非所有下游调用方确实对两者采取相同动作，否则不要把不完整输入与非法输入压平为
同一种错误。

## 排障

| 症状 | 按顺序检查 |
| --- | --- |
| 文件末尾出现 `NeedInput` | 确认已保留尾部；调用 `transcode_eof`；在 `finish` 前采用格式专属 EOF 规则。 |
| `NeedOutput` 持续重复 | 推进两个进度计数；提供报告的容量；核对自定义上界与计数。 |
| 自有 decode 拒绝看似有效的输入 | 检查 trailing unit 或声明的 decode reset/finish 输出；必要时使用生命周期感知 decode。 |
| 转换前容量被拒绝 | 纳入 reset、main 与 finish 上界，并检查算术溢出。 |
| `TranscodeBeforeReset` 或 `TranscodeAfterFinish` | 在第一条流及复用前调用 `reset`。 |
| 畸形输入替换逻辑正在演变成第二套循环 | 把决策移入合适的 engine hooks。 |

## 限制与最佳实践

- 具体格式、字符集和高层 reader/writer adapter 应留在领域 crate。
- 保持 unsafe codec 方法短小；通过 checked 公开 adapter 测试成功、不完整、非法、边界
  和有状态生命周期行为。
- 将容量方法视为覆盖所有可达状态且与当前状态无关的安全上界，而不是典型输出估计。
- 自有输出 adapter 会分配 `Vec`；分配所有权很重要时使用调用方缓冲区 API。
- `NeedInput` 是流式边界信号，而不是最终 EOF。
- 只有使用 `qubit-io` bridge 类型的 crate 才启用 `io`。

## 延伸阅读

- [README](../README.zh_CN.md)
- [English user guide](user_guide.md)
- [API 文档](https://docs.rs/qubit-codec)
- [生命周期感知 decode 示例](../examples/decode_lifecycle.rs)
