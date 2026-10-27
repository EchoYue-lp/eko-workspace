---
schema_version: 1
artifact: delivery-map
design_ref: docs/supreme/specs/zstd-safe-chain-repair/design.md
outcomes:
  framework-safe-chain:
    ships: Framework 保留 data/statistics 真实能力，采用两份最小 Polars 补丁和安全 zstd
      链，按消费图正确审计五项来源，并形成经验证、可精确恢复的本地修订。
    depends_on: []
  sdk-safe-chain-consumer:
    ships: SDK 以同一正式可恢复 Framework
      修订消费补丁，保持原5352项接受身份、431路由和真实Parquet/Host及语言消费者，不复制vendor或扩大风险接受。
    depends_on:
      - framework-safe-chain
design_revision: sha256:aed9f69354cead17f2bf4ffd170472645ed0d471d9ae5b49181667842996a9e3
---
