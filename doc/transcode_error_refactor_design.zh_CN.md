# Transcode 错误与生命周期设计

本文描述当前 `rs-codec` 的实现，而不是历史迁移方案。代码允许破坏性变更，因此
这里以最小、可验证的公共模型为准。

## 设计边界

transcode 层只负责缓冲区游标、容量、进度和流生命周期。codec 或 hook 负责格式域
语义。错误因此分成三层：

1. `TranscodeFailure`：框架能独立判断的结构性失败。
2. `TranscodeDomainError<E>`：带 `reset`、`main`、`finish` 阶段的 codec/hook 域错误。
3. 方向错误：解码、编码、转换分别承载各自可能出现的错误方向。

## 公共错误类型

`TranscodeFailure` 是无泛型、`#[non_exhaustive]` 的框架错误，当前包含：

- 输入/输出下标、输出范围和输出容量错误；
- 输出长度算术溢出；
- 不完整输入和尾随输入；
- 严格单值 decode 不支持 codec 生命周期输出；
- `TranscodeBeforeReset`、`FinishBeforeReset`；
- `TranscodeAfterFinish`、`FinishAfterFinish`；
- reset 或 finish 部分执行失败后的 `LifecyclePoisoned`。

三类方向错误的实际形状如下：

```rust
enum TranscodeDecodeError<E> {
    Failure(TranscodeFailure),
    Domain(TranscodeDomainError<E>),
}

enum TranscodeEncodeError<E, V> {
    Failure(TranscodeFailure),
    Unencodable { input_index: usize, value: Option<V> },
    Domain(TranscodeDomainError<E>),
}

enum TranscodeConvertError<DE, EE, V> {
    Failure(TranscodeFailure),
    DecodeDomain(TranscodeDomainError<DE>),
    EncodeDomain(TranscodeDomainError<EE>),
    Unencodable { input_index: usize, value: Option<V> },
}
```

`TranscodeFailure` 是结构性错误的唯一实现来源。方向错误为了保持调用方迁移成本低，
仍保留 `invalid_*`、`ensure_*`、`map_*` 等转发便利方法；这些方法只把结果转换成
`Failure`，不复制或扩展框架语义。

## 强制生命周期

所有引擎和 `Transcoder` adapter 都遵循：

```text
reset → transcode* → finish
          ↑       ↓
       可返回 NeedInput / NeedOutput
```

新建实例处于 `Uninitialized`，只有一次成功的 `reset` 后才能调用 `transcode` 或
`finish`。`finish` 成功后实例处于 `Finished`，必须再次 `reset` 才能开始下一条流。
reset 或 finish 在 codec/hook 已可能修改状态后失败，会进入 `Poisoned`；只有成功的
后续 reset 能恢复。容量和下标预检失败不改变可重试状态。

`LifecycleGuard` 在所有构建 profile 都执行检查，避免 debug/release 行为分叉。对应
的错误是 `TranscodeBeforeReset`、`FinishBeforeReset`、`TranscodeAfterFinish`、
`FinishAfterFinish` 和 `LifecyclePoisoned`。

## 编码 hook 上下文

`EncodeContext<'a, Value>` 是只读值对象，只暴露：

```rust
input_value() -> &Value
input_index() -> usize
output_index() -> usize
available_output() -> usize
```

hook 不再取得可写输出切片，也不能伪造写入计数。引擎内部的私有
`EncodeAttempt` 独占 `&mut [Unit]`，负责把 hook 返回的策略（拒绝、跳过或替换）落实
到 codec 调用和进度结算中。这样输出写入、边界检查和 `written` 计数只有一个可信来源。

## 引擎执行顺序

- `TranscodeDecodeEngine`：reset hook → `Codec::decode_reset` → 重复 decode →
  `Codec::decode_finish`/finish hook。
- `TranscodeEncodeEngine`：reset hook → `Codec::encode_reset` → 重复 encode →
  `Codec::encode_finish`/finish hook。
- `TranscodeConvertEngine`：先 reset 目标 encoder，再处理源 decoder 的 reset 值；
  streaming 阶段在内部 pending slot 中保留尚未写出的值，finish 时先排空 pending，
  再处理 decoder finish 值，最后完成 encoder finish。

所有阶段都使用绝对下标，返回的 `TranscodeProgress` 计数是相对本次调用的读取和
写入量。容量 bound 必须覆盖 reset、transcode、finish 的所有可达瞬态。

## EOF 语义

普通 `transcode` 只处理当前可见输入；`NeedInput` 的尾部仍由调用方持有。调用方声明
EOF 后，可使用 codec-backed decoder 的 `transcode_eof`，由 codec 的
`decode_eof` 决定是否能完成尾部。`finish` 不接收或重新解释调用方的输入尾部，只
处理引擎已经保留的状态和 codec/hook 的 finish 输出。

## 取舍与暂缓项

- 不恢复旧的 phantom `FailureValue`/统一错误枚举；方向错误直接表达 decode、encode
  和 convert 语义。
- 保留方向错误的 forwarding helper，因为它们是零语义的 API 便利层，并不造成错误
  模型重复。
- `D::Value: Default` 对 converter finish 冷路径的限制暂不改变；普通 transcode 热
  路不依赖该 bound，后续可另行设计无需 Default 的 finish scratch。
- 不把 `EncodeContext` 的内部输出权限暴露给公共 hook API；若未来确需更强扩展点，
  应新增显式的 engine-owned writer，而不是恢复裸 `&mut [Unit]`。

## 验证策略

生命周期测试覆盖首次 reset 前的拒绝、reset 后 streaming、finish 后拒绝和 reset 复用。
编码上下文测试验证 hook 只能观察元数据。`benches/transcode.rs` 中的
`safe_vs_unchecked` 基准对比安全索引与 `get_unchecked`，并配合 release 汇编输出检查
优化器是否已经消除边界检查；基准结果不作为 API 正确性的依据。

上述约束与 `src/transcode`、`tests/transcode`、README 及下游 codec/text adapter 的
当前实现保持一致。
