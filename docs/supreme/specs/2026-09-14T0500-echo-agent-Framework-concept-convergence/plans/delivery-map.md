---
schema_version: 1
artifact: delivery-map
design_ref: docs/supreme/specs/2026-09-14T0500-echo-agent-Framework-concept-convergence/design.md
outcomes:
  repair-readme-workspace-topology:
    ships: 双语 README 以 Cargo metadata 为权威展示完整 11-package workspace，并明确 SDK
      protocol、SDK Host 与 learning consumer 的边界
    depends_on: []
  repair-readme-feature-table:
    ships: 双语 README 的 feature 表只列 Cargo manifest 真实 feature，Task API 的 core
      可用性不再被伪装成不存在的 tasks feature
    depends_on:
      - repair-readme-workspace-topology
  repair-readme-example-target:
    ships: 双语 README 的示例命令全部指向 Cargo 真实 example 或 test contract，并由现有 documentation
      contract fail closed 校验
    depends_on:
      - repair-readme-feature-table
  publish-framework-concept-navigation:
    ships: 发布中英文 Architecture、Core Concepts 与 Lifecycles 基础文档，以唯一状态权威串联现有领域章节和可执行
      examples
    depends_on:
      - repair-readme-workspace-topology
      - repair-readme-feature-table
      - repair-readme-example-target
design_revision: sha256:66b3925591ca1967fad6cbf3331cf7fb9a6ec73c9dfe3d6ca28e02f76b8eed7e
---
该交付图把三个已有 README Finding 与概念导航发布拆成四个独立结果。三个 repair 各自只关闭其唯一 Finding/Issue 对应的本地语义状态；Issue 在提交进入远端 main 前保持 open。概念导航不批量关闭其它 open Finding，也不修改 framework runtime 或 SDK public surface。
