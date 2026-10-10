---
schema_version: 4
slug: 2026-10-09T1545-eko-context-compression/eko-compression-integration
design_ref: null
delivery_ref: docs/supreme/plans/2026-10-09T1545-eko-context-compression/delivery-map.md#eko-compression-integration
outcome:
  summary: echo-agent-cli 将 EKO 的 active goal、recovery capsule、recent user
    constraints、手动压缩和各 surface 入口统一接入框架压缩契约
  acceptance:
    - TaskRuntimeContextProjector 继续是 EKO goal/recovery/recent-constraint
      的唯一产品权威；压缩刷新后每个 projection marker 至多保留一份，作用域退出会删除过期 capsule。
    - CLI、TUI、GUI、channel 的手动压缩都通过同一个 app-core admission/journal service，并且
      focus、cancel、checkpoint 和 terminal event 的行为一致且文档可解释；移除未生效的 keep_messages
      覆盖，统一使用已安装策略。
    - EKO 配置、模型窗口、Visibility Horizon、protected token、recovery-active 和
      compression stats 的显示口径与框架最终预算一致。
    - echo-agent-cli 的 app-core 定向测试、Rust/前端相关门禁和中英文配置文档在最终差异后通过。
out_of_scope:
  - 不修改 echo-agent 的通用压缩实现或在 CLI 复制 ContextManager/ContextCompressor。
  - 不把完整用户 transcript 永久注入 active context。
  - 不启用 SQLite，也不新增 EKO 专属第二套 goal/compression store。
  - 不在本 Outcome 中处理与压缩无关的 SDK、插件、LSP、Side Conversation 或 worktree 清理。
todos:
  - id: align-eko-compression-policy
    summary: 把 EKO 的模型窗口、Visibility Horizon、compress_strategy/compress_window 和
      protected TaskRuntime projection 组合成单一可解释的产品策略，并明确手动压缩参数与框架 recent-tail
      契约的映射。
    files:
      - echo-agent-app-core/src/config.rs
      - echo-agent-app-core/src/infra/factory.rs
      - echo-agent-app-core/src/tasks/task_runtime/compact_context.rs
      - config/eko.example.yaml
      - docs/zh/configuration.md
      - docs/en/configuration.md
    acceptance:
      - 默认 summary、模型窗口和 Visibility Horizon 参数有明确预算关系；TaskRuntime
        goal/recovery/recent constraints 仍由 projection marker 保留，不新增平行状态。
      - "`compress_window`、manual focus 和 token budget 的兼容/优先级规则写入中英文配置文档和示例，并有
        app-core 定向测试。"
  - id: unify-manual-compression-surfaces
    summary: 统一 CLI/TUI/GUI/channel 的 manual compression
      request、focus/cancel、已安装窗口策略、journal safe point、checkpoint receipt 和
      UI/channel 事件投影。
    files:
      - echo-agent-app-core/src/manual_compression.rs
      - src/cli/cmd_impls/context.rs
      - src/cli/channels.rs
      - src/tui/events.rs
      - src/tauri/commands/panels.rs
      - web-frontend/src/api/endpoints.ts
      - web-frontend/src/components/compress/CompressPanel.tsx
    acceptance:
      - 四类 surface 调用同一个 app-core service；成功、取消、Agent identity mismatch、journal
        failure 和无活动会话都有一致 typed outcome。
      - GUI/TUI/CLI/channel 的 before/after messages、tokens saved、checkpoint
        strategy、protected count 和 compression event 可互相对账。
      - 相关 app-core 测试、Rust workspace focused checks、前端 prettier/test/build
        在最后一次改动后通过。
  - id: document-and-verify-eko-boundary
    summary: 补齐 EKO 压缩边界、TaskRuntime projection、持久化权威和 observability 的产品文档与语义验证材料。
    files:
      - docs/zh/features.md
      - docs/en/features.md
      - docs/zh/architecture/persistence.md
      - docs/en/architecture/runtime.md
      - echo-agent-app-core/src/tasks/task_runtime/compact_context.rs
    acceptance:
      - 文档明确 ContextManager active window、ConversationStore
        transcript、RuntimeStateStore checkpoint、TaskRuntimeStore goal/recovery 和
        Trace/ChatEventLog 的不同权威。
      - 测试证明 projection refresh 不累积重复 goal/capsule，recent constraints 在 journal
        rebuild 后仍可恢复，manual compression safe point 只提交一次。
      - 完成前通过 echo-agent-cli 的 semantic-verify 以及与改动匹配的 Rust/前端门禁。
artifact_id: plan:1639eecd-b424-45e1-b2c5-6b5e2434201a
lifecycle: completed
design_sections: []
---
## Notes
基线：echo-agent-cli origin/main = 40fd8044，当前代码已有 TaskRuntime goal/recovery projection、recent constraints 和跨 surface manual_compression service。此 Outcome 只收口 EKO 产品策略与入口契约；框架 recent-tail 契约完成后才能开始实现依赖该契约的映射和验证。
