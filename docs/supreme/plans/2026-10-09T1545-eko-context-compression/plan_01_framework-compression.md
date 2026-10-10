---
schema_version: 4
slug: 2026-10-09T1545-eko-context-compression/framework-compression
design_ref: null
delivery_ref: docs/supreme/plans/2026-10-09T1545-eko-context-compression/delivery-map.md#framework-compression
outcome:
  summary: echo-agent 提供按 token budget 选择最近用户 turn 的通用上下文压缩能力，并在自动 compact 中保持
    tool 组原子性与 active request 保真
  acceptance:
    - Summary、IncrementalSummary 和 SlidingWindow 提供共享的 token-budgeted turn 选择；较旧
      turn 整轮保留，过大当前 turn 保留完整用户请求和近期原子工具组，单个请求超过硬预算明确报错。原 Rust 构造器保留，配置
      compress_window=0 启用 token 策略。
    - assistant tool_calls 与对应 tool results
      始终成组保留或成组折叠；CanonicalContext、projection 和 CompressionCheckpoint 仍由现有
      ContextManager 负责。
    - 自动 compact、手动 force-compress 和摘要失败 fallback 共享同一尾部选择与预算规则；压缩前后
      token、驱逐消息、保护消息和验证结果可审计。
    - 新增行为由框架压缩测试、文档和现有 workspace 门禁共同证明。
out_of_scope:
  - 不把 EKO TaskRuntime goal、recovery capsule、workspace policy 或 GUI/TUI/channel
    逻辑下沉到框架。
  - 不新增第二个 transcript、memory、goal 或 compression-state store。
  - 不在本 Outcome 中修改 echo-agent-cli。
todos:
  - id: define-recent-tail-contract
    summary: 在现有 ContextCompressor/ContextManager 语义内确定 token-budgeted、turn-aware
      recent tail 的唯一选择规则，并保留现有构造方式的兼容行为。
    files:
      - echo-state/src/compression/mod.rs
      - echo-state/src/compression/compressor/summary.rs
      - echo-state/src/compression/compressor/sliding_window.rs
      - echo-state/src/compression/compressor/hybrid.rs
    acceptance:
      - 选择规则明确区分 system/canonical context、完整 user turn、assistant tool
        call/result group 和旧历史；同一规则可被 Summary、IncrementalSummary、SlidingWindow 和
        fallback 复用。
      - 公共 API 形状、默认预算和兼容语义在实现前形成可审阅记录；不出现平行 selector 或第二套压缩权威。
  - id: implement-and-verify-framework-tail
    summary: 实现 recent tail 选择、active request 保真和压缩后校验，并补齐框架测试、文档和 example contract。
    files:
      - echo-state/src/compression/compressor/summary.rs
      - echo-state/src/compression/compressor/sliding_window.rs
      - echo-state/src/compression/invariants.rs
      - src/agent/react/run/phases/compact.rs
      - docs/zh/04-compression.md
      - docs/en/04-compression.md
      - echo-agent-learning/tests/example_contracts
    acceptance:
      - 测试覆盖长 user message、多个 user turn、tool call/result、摘要失败 fallback、重复压缩和
        token budget 边界。
      - cargo fmt、相关 echo_state/echo_agent 测试和文档/example contract 在最终差异后通过。
artifact_id: plan:404c72b9-ecb2-4798-9393-e6847e894e48
lifecycle: completed
design_sections: []
---
## Notes
基线：echo-agent main/origin/main = a8c2d1ae。当前 `SummaryCompressor` 与 `IncrementalSummaryCompressor` 在 summary.rs:384/739 仍按 `keep_recent` 消息数切分；自动 compact 的 `prepare_with_cancel` 已预留 request overhead，但 current query 仍未作为尾部选择输入。实现应优先复用 ContextManager 的 protected/canonical/projection/checkpoint/verifier，不把完整 transcript 常驻模型上下文。
