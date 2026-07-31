# Transcode 错误模型重构设计

本文重新梳理 `rs-codec` 的 transcode 错误模型。约束只有一个：允许破坏性
变更，不保留旧 API 兼容别名，不为了迁移工作量牺牲 API 形状。

## 目标

新的错误模型应让类型结构和真实执行路径一致：

- 解码只报告框架失败或解码域错误。
- 编码只报告框架失败、编码域错误或不可编码值。
- 转换显式区分源端解码、目标端编码和不可编码中间值。
- `Transcoder` 只表达共享的流式转换机械协议；方向语义由
  `TranscodeDecoder` / `TranscodeEncoder` / `TranscodeConverter` 具体化。
- `Transcoder` 不再暴露 `DomainError` / `FailureValue` 这类为旧错误枚举服务的
  关联类型。
- 引擎内部不再需要 `map_failure_value(|()| unreachable!(...))` 这种“人知道但
  类型系统不知道”的分支。

## 当前问题

现在的核心类型是：

```rust
pub enum TranscodeFailure<Value = ()> {
    InvalidInputIndex { index: usize, input_len: usize },
    InvalidOutputIndex { index: usize, output_len: usize },
    InsufficientOutput { output_index: usize, required: usize, available: usize },
    OutputLengthOverflow,
    IncompleteInput { input_index: usize, required: usize, available: usize },
    TrailingInput { consumed: usize, remaining: usize },
    UnencodableValue { input_index: usize, value: Option<Value> },
}

pub enum TranscodeError<E, Value = ()> {
    Failure(TranscodeFailure<Value>),
    Domain(TranscodeDomainError<E>),
}

pub trait Transcoder {
    type Input;
    type Output;
    type DomainError;
    type FailureValue;
}
```

问题不是“泛型多”，而是泛型表达了错误的边界：

- `Value` 只属于 `UnencodableValue`，却污染了所有框架失败。
- 解码路径永远不会产生不可编码值，却必须把 `FailureValue = ()` 编进类型。
- 转换路径的真实错误方向是 decode / encode，当前却先包进 `ConvertError`，
  再塞进 `TranscodeError`。
- `TranscodeConvertEngine` 需要把解码错误的 `()` 映射成目标值类型，因而出现
  不可达分支。

因此重构的核心不是少写几行代码，而是把“不可能发生”表达进类型结构。

## 新错误分层

### `TranscodeFailure`

`TranscodeFailure` 改为无泛型，只表示 transcode 框架自身能判断的结构性失败。

```rust
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
#[non_exhaustive]
pub enum TranscodeFailure {
    #[error("invalid input index {index} for input length {input_len}")]
    InvalidInputIndex { index: usize, input_len: usize },

    #[error("invalid output index {index} for output length {output_len}")]
    InvalidOutputIndex { index: usize, output_len: usize },

    #[error(
        "invalid output range at index {output_index} with length {range_len} for output length {output_len}"
    )]
    InvalidOutputRange {
        output_index: usize,
        range_len: usize,
        output_len: usize,
    },

    #[error(
        "insufficient output at index {output_index}: required {required} units, available {available}"
    )]
    InsufficientOutput {
        output_index: usize,
        required: usize,
        available: usize,
    },

    #[error("output length arithmetic overflow")]
    OutputLengthOverflow,

    #[error(
        "incomplete input at index {input_index}: required {required} units, available {available}"
    )]
    IncompleteInput {
        input_index: usize,
        required: usize,
        available: usize,
    },

    #[error("trailing input after value: consumed {consumed} units, remaining {remaining}")]
    TrailingInput { consumed: usize, remaining: usize },

    #[error("transcode called after finish without an intervening reset")]
    TranscodeAfterFinish,

    #[error("finish called twice without an intervening reset")]
    FinishAfterFinish,

    #[error("transcoder lifecycle is poisoned; a successful reset is required")]
    LifecyclePoisoned,
}
```

`TranscodeFailure` 是允许继续增加框架失败类别的开放错误集合，因此使用
`#[non_exhaustive]`。三个方向错误包装仍是封闭分类；新增框架失败继续通过其
`Failure` 变体承载，不需要同步扩张包装枚举。

所有索引、容量、完整输入检查都归到这个类型上：

```rust
impl TranscodeFailure {
    pub const fn invalid_input_index(index: usize, len: usize) -> Self;
    pub const fn invalid_output_index(index: usize, len: usize) -> Self;
    pub const fn insufficient_output(
        output_index: usize,
        required: usize,
        available: usize,
    ) -> Self;
    pub const fn output_length_overflow() -> Self;
    pub const fn incomplete_input(
        input_index: usize,
        required: usize,
        available: usize,
    ) -> Self;
    pub const fn trailing_input(consumed: usize, remaining: usize) -> Self;

    pub fn ensure_input_index(input_len: usize, input_index: usize) -> Result<(), Self>;
    pub fn ensure_min_input(
        input_len: usize,
        input_index: usize,
        min_required: usize,
    ) -> Result<(), Self>;
    pub fn ensure_no_trailing_input(consumed: usize, total: usize) -> Result<(), Self>;
    pub fn ensure_output_index(output_len: usize, output_index: usize) -> Result<(), Self>;
    pub fn ensure_transcode_indices(
        input_len: usize,
        input_index: usize,
        output_len: usize,
        output_index: usize,
    ) -> Result<(), Self>;
    pub fn ensure_output_capacity(
        output_len: usize,
        output_index: usize,
        required: usize,
    ) -> Result<(), Self>;
    pub fn ensure_output_range(
        output_len: usize,
        output_index: usize,
        range_len: usize,
        required: usize,
    ) -> Result<(), Self>;
}

impl From<CapacityError> for TranscodeFailure {
    fn from(error: CapacityError) -> Self;
}
```

这样三个高层错误类型不再复制 `ensure_*` 工具方法，只通过
`From<TranscodeFailure>` 接收框架失败。

### `TranscodeDecodeError<E>`

解码错误只包含框架失败和解码域错误。

```rust
#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
pub enum TranscodeDecodeError<E> {
    #[error(transparent)]
    Failure(#[from] TranscodeFailure),

    #[error(transparent)]
    Domain(#[from] TranscodeDomainError<E>),
}
```

保留面向域错误的便利构造和映射：

```rust
impl<E> TranscodeDecodeError<E> {
    pub const fn domain_reset(source: E) -> Self;
    pub const fn domain_main(source: E, input_index: usize) -> Self;
    pub const fn domain_main_with_consumed(
        source: E,
        input_index: usize,
        input_consumed: Option<NonZeroUsize>,
    ) -> Self;
    pub const fn domain_finish(source: E) -> Self;

    pub fn from_decode_failure(
        failure: DecodeFailure<E>,
        input_index: usize,
        available: usize,
    ) -> Self;

    pub const fn is_domain(&self) -> bool;
    pub const fn failure_ref(&self) -> Option<&TranscodeFailure>;
    pub const fn domain_error_ref(&self) -> Option<&TranscodeDomainError<E>>;
    pub const fn domain_ref(&self) -> Option<&E>;
    pub fn map_domain<T, F>(self, f: F) -> TranscodeDecodeError<T>
    where
        F: FnOnce(E) -> T;
}
```

### `TranscodeEncodeError<E, V>`

编码错误把不可编码值提升为平级变体。

```rust
#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
pub enum TranscodeEncodeError<E, V> {
    #[error(transparent)]
    Failure(#[from] TranscodeFailure),

    #[error("unencodable value at input index {input_index}")]
    Unencodable {
        input_index: usize,
        value: Option<V>,
    },

    #[error(transparent)]
    Domain(#[from] TranscodeDomainError<E>),
}
```

`Unencodable` 不再是框架失败。它表示编码域边界：输入值不是目标 codec 可表示
的值，但这不是索引、容量或进度契约错误。

```rust
impl<E, V> TranscodeEncodeError<E, V> {
    pub const fn domain_reset(source: E) -> Self;
    pub const fn domain_main(source: E, input_index: usize) -> Self;
    pub const fn domain_finish(source: E) -> Self;

    pub fn unencodable(input_index: usize, value: V) -> Self;
    pub const fn unencodable_without_context(input_index: usize) -> Self;

    pub const fn failure_ref(&self) -> Option<&TranscodeFailure>;
    pub const fn domain_error_ref(&self) -> Option<&TranscodeDomainError<E>>;
    pub const fn domain_ref(&self) -> Option<&E>;
    pub const fn unencodable_ref(&self) -> Option<(usize, Option<&V>)>;

    pub fn map_domain<T, F>(self, f: F) -> TranscodeEncodeError<T, V>
    where
        F: FnOnce(E) -> T;

    pub fn map_value<W, F>(self, f: F) -> TranscodeEncodeError<E, W>
    where
        F: FnOnce(V) -> W;
}
```

### `TranscodeConvertError<DE, EE, V>`

转换错误直接表达四类结果：

```rust
#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
pub enum TranscodeConvertError<DE, EE, V> {
    #[error(transparent)]
    Failure(#[from] TranscodeFailure),

    #[error("decode side failed: {0}")]
    DecodeDomain(#[source] TranscodeDomainError<DE>),

    #[error("encode side failed: {0}")]
    EncodeDomain(#[source] TranscodeDomainError<EE>),

    #[error("unencodable value at input index {input_index}")]
    Unencodable {
        input_index: usize,
        value: Option<V>,
    },
}
```

这里不要使用 `#[error(transparent)]` 包装 decode / encode 域错误，否则 Display
会丢失错误方向。方向是 converter 错误最重要的信息之一。

`ConvertError<D, E>` 从 core transcode API 中删除。需要方向标记时，直接使用
`TranscodeConvertError` 的 `DecodeDomain` / `EncodeDomain`；需要业务级转换错误
时，在上层 crate 定义自己的错误枚举。

## Codec 便利别名

三类新错误类型的泛型参数是域错误类型，不是 codec 类型。为了让 codec-backed
engine 的签名保持清楚，提供显式命名的 codec 便利别名：

```rust
pub type TranscodeDecodeErrorOf<C> =
    TranscodeDecodeError<<C as Codec>::DecodeError>;

pub type TranscodeEncodeErrorOf<C> =
    TranscodeEncodeError<<C as Codec>::EncodeError, <C as Codec>::Value>;

pub type TranscodeConvertErrorOf<D, E> = TranscodeConvertError<
    <D as Codec>::DecodeError,
    <E as Codec>::EncodeError,
    <D as Codec>::Value,
>;

pub type DecodeInvalidActionOf<C> =
    DecodeInvalidAction<<C as Codec>::Value>;

pub type EncodeUnencodableActionOf<C> =
    EncodeUnencodableAction<<C as Codec>::Value>;
```

不要继续使用 `TranscodeDecodeError<C>` 这种“泛型看起来是 codec，实际可能是域
错误”的别名；这会把旧设计的歧义带进新 API。

核心错误类型不采用 `TranscodeEncodeError<EC>` /
`TranscodeDecodeError<DC>` / `TranscodeConvertError<DC, EC>` 这种 carrier
泛型。错误类型应按自己实际携带的数据参数化，而不是按产生错误的对象参数化：

- `DE`、`EE`、`V` 才是错误 payload；codec、engine、facade 只是错误来源。
- 两个不同 carrier 如果拥有相同 domain error 和 value，不应因为来源类型不同而
  变成无法互换的错误类型。
- `map_domain` 在 payload 泛型上是自然操作；carrier 泛型则需要额外的 carrier
  trait 或新 wrapper 类型来表达映射后的 domain error。
- converter 需要表达 decode error、encode error 和中间 value 三个事实。用
  carrier 投影会把关系藏在关联类型约束里，错误类型本身反而不直观。

因此设计原则是：公开核心错误按 payload 参数化，codec-backed 常见场景用清楚命名
的 alias 补 ergonomics。

## `From` 桥

转换错误通过 `From` 接收解码和编码错误：

```rust
impl<DE, EE, V> From<TranscodeDecodeError<DE>>
    for TranscodeConvertError<DE, EE, V>
{
    fn from(error: TranscodeDecodeError<DE>) -> Self {
        match error {
            TranscodeDecodeError::Failure(failure) => Self::Failure(failure),
            TranscodeDecodeError::Domain(error) => Self::DecodeDomain(error),
        }
    }
}

impl<DE, EE, V> From<TranscodeEncodeError<EE, V>>
    for TranscodeConvertError<DE, EE, V>
{
    fn from(error: TranscodeEncodeError<EE, V>) -> Self {
        match error {
            TranscodeEncodeError::Failure(failure) => Self::Failure(failure),
            TranscodeEncodeError::Domain(error) => Self::EncodeDomain(error),
            TranscodeEncodeError::Unencodable { input_index, value } => {
                Self::Unencodable { input_index, value }
            }
        }
    }
}
```

这两个 impl 是消除 `unreachable!` 的核心。解码错误进入转换错误时没有值上下文
要映射；编码错误进入转换错误时值类型相同。

另外三个高层错误类型都实现：

```rust
impl<E> From<CapacityError> for TranscodeDecodeError<E>;
impl<E, V> From<CapacityError> for TranscodeEncodeError<E, V>;
impl<DE, EE, V> From<CapacityError> for TranscodeConvertError<DE, EE, V>;
```

这些 impl 只是便利入口，内部统一转成
`TranscodeFailure::OutputLengthOverflow`。

## `Transcoder` trait

`Transcoder` 只暴露一个错误关联类型：

```rust
pub trait Transcoder {
    type Input;
    type Output;
    type Error: From<TranscodeFailure>;

    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError>;

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn max_total_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        let reset = self.max_reset_output_len()?;
        let transcode = self.max_transcode_output_len(input_len)?;
        let finish = self.max_finish_output_len()?;
        reset
            .checked_add(transcode)
            .and_then(|len| len.checked_add(finish))
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    fn reset(
        &mut self,
        output: &mut [Self::Output],
        output_index: usize,
    ) -> Result<usize, Self::Error>;

    fn transcode(
        &mut self,
        input: &[Self::Input],
        input_index: usize,
        output: &mut [Self::Output],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error>;

    fn finish(
        &mut self,
        output: &mut [Self::Output],
        output_index: usize,
    ) -> Result<usize, Self::Error>;

    fn transcode_complete_into(
        &mut self,
        input: &[Self::Input],
        output: &mut [Self::Output],
    ) -> Result<usize, Self::Error> {
        let mut output_cursor = self.reset(output, 0)?;
        let transcode_required = self
            .max_transcode_output_len(input.len())
            .map_err(TranscodeFailure::from)?;
        let finish_required = self
            .max_finish_output_len()
            .map_err(TranscodeFailure::from)?;
        let remaining_required = transcode_required
            .checked_add(finish_required)
            .ok_or_else(TranscodeFailure::output_length_overflow)?;

        TranscodeFailure::ensure_output_capacity(
            output.len(),
            output_cursor,
            remaining_required,
        )?;

        let progress = self.transcode(input, 0, output, output_cursor)?;
        if progress.is_complete() && progress.read() < input.len() {
            return Err(TranscodeFailure::trailing_input(
                progress.read(),
                input.len() - progress.read(),
            )
            .into());
        }

        debug_assert!(
            progress
                .validate(
                    0,
                    input.len(),
                    output_cursor,
                    output.len().saturating_sub(output_cursor),
                )
                .is_ok(),
            "Transcoder::transcode returned invalid progress",
        );

        output_cursor += progress.written();
        match progress.status() {
            TranscodeStatus::Complete => {}
            TranscodeStatus::NeedOutput {
                output_index,
                required,
                available,
            } => {
                return Err(TranscodeFailure::insufficient_output(
                    output_index,
                    required.get(),
                    available,
                )
                .into());
            }
            TranscodeStatus::NeedInput {
                input_index,
                required,
                available,
            } => {
                return Err(TranscodeFailure::incomplete_input(
                    input_index,
                    required.get(),
                    available,
                )
                .into());
            }
        }

        output_cursor += self.finish(output, output_cursor)?;
        Ok(output_cursor)
    }
}
```

`type Error: From<TranscodeFailure>` 是唯一必要约束。`CapacityError` 是 capacity
planning API 的错误，不是 transcode runtime 错误；默认实现显式把它转成
`TranscodeFailure`，避免要求自定义 transcoder 同时实现两个 `From`。

`TranscodeErrorOf<T>` 删除。泛型代码直接写 `T::Error`：

```rust
D: TranscodeDecoder<Input = u8, Output = char>,
D::Error: StdError + Send + Sync + 'static,
```

role traits 不再保持为空。它们具体化方向语义，并约束 `Transcoder::Error` 的
标准形态：

```rust
pub trait TranscodeDecoder:
    Transcoder<Error = TranscodeDecodeError<Self::DecodeError>>
{
    type DecodeError;
}

pub trait TranscodeEncoder:
    Transcoder<
        Error = TranscodeEncodeError<
            Self::EncodeError,
            <Self as Transcoder>::Input,
        >,
    >
{
    type EncodeError;
}

pub trait TranscodeConverter:
    Transcoder<
        Error = TranscodeConvertError<
            Self::DecodeError,
            Self::EncodeError,
            Self::Value,
        >,
    >
{
    type DecodeError;
    type EncodeError;
    type Value;
}
```

这使 `Transcoder` 继续作为统一驱动协议存在，而三个 role traits 明确声明各自的
错误族。泛型 I/O、测试辅助和 one-shot 默认实现可以依赖 `Transcoder`；需要知道
错误方向的 API 依赖具体 role trait。

`TranscodeEncoder` 的 `Value` 不单独声明，直接使用 `<Self as Transcoder>::Input`。
对 encoder 来说，输入就是待编码值。`TranscodeConverter` 则需要单独声明 `Value`，
因为转换的输入是源端单位，输出是目标端单位，不等于中间逻辑值。

## 是否保留 `Transcoder`

保留 `Transcoder`，但只保留它真实表达的共同结构：

- 都是 `input[] -> output[]` 的流式转换。
- 都遵循 `reset -> transcode* -> finish` 生命周期。
- 都需要 capacity planning。
- 都返回 `TranscodeProgress` / `TranscodeStatus`。
- 都能复用 `transcode_complete_into` 默认驱动逻辑。
- I/O adapter 和测试辅助经常只关心“一个流式转换器”，并不关心它是 decode、
  encode 还是 convert。

因此 `Transcoder` 不是旧设计中的坏抽象。坏抽象是把 `DomainError` /
`FailureValue` 放进 `Transcoder`，让机械协议承担错误族拼装职责。

不删除 `Transcoder` 的原因是：如果只保留 `TranscodeDecoder` /
`TranscodeEncoder` / `TranscodeConverter`，三者会复制同一套 reset、capacity、
progress 和 complete-run 方法，或者被迫引入隐藏 helper / macro。那只是把真实
共同结构从公开 trait 藏进实现细节，API 并不会更清晰。

## 引擎 API

### Decode engine

`TranscodeDecodeEngine<C, H>` 使用 codec 便利别名：

```rust
impl<C, H> Transcoder for TranscodeDecodeEngine<C, H>
where
    C: Codec,
    H: TranscodeDecodeHooks<C>,
{
    type Input = C::Unit;
    type Output = C::Value;
    type Error = TranscodeDecodeErrorOf<C>;
}
```

公开方法返回：

```rust
Result<usize, TranscodeDecodeErrorOf<C>>
Result<TranscodeProgress, TranscodeDecodeErrorOf<C>>
```

内部把当前 `TranscodeError::domain_*` 调用替换成
`TranscodeDecodeError::domain_*`，把 `TranscodeError::ensure_*` 替换成
`TranscodeFailure::ensure_*`。

### Encode engine

`TranscodeEncodeEngine<C, H>`：

```rust
impl<C, H> Transcoder for TranscodeEncodeEngine<C, H>
where
    C: Codec,
    H: TranscodeEncodeHooks<C>,
{
    type Input = C::Value;
    type Output = C::Unit;
    type Error = TranscodeEncodeErrorOf<C>;
}
```

`EncodeUnencodableAction::Reject` 返回：

```rust
Err(TranscodeEncodeError::unencodable_without_context(
    context.input_index(),
))
```

streaming encode engine 不要求 `C::Value: Clone`，因此默认只携带 index。需要值上下文
的上层 one-shot API 可以在有所有权或可 clone 时构造 `Unencodable { value: Some(_) }`。

### Convert engine

`TranscodeConvertEngine<D, E, DH, EH>`：

```rust
impl<D, E, DH, EH> Transcoder for TranscodeConvertEngine<D, E, DH, EH>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    D::Value: Clone + Default,
    DH: TranscodeDecodeHooks<D>,
    EH: TranscodeEncodeHooks<E>,
{
    type Input = D::Unit;
    type Output = E::Unit;
    type Error = TranscodeConvertErrorOf<D, E>;
}
```

解码错误通过 `?` 自然进入转换错误：

```rust
let (outcome, pending) = self.decode_engine.decode_one(
    state.input(),
    state.decode_context(),
    PendingValue::new,
)?;
```

编码错误也可以通过 `?` 传播：

```rust
let outcome = self.encode_engine.encode_one(context)?;
```

如果要进一步改善错误上下文，可以在 converter 的 `encode_pending` 中对
`Unencodable { value: None }` 做一次有所有权的增强：pending value 在错误路径上
不需要重新放回 pending slot，因此可以移动到 `Some(value)`。这不是消除旧
`unreachable!` 的必要条件，但它会让 converter 的错误比裸 encode engine 更有信息量。
建议实现一个专门方法，避免污染通用 `From`：

```rust
impl<DE, EE, V> TranscodeConvertError<DE, EE, V> {
    pub fn from_encode_error_with_value(
        error: TranscodeEncodeError<EE, V>,
        fallback_value: V,
    ) -> Self {
        match error {
            TranscodeEncodeError::Unencodable {
                input_index,
                value: None,
            } => Self::Unencodable {
                input_index,
                value: Some(fallback_value),
            },
            other => Self::from(other),
        }
    }
}
```

如果这会让 hot path 代码变得别扭，可以先只使用 `From` 桥；不要为了补充值上下文
重新引入复杂映射。

### Hook traits

decode / encode hook trait 仍然绑定 codec 类型，但返回 codec 便利错误别名：

```rust
pub trait TranscodeDecodeHooks<C>
where
    C: Codec,
{
    fn handle_invalid_decode(
        &mut self,
        codec: &mut C,
        error: &C::DecodeError,
        consumed: Option<NonZeroUsize>,
        context: DecodeContext,
    ) -> Result<DecodeInvalidAction<C::Value>, TranscodeDecodeErrorOf<C>>;

    fn finish_hooks(
        &mut self,
        codec: &mut C,
        output: &mut [C::Value],
        output_index: usize,
    ) -> Result<usize, TranscodeDecodeErrorOf<C>>;
}

pub trait TranscodeEncodeHooks<C>
where
    C: Codec,
{
    fn handle_unencodable_encode(
        &mut self,
        codec: &mut C,
        context: &EncodeContext<'_, C::Value, C::Unit>,
    ) -> Result<EncodeUnencodableAction<C::Value>, TranscodeEncodeErrorOf<C>>;

    fn finish_hooks(
        &mut self,
        codec: &mut C,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<usize, TranscodeEncodeErrorOf<C>>;
}
```

hook 层不引入新的错误抽象。它只是 engine 策略扩展点，因此直接使用对应 engine
的错误类型最清楚。

## Value 层

一值转换 API 也使用新错误类型：

```rust
CodecValueDecoder<C>::decode(
    &mut self,
    input: &[C::Unit],
) -> Result<C::Value, TranscodeDecodeErrorOf<C>>;

CodecValueEncoder<C>::encode_into(
    &mut self,
    input: &C::Value,
    output: &mut Vec<C::Unit>,
) -> Result<usize, TranscodeEncodeErrorOf<C>>;
```

内部 helper 对应调整：

```rust
complete_encode_len<C>(
    codec: &C,
    value: &C::Value,
) -> Result<usize, TranscodeEncodeErrorOf<C>>
where
    C::Value: Clone;

encode_complete_value_into_reserved<C>(
    codec: &mut C,
    value: &C::Value,
    output: &mut [C::Unit],
    output_index: usize,
    required: usize,
) -> Result<usize, TranscodeEncodeErrorOf<C>>;

decode_exact_complete_value<C>(
    codec: &mut C,
    input: &[C::Unit],
    scratch: &mut [C::Value],
) -> Result<C::Value, TranscodeDecodeErrorOf<C>>;
```

`complete_encode_len` 遇到不可编码值时返回：

```rust
Err(TranscodeEncodeError::unencodable(0, value.clone()))
```

这比 streaming engine 更完整，因为 value 层已经要求 `C::Value: Clone`。

## I/O 适配层

泛型 streaming I/O 适配器不再依赖 `TranscodeErrorOf<T>`：

```rust
pub fn transcode_into<D, M, Value>(
    &mut self,
    decoder: &mut D,
    map_error: &mut M,
    output: &mut [Value],
    output_index: usize,
    count: usize,
) -> Result<usize>
where
    D: Transcoder<Input = I::Item, Output = Value>,
    M: FnMut(D::Error) -> Error;

pub fn transcode_from<E, M, Value>(
    &mut self,
    encoder: &mut E,
    map_error: &mut M,
    input: &[Value],
    input_index: usize,
    count: usize,
) -> Result<usize>
where
    E: Transcoder<Input = Value, Output = O::Item>,
    M: FnMut(E::Error) -> Error;
```

一值 encode output 的错误映射改成匹配 `TranscodeEncodeError`：

```rust
match error {
    TranscodeEncodeError::Domain(error) => map_error(error.into_source()),
    TranscodeEncodeError::Unencodable { .. } => {
        Error::new(ErrorKind::InvalidInput, "codec cannot encode value")
    }
    TranscodeEncodeError::Failure(failure) => {
        Error::new(ErrorKind::InvalidData, failure.to_string())
    }
}
```

## Text crate 映射

`rs-codec-text` 的 transcode wrapper 直接声明新错误：

```rust
impl<C> Transcoder for CharsetDecoder<C>
where
    C: CharsetCodec,
{
    type Input = C::Unit;
    type Output = char;
    type Error = TranscodeDecodeError<CharsetDecodeError>;
}

impl<C> Transcoder for CharsetEncoder<C>
where
    C: CharsetCodec,
{
    type Input = char;
    type Output = C::Unit;
    type Error = TranscodeEncodeError<CharsetEncodeError, char>;
}

impl<D, E> Transcoder for CharsetConverter<D, E>
where
    D: CharsetCodec,
    E: CharsetCodec,
{
    type Input = D::Unit;
    type Output = E::Unit;
    type Error = TranscodeConvertError<
        CharsetDecodeError,
        CharsetEncodeError,
        char,
    >;
}
```

`CharsetEncodeError::map_transcode_failure` 拆成两个入口：

```rust
pub fn map_transcode_failure(
    charset: Charset,
    failure: TranscodeFailure,
) -> Self;

pub fn map_unencodable(
    charset: Charset,
    input_index: usize,
    value: Option<char>,
) -> Self;
```

这样 `TranscodeFailure` 不再知道 `char`，不可编码值由 encode/convert 错误自己携带。

`CharsetConvertError` 可以作为 text crate 的业务层错误继续存在，但它不再依赖 core
的 `ConvertError`。从 core convert 错误映射时：

```rust
match error {
    TranscodeConvertError::Failure(failure) => {
        CharsetConvertError::Encode(CharsetEncodeError::map_transcode_failure(
            target_charset,
            failure,
        ))
    }
    TranscodeConvertError::DecodeDomain(error) => {
        CharsetConvertError::Decode(error.into_source())
    }
    TranscodeConvertError::EncodeDomain(error) => {
        CharsetConvertError::Encode(error.into_source())
    }
    TranscodeConvertError::Unencodable { input_index, value } => {
        CharsetConvertError::Encode(CharsetEncodeError::map_unencodable(
            target_charset,
            input_index,
            value,
        ))
    }
}
```

## 删除项

直接删除以下公开 API：

- `TranscodeError<E, Value>`
- `TranscodeFailure<Value>`
- `Transcoder::DomainError`
- `Transcoder::FailureValue`
- `TranscodeErrorOf<T>`
- `ConvertError<D, E>`
- 旧 `TranscodeDecodeError<C>` / `TranscodeEncodeError<C>` codec 型别名

不提供 deprecated alias。这个版本的目的就是让 API 表达正确边界。

## 文件布局

保持 `rs-codec/src/transcode/` 当前扁平风格，新增按类型命名的文件：

- `transcode_failure.rs`
- `transcode_decode_error.rs`
- `transcode_encode_error.rs`
- `transcode_convert_error.rs`
- `transcode_domain_error.rs`
- `transcoder.rs`

删除：

- `transcode_error.rs`
- `convert_error.rs`

`transcode/mod.rs` 和 `lib.rs` 只重导出新类型与 codec 便利别名。

## 实施顺序

虽然不以工作量为约束，但实施仍应按依赖方向推进，避免一次性把所有编译错误混在一起：

1. 改 `TranscodeFailure`，移除泛型和 `UnencodableValue`，迁移所有 `ensure_*`。
2. 新增三类路径错误和 codec 便利别名。
3. 改 `Transcoder`，用 `type Error` 替换 `DomainError` / `FailureValue`。
4. 改 value 层 helper 和 value adapter。
5. 改 decode / encode / convert engine。
6. 删除 `TranscodeError` 和 `ConvertError`。
7. 改 `rs-codec-text` 的 decoder / encoder / converter。
8. 改 `rs-codec` I/O 适配器和 `rs-io-text` 约束。
9. 改 tests 和文档。

## 测试策略

测试应围绕新类型边界，而不是旧 API 的迁移痕迹：

- `TranscodeFailure` 构造、Display、`ensure_*` 边界。
- `TranscodeDecodeError` 的 domain 构造、`from_decode_failure` 和 `map_domain`。
- `TranscodeEncodeError` 的 `Unencodable`、`map_domain`、`map_value`。
- `TranscodeConvertError` 的 decode / encode 方向 Display、`From` 桥和 `?` 传播。
- `Transcoder::transcode_complete_into` 对 incomplete、trailing、insufficient output 的默认错误。
- decode engine、encode engine、convert engine 的关键行为回归。
- value-level encoder/decoder 对不可编码、尾随输入、容量溢出的错误分类。
- text crate 的 malformed / unmappable 策略映射。
- I/O adapter 的 `D::Error` / `E::Error` bound 编译路径。

需要删除所有断言旧 `UnencodableValue`、`ConvertError`、`TranscodeErrorOf` 的测试。

## 最终 API 示例

解码：

```rust
pub fn transcode(
    &mut self,
    input: &[C::Unit],
    input_index: usize,
    output: &mut [C::Value],
    output_index: usize,
) -> Result<TranscodeProgress, TranscodeDecodeErrorOf<C>>;
```

编码：

```rust
pub fn transcode(
    &mut self,
    input: &[C::Value],
    input_index: usize,
    output: &mut [C::Unit],
    output_index: usize,
) -> Result<TranscodeProgress, TranscodeEncodeErrorOf<C>>;
```

转换：

```rust
pub fn transcode(
    &mut self,
    input: &[D::Unit],
    input_index: usize,
    output: &mut [E::Unit],
    output_index: usize,
) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>>;
```

泛型 I/O：

```rust
where
    D: TranscodeDecoder<Input = u8, Output = char>,
    D::Error: StdError + Send + Sync + 'static,
```

错误处理：

```rust
match error {
    TranscodeConvertError::Failure(failure) => handle_framework(failure),
    TranscodeConvertError::DecodeDomain(error) => handle_decode(error),
    TranscodeConvertError::EncodeDomain(error) => handle_encode(error),
    TranscodeConvertError::Unencodable { input_index, value } => {
        handle_unencodable(input_index, value)
    }
}
```

## 设计取舍

保留 `TranscodeDomainError<E>`，因为 reset / main / finish 是真实语义，不是旧结构
造成的噪音。

删除 `TranscodeError<E, V>`，因为它试图同时服务三条路径，最终只能靠 phantom 泛型
和不可达分支维持。

删除 `ConvertError<D, E>`，因为转换方向已经是 `TranscodeConvertError` 的一等信息。
再保留一个中间 enum 只会让错误层级回到旧形态。

不让 `Transcoder` 重新暴露 domain error 类型。自定义 transcoder 的错误可以是三类
标准错误之一，也可以是业务自定义错误；trait 只要求它能接收框架失败。

## 一句话结论

新的模型把“框架失败”“域错误”“不可编码值”“转换方向”拆成互不污染的类型边界。
`Transcoder` 只描述输入、输出和完整错误类型；decode / encode / convert 各自拥有
符合自身语义的错误枚举。这样 API 更窄、更直白，也更少需要用代码解释类型系统
本应表达的事实。
