# Qubit Codec 用户手册

[中文 README](../README.zh_CN.md) · [English user guide](user_guide.md) · [API 文档](https://docs.rs/qubit-codec)

本文适用于 `qubit-codec` 0.17，要求 Rust 1.94 或更高版本。它面向需要在本地保留格式规则、同时复用容量规划、生命周期和流式转换机制的编解码器库与适配器库作者。若你只想直接使用现成的 hex、Base64 或字符集实现，应选用对应的领域库，而不是本库。读到[检查转换结果](#检查转换结果)，就能发布一个返回自有输出或写入调用方缓冲区的定宽 `Codec`；后续章节按需介绍增量流、生命周期解码、策略钩子、可选 I/O 接口以及 `registry` 功能。

## 目录

- [它解决什么问题](#它解决什么问题)
- [从哪里开始](#从哪里开始)
- [发布一个定宽 codec](#发布一个定宽-codec)
  - [只实现一次表示规则](#只实现一次表示规则)
  - [声明 Codec 上界](#声明-codec-上界)
  - [提供自有单值操作](#提供自有单值操作)
  - [把多个 value 编码到调用方缓冲区](#把多个-value-编码到调用方缓冲区)
  - [涉及的核心类型](#涉及的核心类型)
- [使用通用 JSON value codec](#使用通用-json-value-codec)
- [检查转换结果](#检查转换结果)
  - [成功时能看到什么](#成功时能看到什么)
  - [不完整输入与非法输入](#不完整输入与非法输入)
- [驱动增量流](#驱动增量流)
- [处理生命周期 decode 输出](#处理生命周期-decode-输出)
- [为畸形输入或 EOF 配置策略](#为畸形输入或-eof-配置策略)
- [选择字节序辅助类型](#选择字节序辅助类型)
- [接入缓冲 I/O](#接入缓冲-io)
- [在链接期注册 codec 供运行时查找](#在链接期注册-codec-供运行时查找)
  - [构造不依赖全局注册的局部注册表](#构造不依赖全局注册的局部注册表)
  - [从 0.16 升级 registry 消费者到 0.17](#从-016-升级-registry-消费者到-017)
- [错误、诊断与排障](#错误诊断与排障)
- [边界与实践清单](#边界与实践清单)
- [延伸阅读](#延伸阅读)

## 它解决什么问题

以一个二进制协议库为例。它要把 `u16` 字段编码成两个大端字节供报文使用，还要对外提供简单的 `encode(&u16) -> Vec<u8>` 供测试和一次性调用；之后还要从 socket 按块读入字节，不能把“只收到一个字节”误判成数据损坏。如果每一层各自实现下标运算、容量规划、reset/finish 生命周期，以及“还需要更多字节”和“字节本身不合法”的区分，同一格式很容易积累互不兼容的契约。

使用 `qubit-codec` 时，格式库在 `Codec` 上实现一次表示规则；带边界检查的适配器提供自有输出（`CodecValueEncoder` / `CodecValueDecoder`）、严格的调用方缓冲区转换（`CodecTranscode*`），以及可选的策略引擎钩子。本库**不**提供 hex、percent encoding、UTF-8 等具体格式，这些仍由领域库负责。本库也**不**替调用方决定 EOF：在调用方确认输入结束或采用文档化的 EOF 策略之前，合法但不完整的输入前缀只会被报告为不完整，而不是非法。

## 从哪里开始

1. [发布一个定宽 codec](#发布一个定宽-codec) 介绍 `U16BeCodec`、自有 encode/decode，以及批量写入调用方缓冲区。
2. [检查转换结果](#检查转换结果) 说明成功时的可观察结果，以及不完整输入与非法输入的区别。基础接入到这里结束。
3. 按需要查阅[驱动增量流](#驱动增量流)、[处理生命周期 decode 输出](#处理生命周期-decode-输出)、[为畸形输入或 EOF 配置策略](#为畸形输入或-eof-配置策略)、[接入缓冲 I/O](#接入缓冲-io)或[在链接期注册 codec 供运行时查找](#在链接期注册-codec-供运行时查找)。

可选功能 `json`、`io` 与 `registry` 相互独立，只在库实际需要相应能力时启用。`json` 提供基于 Serde 的通用 JSON 编解码器，输入必须是完整 JSON 字符串或字节。

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

    unsafe fn encode(&mut self, value: &u16, output: &mut [u8], output_index: usize) -> Result<usize, Infallible> {
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

`Codec::decode` 内的 `DecodeFailure::Incomplete` 表示可见输入前缀在补充更多单元后仍可能合法；`DecodeFailure::Invalid` 表示领域规则拒绝该输入。适配器会把这两类情况映射为 `TranscodeFailure` 或领域错误；除非所有下游调用方的处理确实相同，否则不要把“不完整”和“非法”合并成一种错误。

定宽 codec 的基础路径到这里结束。下面各节按需查阅：先是增量流生命周期，再是生命周期 decode 输出、策略 hooks、字节序辅助类型、I/O bridge 与注册表查找。

## 使用通用 JSON value codec

启用 `json` 后，可使用基于 Serde 的 codec，将 value 编码为自有 `String` 或 `Vec<u8>`，也可从完整 JSON 字符串或字节切片解码。编码要求 `T: serde::Serialize`；解码要求 `T: serde::de::DeserializeOwned`，确保结果自有数据。两个方向均返回 `serde_json::Error`：

```rust
use qubit_codec::{JsonBytesValueCodec, JsonStringValueCodec, ValueDecoder, ValueEncoder};

let mut text = JsonStringValueCodec::<String>::new();
let encoded = text.encode(&String::from("hello")).unwrap();
assert_eq!(text.decode(&encoded).unwrap(), "hello");
let mut bytes = JsonBytesValueCodec::<String>::new();
let encoded = bytes.encode(&String::from("hello")).unwrap();
assert_eq!(bytes.decode(&encoded).unwrap(), "hello");
```

同时启用 `json` 和 `registry` 时，内置描述符只注册 `serde_json::Value`，ID 为 `qubit.codec.json.string` 和 `qubit.codec.json.bytes`。自定义 DTO 可以直接使用通用 codec；若要注册，需提供自己的描述符和唯一 ID，不会自动注册所有泛型 DTO。详见[注册表查找](#在链接期注册-codec-供运行时查找)。

## 驱动增量流

对于 `TranscodeDecodeInput::transcode_step` 和
`AsyncTranscodeDecodeInput::poll_transcode`，只有校验成功的进度才会提交输入消费。
若解码或进度校验失败，本次调用不会消费本次 unread 范围，但 decoder 状态和目标输出
可能已经改变，且不会回滚；此时也没有完整进度报告。失败前发生的 refill 不会回滚底层
source。除非 decoder 定义了恢复协议，否则不要重放原输入。异步取消安全仅适用于 poll
仍处于 Pending 时的取消，不表示错误后会回滚。

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
| `Complete` | 从 `input_index` 起的全部可见输入均已消费。 | 提供下一段；源确认 EOF 后，先完成 `transcode_eof`（必要时传空切片），再调用 `finish`。 |
| `NeedInput` | 不完整尾部未被消费。 | 保留尾部并补充输入；EOF 时采用格式明确规定的策略（`transcode_eof` 或 hooks）。 |
| `NeedOutput` | 转换在超过输出容量之前停止。 | 排空或扩展输出，并按报告的进度继续。 |

只有调用方确认不会再收到源 unit 后才调用 `transcode_eof`。默认实现会把仍存在的 `NeedInput` 转为 `TranscodeFailure::IncompleteInput`。`finish` 不接收源输入尾部，因此无法重新解释它。

源确认 EOF 后，保留任何未读尾部，并根据每次 `transcode_eof` 返回的 `read()` 与 `written()` 推进游标。如果返回 `NeedOutput`，就排空或扩展输出并重试 EOF 操作；不要回到普通 `transcode`。EOF 操作返回 `Complete` 后，根据 `max_finish_output_len` 分配输出并调用 `finish`。如果某次调用已经完成 EOF 操作，就不要重复派发同一个 EOF 事件。这些操作不提供通用回滚；EOF 出错后，除非 transcoder 定义了恢复协议，否则不要重放原输入。

I/O bridge 可能在所有可见输入都已消费后，仍用空输入切片派发这次 EOF 调用。即使 unread 输入为空，只要 transcoder 返回 `NeedInput`，默认实现仍会报告 `IncompleteInput`。如果格式需要在 EOF 时排空内部缓存，或把空流视为正常结束，自定义 transcoder 必须覆盖 `transcode_eof`，并为这种情况返回相应进度。

内置 transcode engine 新建时尚未初始化。第一次成功操作必须是 `reset`；成功 `finish` 后，复用前必须再次 `reset`。生命周期操作失败可能使实例进入 poisoned 状态，直到下一次成功 `reset`。

## 处理生命周期 decode 输出

无状态 codec 的 reset/finish 上界为零。若 decoder 会从 `decode_reset` 或 `decode_finish` 输出 value，必须声明 `MAX_DECODE_RESET_VALUES` 或 `MAX_DECODE_FINISH_VALUES`。严格的 `CodecValueDecoder::decode` 会拒绝会产出生命周期 value 的输入；此时应改用 `decode_lifecycle` 或 `decode_lifecycle_with_scratch`。

可运行的[生命周期示例](../examples/decode_lifecycle.rs) 演示了一个在 decode 状态打开和关闭时输出 marker value 的单字节 codec。调用 `decode_lifecycle` 后，应分别检查 reset、main 与 finish 部分，而不是把它们合并成一个 main value。

## 为畸形输入或 EOF 配置策略

严格的 `CodecTranscode*` adapter 会直接返回领域失败。格式需要替换、跳过、计数或分阶段报告时，应保留共享循环并实现 hooks：

- `TranscodeEncodeEngine` 与 `TranscodeEncodeHooks` 处理不可编码 value 以及 encode reset/finish 策略；
- `TranscodeDecodeEngine` 与 `TranscodeDecodeHooks` 处理非法输入，以及 EOF 后的不完整输入；
- `TranscodeConvertEngine` 组合上述两组 hooks。

只有在输入源确认 EOF 后才调用 `transcode_eof`。保留尾部达到 `Codec::MIN_UNITS_PER_VALUE` 时，会先调用 `Codec::decode_eof`；只有它仍报告不完整输入时才调用 incomplete hook。hook 的 `source` 只有在 codec 通过 `DecodeFailure::incomplete_with_source` 返回携带 source 的 incomplete failure 时才是 `Some`。仅仅调用过 codec 并不能保证有 source：`DecodeFailure::incomplete` 不携带 source；短于 `Codec::MIN_UNITS_PER_VALUE` 的尾部也会在未调用 codec 的情况下进入 hook。默认动作为 `Reject`，在 codec 提供不完整输入的领域错误时会保留该错误。需要恢复时可以返回：

```rust
use qubit_codec::engine::DecodeIncompleteAction;

let skip = DecodeIncompleteAction::<char>::Skip;
let replacement = DecodeIncompleteAction::Emit { value: '\u{fffd}' };
```

`Skip` 会消费整个剩余尾部但不产生 value。`Emit` 同样消费整个尾部并写入一个替代 value，因此调用方必须提供输出槽位。下面的完整示例使用 `DecodeFailure::incomplete`，因此 codec 虽然处理了 `~` 前缀，hook 收到的仍是 `None`。若 codec 使用 `DecodeFailure::incomplete_with_source`，hook 才会收到 `Some(source)`。

```rust
use core::convert::Infallible;
use core::num::NonZeroUsize;

use qubit_codec::{Codec, DecodeFailure, TranscodeDecodeErrorOf};
use qubit_codec::engine::{
    DecodeContext, DecodeIncompleteAction, TranscodeDecodeEngine, TranscodeDecodeHooks,
};

struct PrefixCodec;

impl Codec for PrefixCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
        if input[index] == b'~' && input.len() - index < 2 {
            return Err(DecodeFailure::incomplete(NonZeroUsize::new(2).unwrap()));
        }
        Ok((input[index], NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<usize, Infallible> {
        output[index] = *value;
        Ok(1)
    }
}

struct ReplaceIncomplete;

impl TranscodeDecodeHooks<PrefixCodec> for ReplaceIncomplete {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut PrefixCodec,
        error: &Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<qubit_codec::engine::DecodeInvalidAction<u8>, TranscodeDecodeErrorOf<PrefixCodec>> {
        match *error {}
    }

    fn handle_incomplete_decode(
        &mut self,
        _codec: &mut PrefixCodec,
        source: Option<&Infallible>,
        _required_total: NonZeroUsize,
        _context: DecodeContext,
    ) -> Result<DecodeIncompleteAction<u8>, TranscodeDecodeErrorOf<PrefixCodec>> {
        assert!(source.is_none());
        Ok(DecodeIncompleteAction::Emit { value: b'?' })
    }
}

let mut decoder = TranscodeDecodeEngine::new(PrefixCodec, ReplaceIncomplete);
let mut output = [0_u8; 4];
assert_eq!(0, decoder.reset(&mut output, 0).unwrap());

// 普通流输入输出 `A`，并保留 `~` 等待后续 unit。
let progress = decoder.transcode(b"A~", 0, &mut output, 0).unwrap();
assert!(progress.is_need_input());
assert_eq!((1, 1), (progress.read(), progress.written()));

// EOF 后策略消费不完整的 `~` 并输出 `?`。
let eof = decoder
    .transcode_eof(b"A~", progress.read(), &mut output, progress.written())
    .unwrap();
assert!(eof.is_complete());
assert_eq!((1, 1), (eof.read(), eof.written()));
let written = progress.written() + eof.written();
let finished = decoder.finish(&mut output, written).unwrap();
assert_eq!(0, finished);
assert_eq!(b"A?", &output[..written + finished]);
```

当 framing、流状态或 EOF 行为无法用 `Codec` 和 hooks 表达时，应实现自定义 `Transcoder`。

## 选择字节序辅助类型

运行时配置使用 `ByteOrder`；静态选择有价值时使用 `ByteOrderSpec` 与 `BigEndian`、`LittleEndian` 或 `NativeEndian`。这些类型描述字节序策略，不实现具体整数 codec。前面的 `U16BeCodec` 场景把端序直接写在表示规则里。

## 接入缓冲 I/O

只有使用 `qubit-io` bridge 时才启用 `io` feature：

```toml
[dependencies]
qubit-io = "0.19"
qubit-codec = { version = "0.17", features = ["io"] }
```

`TranscodeDecodeInput` 与 `TranscodeEncodeOutput` 对接缓冲式 `qubit-io` trait。refill 确认 EOF 后，达到 codec 最低长度的尾部先交给 `decode_eof`；只有仍不完整才进入 hook。同步 `transcode` 在仍有目标容量且继续驱动到 EOF 时会调用 `transcode_eof`；如果 unread buffer 已空，就传入空输入，让 decoder 处理内部保留状态。显式逐步驱动时，`transcode_eof_step` 只要目标容量非零，也会把空输入交给 decoder。`rs-codec-misc` 的 percent codec 会在此时把末尾 `%` 或 `%A` 解释为畸形转义：bridge 决定何时输入结束，格式 codec 决定尾部含义。

EOF step 只接受 `Complete` 或 `NeedOutput`。如果自定义 decoder 在 EOF 后仍成功返回 `NeedInput`，bridge 返回 `InvalidData`，且不提交该进度的源消费。此校验不回滚 decoder 状态或 decoder 已写入的输出。

批量 decode `transcode` 的返回数量不能作为完成信号：短返回（包括零）可能表示 decoder 需要更大的连续输出范围。扩展输出容量后重试，或使用 `transcode_step` 与 `transcode_eof_step` 观察停止原因和所需容量。step 方法详见 [`TranscodeDecodeInput` Rustdoc](../src/transcode/io/transcode_decode_input.rs)。

异步输入先 reset decoder 并收集实际输出。每次 `AsyncTranscodeDecodeInput::poll_transcode` 返回 progress 后，bridge 已应用 `read()`；调用方只需按 `written()` 推进输出游标，不能再次消费源输入。遇到 `NeedInput { required }` 时，等待 `fill_until_async(required.get())`：返回 `true` 就继续普通 poll；返回 `false` 表示 EOF 已确认，同时保留不足长度的 unread 尾部。把尾部交给 `transcode_eof_step`，逐步收集 `written()`，遇到 `NeedOutput` 时提供足够空间重试。如果 `poll_transcode` 返回 `AsyncTranscodeDecodeStep::EndOfInput`，即使 unread buffer 为空也要调用 `transcode_eof_step`；decoder 可能需要依据内部保留状态处理 EOF 并产出结果。EOF 步骤返回 `Complete` 后调用 `finish` 并收集其输出。策略分支可能预检完整动作的保守输出容量，因此即使 `Skip` 或 `Reject` 也可能先返回 `NeedOutput`。详见 [`AsyncTranscodeDecodeInput` Rustdoc](../src/transcode/io/async_transcode_decode_input.rs)。

异步 EOF step 遵守相同规则：若自定义 decoder 在 EOF 后成功返回 `NeedInput`，bridge 返回 `InvalidData`，且不提交该步的源消费；decoder 状态和已写入的输出不会回滚。

同步 `transcode` 会在编码前预留本批次完整输出上界，保证整批输入消费；大批次可能需要较多内存。需要整批消费时继续使用该入口；需要逐步驱动大输入时，可使用 `transcode_step`：

下面是可独立复制到文件并运行的完整程序。它用两字节编码器写入缓冲 bridge，并检查最终字节；同一份源码也维护在 [`examples/io_transcode_step.rs`](../examples/io_transcode_step.rs) 中。

<!-- io-guide-start -->
```rust
use core::convert::Infallible;
use core::num::NonZeroUsize;
use std::io::Cursor;
use std::io::Error;

use qubit_codec::Codec;
use qubit_codec::CodecTranscodeEncoder;
use qubit_codec::DecodeFailure;
use qubit_codec::TranscodeEncodeOutput;
use qubit_codec::TranscodeStatus;

/// Encodes one fixed-width big-endian value.
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
        Ok((
            u16::from_be_bytes([input[input_index], input[input_index + 1]]),
            NonZeroUsize::new(2).unwrap(),
        ))
    }

    unsafe fn encode(&mut self, value: &u16, out: &mut [u8], index: usize) -> Result<usize, Infallible> {
        debug_assert!(index + 2 <= out.len());
        out[index..index + 2].copy_from_slice(&value.to_be_bytes());
        Ok(2)
    }
}

/// Runs a complete stepwise encode lifecycle and checks the delivered bytes.
fn main() -> Result<(), Error> {
    let values = [0x1234_u16, 0xabcd];
    let mut encoder = CodecTranscodeEncoder::new(U16BeCodec);
    let mut output = TranscodeEncodeOutput::with_capacity(Cursor::new(Vec::<u8>::new()), 2);
    let mut map_error = |_| Error::other("encode error");
    output.reset(&mut encoder, &mut map_error)?;
    let mut offset = 0;
    while offset < values.len() {
        let progress = output.transcode_step(&mut encoder, &mut map_error, &values, offset, values.len() - offset)?;
        offset += progress.read();
        if let TranscodeStatus::NeedOutput { required, .. } = progress.status() {
            output.ensure_spare_capacity(required.get())?;
        }
    }
    output.finish(&mut encoder, &mut map_error)?;
    assert_eq!(output.inner().get_ref(), &[0x12, 0x34, 0xab, 0xcd]);
    Ok(())
}
```
<!-- io-guide-end -->

运行命令：`cargo run --example io_transcode_step --features io`。

返回的 `written()` 已提交到 bridge，不能再次 advance；先按 `read()` 推进源游标，再申请下一步空间。`count` 为零时只验证范围，不调用 encoder，也不交付 pending 输出。单值上界只是预检估算：`NeedOutput` 可能请求更多空间，且历史缓冲容量不会自动缩小。现有下游缓冲 writer 不会自动改用此入口。操作可能先交付已缓冲输出再接受下一步；异步 poll 则先建立背压，encoder 尚未调用时就可能返回 `Pending`。显式 `flush` 仍是 sink 刷新边界。`finish_to_buffer` 只把 encoder 收尾输出追加到内存，不执行 I/O；`finish` 会交付 pending 并刷新 sink。

错误不提供通用事务回滚或完整进度。范围或容量预检错误发生在 encoder 调用前，但准备容量时可能已交付旧 pending；sink 交付错误遵循底层输出的 pending 与恢复契约。如果 encoder 或进度验证失败，encoder 可能已经推进，但失败调用写入的单位不会提交到该 bridge 的 pending 输出。除非具体 encoder 定义了恢复协议，否则不要从原输入范围起点重放。成功的 reset 可以开始新的逻辑流，但不能撤销旧流已交付的输出。

## 在链接期注册 codec 供运行时查找

当多个 crate 需要按稳定 ID 在链接期发现双向 value codec（字符串或字节 whole-value）时，启用 `registry` feature：

```toml
[dependencies]
qubit-codec = { version = "0.17", features = ["registry"] }
```

为固定 value 类型成对实现 `ValueEncoder` 与 `ValueDecoder` 后注册。字符串 wire 与字节 wire 使用不同的 codec 类型与注册宏，各自进入独立的进程级注册表；同一 `ValueCodecId` 可以同时出现在两张表中。

注册 ID 与注册表内容都是静态的：宏 ID 必须是编译期常量表达式，`ValueCodecId::try_new` 接受 `&'static str`，`ValueCodecRegistry::from_registrations` 也要求静态注册项。每个进程级 `global()` 注册表会从已链接注册项构建一次，此后保持不可变；没有运行时追加注册项的 API。运行时仍可在已有注册 codec 中选择：将配置 ID 保留为字符串，并用接受 `&str` 的 `registry.get(configured_id.as_str())` 查找。一个 ID 若有两个以上来源，诊断会按稳定顺序列出全部来源。

### 构造不依赖全局注册的局部注册表

`ValueCodecRegistry::from_registrations` 从静态注册项构造独立注册表。`ValueCodecId::new` 创建已验证的 ID；注册项及其 descriptor 必须位于静态存储中，因为构造函数接收 `&'static ValueCodecRegistration`。此调用不使用注册宏、不读取 inventory，也不会改变进程级 `global()` 注册表。若 ID 重复，会返回 `ValueCodecRegistryError`。

构造 descriptor 时遵循相同的错误类型约束：encoder 和 decoder 的错误类型必须实现 `Error + Send + Sync + 'static`。descriptor 会擦除具体错误类型，执行错误可通过 error source 暴露它们；codec 与 value 类型本身无需这些上界。decode 结果经擦除 API 以 `Box<dyn Any>` 返回，已知具体输出类型时可用 `downcast` 取回。

```rust
use std::convert::Infallible;
use qubit_codec::ValueCodecId;
use qubit_codec::ValueCodecRegistration;
use qubit_codec::ValueCodecRegistrationSource;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_codec::ValueStringCodecDescriptor;
use qubit_codec::ValueStringCodecRegistration;
use qubit_codec::ValueStringCodecRegistry;

#[derive(Default)]
struct LocalText;

impl ValueEncoder<String> for LocalText {
    type Output = String;
    type Error = Infallible;

    fn encode(&mut self, input: &String) -> Result<String, Infallible> {
        Ok(input.clone())
    }
}

impl ValueDecoder<str> for LocalText {
    type Output = String;
    type Error = Infallible;

    fn decode(&mut self, input: &str) -> Result<String, Infallible> {
        Ok(input.to_owned())
    }
}

static DESCRIPTOR: ValueStringCodecDescriptor =
    ValueStringCodecDescriptor::of::<LocalText, String>();
static REGISTRATION: ValueStringCodecRegistration = ValueCodecRegistration::new(
    ValueCodecId::new("example.local.text"),
    &DESCRIPTOR,
    ValueCodecRegistrationSource::new("local-example", "example", file!(), line!()),
);

let registry = ValueStringCodecRegistry::from_registrations([&REGISTRATION])
    .expect("unique local registration");
let descriptor = registry.get("example.local.text")
    .expect("registered locally").descriptor();
let encoded = descriptor.encode(&String::from("hello")).expect("encode");
assert_eq!("hello", encoded);
let decoded = descriptor.decode(&encoded).expect("decode");
assert_eq!("hello", *decoded.downcast::<String>().expect("String output"));
assert!(registry.get("example.missing").is_none());
```

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

也可以通过借用的 `str` 视图编码自有 `String`。`value` 仍是擦除和解码得到的类型；`input` 只选择 encoder 接收的借用输入：

```rust
use std::io::Error;
use qubit_codec::{ValueDecoder, ValueEncoder, ValueStringCodecRegistry, register_value_string_codec};

#[derive(Default)]
struct BorrowedTextCodec;

impl ValueEncoder<str> for BorrowedTextCodec {
    type Output = String;
    type Error = Error;

    fn encode(&mut self, input: &str) -> Result<String, Error> {
        Ok(input.to_owned())
    }
}

impl ValueDecoder<str> for BorrowedTextCodec {
    type Output = String;
    type Error = Error;

    fn decode(&mut self, input: &str) -> Result<String, Error> {
        Ok(input.to_owned())
    }
}

register_value_string_codec!(
    id = "example.borrowed.text",
    codec = BorrowedTextCodec,
    value = String,
    input = str,
);

fn main() {
    let configured_id = String::from("example.borrowed.text");
    let registry = ValueStringCodecRegistry::try_global().expect("valid registrations");
    let descriptor = registry
        .get(configured_id.as_str())
        .expect("known codec ID")
        .descriptor();
    let encoded = descriptor.encode(&String::from("hello")).expect("encoding succeeds");
    assert_eq!(encoded, "hello");
    let decoded = descriptor.decode(&encoded).expect("decoding succeeds");
    assert_eq!(*decoded.downcast::<String>().expect("value type is String"), "hello");
}
```

字节视图可使用 `value = Vec<u8>, input = [u8]`；descriptor 仍核对并返回 `Vec<u8>`。

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

在上述 `U32BeBytesCodec` 注册已链接进程序后，可以用运行时持有的字符串 ID 完成查找与往返：

```rust
use qubit_codec::ValueBytesCodecRegistry;

let configured_id = String::from("example.u32");
let registry = ValueBytesCodecRegistry::try_global().expect("valid registrations");
let registration = registry.get(configured_id.as_str()).expect("known codec ID");
let descriptor = registration.descriptor();
let bytes = descriptor.encode(&42_u32).expect("u32 encoding succeeds");
let decoded = descriptor.decode(&bytes).expect("u32 decoding succeeds");
let value = decoded.downcast::<u32>().expect("registered value type is u32");
assert_eq!(42, *value);
```

`ValueStringCodecRegistry::try_global()` 与 `ValueBytesCodecRegistry::try_global()` 在每个进程内分别收集一次已链接的注册项，并缓存初始化失败，包括重复 ID 冲突；之后每次调用都会返回缓存的冲突错误。对应的 `global()` 在注册冲突时会 panic，因此需要处理注册冲突的应用应使用 `try_global()`。成功构建的注册表保持不可变。宏与注册 ID 使用静态 ID 并在声明编译时校验；运行时查找使用接受 `&str` 的 `get`，并按原样 ID 字符串匹配，不做规范化。执行前会先核对擦除输入的实际类型。重复 ID 在同一注册表内属于注册错误，不会静默覆盖。该路径只擦除注册时 value 类型已固定的双向 whole-value codec，不能替代 `Codec` 流式 adapter。

注册 codec 的关联错误必须实现 `Error + Send + Sync + 'static`，这样执行错误才能在线程间保留原始 source。成功解码值仍返回 `Box<dyn Any>`；这个约束不代表整个 decode 结果都能跨线程传递。直接调用 `ValueEncoder` 和 `ValueDecoder` 不受影响；registry descriptor 构造器要求错误类型满足线程安全约束。

### 从 0.16 升级 registry 消费者到 0.17

凡是需要交换 `ValueCodecId`、registration、descriptor 或 registry 类型的 crate，都应统一依赖 `qubit-codec = "0.17"`。Cargo 可以同时编译 0.16 和 0.17，但两版的 Rust 类型与进程级 inventory 相互独立：通过 0.17 注册的 codec 不会出现在 0.16 的 registry 中。若泛型集成对外暴露已注册 codec 的错误类型，应把 bound 更新为 `Error + Send + Sync + 'static`；也可以在该集成边界转换不满足条件的错误。使用缓冲 I/O adapter 的 crate 也要在同一份 0.17 依赖上启用 `io` feature。

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
| 文件末尾出现 `NeedInput`，包括 unread 输入为空时 | 保留尚未消费的尾部并调用 `transcode_eof`；若格式允许空输入结束，应覆盖默认 EOF 行为，在 `finish` 前完成转换或排空内部状态。 |
| `NeedOutput` 持续重复 | 推进两个进度计数；提供报告的容量；核对自定义上界。 |
| 流式编码返回错误 | 区分预检、sink 交付及 encoder/进度失败；encoder 出错时可能已推进状态但没有完整进度。 |
| 自有 decode 拒绝看似有效的输入 | 检查 trailing unit 或 decode reset/finish 输出；必要时使用生命周期感知 decode。 |
| 转换前容量被拒绝 | 纳入 reset、main 与 finish 上界，并检查算术溢出。 |
| `TranscodeBeforeReset` 或 `TranscodeAfterFinish` | 在第一条流及复用前调用 `reset`。 |
| 替换逻辑正在演变成第二套循环 | 把决策移入 engine hooks。 |
| 注册表查找失败或执行类型不对 | 确认对应 `register_value_string_codec!` / `register_value_bytes_codec!` 已链接、在该注册表内 ID 唯一、输入类型与注册一致。 |

## 边界与实践清单

- 具体格式、字符集和高层 reader/writer adapter 应留在领域 crate，而不是放进 `qubit-codec`。
- `NeedInput` 是流式信号，不是最终 EOF。确认 EOF 后，bridge 可能用空切片调用 `transcode_eof`；默认实现仍把 `NeedInput` 视为不完整，因此允许空 EOF 或需排空内部状态的格式必须显式实现该行为。
- 容量方法必须覆盖所有可达瞬态，不能只看典型输出大小。
- 自有输出 adapter 可能分配 `Vec`；分配所有权很重要时使用调用方缓冲区 API。
- 通过 checked 公开 adapter 测试成功、不完整、非法、边界和有状态生命周期行为，不要只测 unsafe `Codec` 方法。
- 只有实际使用对应集成路径的 crate 才启用 `io` 与 `registry`。

## 延伸阅读

- [中文 README](../README.zh_CN.md) · [English README](../README.md) · [API 文档](https://docs.rs/qubit-codec)
- [生命周期感知 decode 示例](../examples/decode_lifecycle.rs)
