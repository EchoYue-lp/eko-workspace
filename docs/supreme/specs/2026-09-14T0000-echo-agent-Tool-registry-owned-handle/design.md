---
title: echo-agent Tool Registry Owned Handle 设计
artifact: design
carrier: markdown
---

# echo-agent Tool registry owned handle 设计

## 问题与目标

`ToolManager`当前把`Box<dyn Tool>`直接存入`DashMap`，公开`get_tool`返回`DashMap::Ref`。Non-stream和stream执行路径取得该Ref后跨参数校验、permit等待、retry、timeout和完整Tool future持有它。同步`replace`或`unregister`需要写同一shard；在Tokio current-thread runtime上，如果调用方从同一executor线程发起mutation，阻塞写会阻止旧Tool future恢复并释放Ref，形成deadlock。

目标是让registry lookup返回独立于map guard的owned Tool lifetime，使动态mutation不等待异步调用结束，同时保持一个registry authority、Tool generation fence、result cache freshness和既有Agent动态工具能力。

## 目标行为

- `ToolManager`以`Arc<dyn Tool>`作为每个已注册Tool generation的owned runtime handle。
- Lookup只在同步临界区内克隆Arc并立即释放DashMap guard；校验、permit、retry和Tool future只持有Arc，不持有registry guard。
- `replace`原子发布新Arc并返回旧generation的Arc；已领取旧Arc的调用可以按既有生命周期完成，但不能向新generation的result cache发布结果。
- `unregister`从catalog移除当前Arc并返回它；已领取的旧Arc继续由调用自身持有，不把“从catalog移除”伪装成强制取消。
- `result_cache_epoch`继续在lookup前观察，所有register/replace/unregister publication继续推进epoch并清cache。旧Arc即使晚完成，也不能成为新generation的cache authority。
- current-thread runtime中，active Tool挂起期间同步replace/unregister必须立即返回，不依赖`spawn_blocking`规避registry内部锁。

## 范围与非目标

范围包括`echo-execution::ToolManager`的registry value ownership、公开Rust返回类型、framework内调用点、动态工具示例与合同、Rust public inventory、正式ADR和语义闭环。

以下不在本设计内：

- 不改变Tool输入仍以`Box<dyn Tool>`注册的builder/registrar ergonomics；注册边界内部转换为Arc。
- 不增加第二registry、per-call shadow map或application-owned Tool catalog。
- 不改变Tool参数、permission、sandbox、stream event、retry、timeout和Plan mode语义。
- 不强制取消已领取旧generation的Tool；取消仍由现有`ToolContext.cancel`和调用生命周期负责。
- 不为Host/Rust-only registry API新增TypeScript、Python或Java facade。
- 不修复Issue #115以外的79个open Finding。

## 系统边界

该能力属于framework：任何embedding application都可能动态注册、替换或移除Tool，且并发容器、generation publication和Tool lifetime不依赖EKO产品策略。EKO/CLI只消费framework结果，不拥有第二套registry或reload状态机。

```text
register Box<dyn Tool>
        |
        v
Arc<dyn Tool> ----> DashMap<String, Arc<dyn Tool>>  (唯一catalog authority)
                           |
lookup: clone Arc ----------+----> drop map guard ----> async execute
                           |
replace/unregister ---------+----> publish/remove + epoch bump + cache clear

old Arc execution -------------------------------> may finish normally
old observed epoch != current epoch -------------> must not cache
```

## 公共合同

- `ToolManager::get_tool`从借用的`DashMap::Ref`改为`Option<Arc<dyn Tool>>`。
- `ToolManager::replace`与`ToolManager::unregister`从`Option<Box<dyn Tool>>`改为`Option<Arc<dyn Tool>>`。
- `ReactAgent`的同名固有`replace_tool`/`remove_tool`随底层返回owned Arc；`Agent::remove_tool -> bool`保持不变。
- 仅依赖`is_some`或通过Deref调用Tool方法的调用方保持源码等价；显式要求`Box<dyn Tool>`的Rust调用方需要迁移为Arc。
- 这些identity已分类为`host_or_rust_only`，必须刷新Rust public inventory和signature digest，但不展开三语言SDK facade。

## 异常与边界场景

- Concurrent lookup与replace：lookup得到old或new任一完整generation；不能得到部分构造值。旧generation完成后因epoch不匹配不能cache。
- Concurrent lookup与unregister：已取得Arc的调用可完成；后续lookup返回None。Unregister不是取消协议。
- Re-register同名Tool：仍由现有registration lock与duplicate规则裁决，不改变try-register原子性。
- Mutation期间definition cache：publication后推进definition version并让下一次读取重建；不得出现第二个版本源。
- Arc回收：旧Tool在registry和所有in-flight/外部owned handles释放后Drop；这是明确的generation lifetime，不要求同步mutation等待资源析构。
- Tool内部资源需要显式async close时，继续由Tool自身或embedding lifecycle合同负责；Arc Drop不被提升为异步cleanup receipt。

## 关键取舍与业界依据

Tokio官方教程明确指出，同步guard跨await即使类型实现Send也可能deadlock，推荐把锁限制在非async临界区。DashMap对insert/remove等mutation明确警告：持有map reference时可能deadlock。OpenAI Codex的Rust Tool registry把runtime保存为`Arc<dyn CoreToolRuntime>`，lookup克隆owned Arc，再进入异步dispatch。这三者共同支持“短临界区克隆owned handle”而不是“让异步执行持有容器guard”。

- Tokio shared state: <https://tokio.rs/tokio/tutorial/shared-state#holding-a-mutexguard-across-an-await>
- DashMap insert locking contract: <https://docs.rs/dashmap/latest/dashmap/struct.DashMap.html#method.insert>
- OpenAI Codex Tool registry: <https://github.com/openai/codex/blob/16537b20a5ec0ea9aa079f4ad4b0e30e8a9efacf/codex-rs/core/src/tools/registry.rs#L459-L463>

保留`Box`返回值的代理方案被排除：它需要手工转发完整且持续演进的Tool trait，容易让risk、stream、validation、sandbox或schema方法发生包装漂移。为执行另建Arc shadow registry也被排除，因为会产生两个Tool generation authority。Actor化registry可避免同步锁，但会把所有同步查询和builder路径改成消息协议，超过本Finding所需范围。

## 复用与实现约束

- 复用标准库`Arc`、现有DashMap、registration lock、definitions version和result cache epoch，不新增依赖或状态schema。
- Registry value直接是Arc，不新增包装trait或duplicate map。
- 任何取得DashMap Ref的helper必须在返回前克隆Arc；async函数局部变量不得携带DashMap guard。
- 不使用`unwrap`、`expect`、`panic`、`unreachable`或直接不安全索引。
- 更新正式ADR与动态工具示例；Rust inventory变化按既有generator产生，不手改digest。

## 验收标准

- 确定性current-thread测试证明active Read挂起时，同线程同步replace和unregister均可返回，随后旧调用可完成。
- Replacement/unregister后新lookup分别得到新Tool/None，旧generation不能回填result cache。
- ToolManager cache、stream、validation、retry、cancel、timeout和registration测试无回归。
- 所有framework调用点不再使用`DashMap::Ref::value()`访问Tool。
- Public inventory只出现预期的Rust/Host-only signature变化；TypeScript、Python、Java facade和wire/schema无新增映射。
- ADR、Finding #115、repair/verification/rereview证据闭合；Issue #115在本地修复阶段保持open，远端main交付后再关闭。
- 适用fmt、Clippy、workspace/feature、example、semantic strict/change-evidence门禁全部通过。
