---
schema_version: 4
slug: zstd-safe-chain-repair/framework-safe-chain
outcome:
  summary: Framework 保留 data/statistics 真实能力，采用两份最小 Polars 补丁和安全 zstd
    链，按消费图正确审计五项来源，并形成经验证、可精确恢复的本地修订。
  acceptance:
    - 两份官方0.55.2包共445发布成员除两个normalized
      manifest的zstd要求外逐字保持；当前根真实解析安全链，不降级已有Tokio修复。
    - 五声明完整校验；F完整图选五、真实无data的CLI锁选原三、合法单父图按真实引用选取；被引用缺包/重复/错版/错source和输入覆盖继续拒绝，投影不改变真实依赖边。
    - 取得本次真实data/statistics与Arrow/Parquet/context及工具消费者、适用feature/示例编译、Framework自身标准依赖门禁证据，原始失败和自然出口保留，不用probe或其它仓风险合同代替。
    - 形成精确Framework本地Git对象/树和恢复回读材料供SDK后继候选；正式说明/例子/官网同步检查有据，远端冷恢复与main/CI仍独立未完成。
out_of_scope:
  - 本Outcome不修改SDK Host pin、根patch、接受集或既有94路径候选；SDK消费结果按交付图依赖另行实施。
  - 不搬迁echo-tools的data/git目录，不改变EKO运行状态/工具schema，不删数据能力、不改名/虚构版本/增加ignore或扩大风险接受。
  - 不push、merge、关闭Issue、发布或部署；不将本地传输冒充远端公开冷恢复。
design_ref: docs/supreme/specs/zstd-safe-chain-repair/design.md
design_sections:
  - ref: design.md § 目标行为与范围
    digest: sha256:a2390de8ab2976d06e65427738449815b7409f5e549e7608ca41f294cb336fa5
  - ref: design.md § 系统边界与已有实现
    digest: sha256:a3a8d403dc13e4fae638bf6f10247aff8e9a0ad9b28ca7f92c4df3410c6b1abc
  - ref: design.md § 原包归属与构建图
    digest: sha256:02896a8b0d70a55fc134d9a5eb1a60bf34387f684f33e7faea005b5a852f6c75
  - ref: design.md § 五来源审计合同
    digest: sha256:ac5f2135815a0935621948f0edca5e7ae61761db5749a98ffcdb9264a05d4551
  - ref: design.md § 公共类型与接受合同
    digest: sha256:c9501a1a33fc73af9317dc4ea80ef1ed7c8b67e708204c500796dd92d05fa89e
  - ref: design.md § 来源切换的资格关系
    digest: sha256:c20aef86ad594bf3b0f702ea484659904a2983869d2b5b1175d09d72f98a96c7
  - ref: design.md § 异常、并发与恢复
    digest: sha256:a1b2bd803ef9cd8cbf3b5ee1b37194fa10bd26cf580eb5febc5145081cbacf18
  - ref: design.md § 验收标准
    digest: sha256:6c6c4f60b27b446e221f06e45bae119d861f8484e507bc0d88948445a04f32b0
delivery_ref: docs/supreme/specs/zstd-safe-chain-repair/plans/delivery-map.md#framework-safe-chain
todos:
  - id: select-official-parent-patches
    summary: 接入两份最小官方父包补丁，当前根解析唯一真实安全链并保留原归属。
    files:
      - echo-agent/Cargo.toml
      - echo-agent/Cargo.lock
      - echo-agent/vendor/polars-arrow-0.55.2/
      - echo-agent/vendor/polars-parquet-0.55.2/
      - echo-agent/vendor/README.md
    acceptance:
      - archive
        checksum、445成员和许可证/VCS回读吻合；两个manifest之外无上游源改写，两包不进入第一方workspace成员。
      - 真实lock/metadata保持其它当前依赖意图，只用同源两父与安全链，不复制旧probe整锁或手写解析边。
  - id: audit-actual-consumer-origins
    summary: 扩展唯一审计生成器，完整五声明按真实消费图选三/五或合法单父，错误继续拒绝。
    files:
      - echo-agent/vendor/registry-origins.toml
      - echo-agent/scripts/generate-vendored-media-audit-lock.py
      - echo-agent/scripts/tests/test_vendored_media_audit_lock.py
    acceptance:
      - 正式当前F与既有CLI锁均经同一helper；原三媒体身份、拒绝及字段边等价保留，新父被引用却缺失、重复/错source、声明不全等控制有真实回归。
  - id: qualify-data-and-recoverable-source
    summary: 验证真实数据压解和工具能力、标准依赖来源，保存SDK可精确消费的本地修订与恢复入口。
    files:
      - echo-agent/echo-tools/src/data.rs
      - echo-agent/echo-tools/src/data_quality.rs
      - echo-agent/tests/
      - echo-agent/docs/adr/0100-zstd-safe-chain-vendoring.md
      - echo-agent/.echo-semantic/maps/map.workspace-architecture.md
      - echo-website/docs-sync-manifest.json
    acceptance:
      - 真实Arrow/Parquet压解、context跨尺寸复用/坏帧恢复及含中文/null/数值的原工具往返有具名证据；data/statistics、相关media组合及适用示例按原feature边界可编译。
      - F本次audit/deny、五源扫描、完整yanked/license/source及原精确duplicate合同分别记录；新风险和网络/索引错误不冒通过，不继承SDK例外。
      - 源码/文档审查与最终验证后本地提交并回读精确对象；每批Cargo保必要制品和原日志后准确clean，不承诺远端可取。
artifact_id: plan:5f9e1509-cdb3-46eb-8947-d4ffffd8505b
lifecycle: ready
---
