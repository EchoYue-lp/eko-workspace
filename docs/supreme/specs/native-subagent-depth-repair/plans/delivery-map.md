---
schema_version: 1
artifact: delivery-map
design_ref: docs/supreme/specs/native-subagent-depth-repair/design.md
outcomes:
  run-attempt-isolation:
    ships: 同一主 Agent 中不同 TaskRun 可复用 task/attempt 并独立执行，下一次尝试指导按同一 scope/Run 隔离且只消费一次。
    depends_on: []
  readonly-depth-enforcement:
    ships: 真实内置只读 reviewer 按运行绑定恰达 1／2／3 层，所有后代保留实际只读与父工具上界，并完成同 conversation 双 Run
      的新原生验收。
    depends_on:
      - run-attempt-isolation
design_revision: sha256:773d16c9ed2f04eef90b6fe45cd11499ab5f4caf0beb41794d3f5b3b58ccf81c
---
