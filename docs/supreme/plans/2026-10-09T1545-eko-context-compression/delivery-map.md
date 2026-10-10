---
schema_version: 1
artifact: delivery-map
design_ref: null
outcomes:
  framework-compression:
    ships: echo-agent 提供按 token budget 选择最近用户 turn 的通用上下文压缩能力，并在自动 compact 中保持 tool
      组原子性与 active request 保真
    depends_on: []
  eko-compression-integration:
    ships: echo-agent-cli 将 EKO 的 active goal、recovery capsule、recent user
      constraints、手动压缩和各 surface 入口统一接入框架压缩契约
    depends_on:
      - framework-compression
design_revision: null
---
