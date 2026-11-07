---
schema_version: 4
slug: 2026-11-07T1729-缓存前缀稳定性/plan_01
outcome:
  summary: 在原模型执行主路径实现稳定指令、不可变历史和有版本动态事实追加，使缓存前缀在压缩边界外稳定，并以真实协议请求及同模型对照验收命中收益。
  acceptance:
    - 同一generation连续请求的已见历史字段与顺序保持，动态budget/recovery不再重写早部；版本去重、清除、压缩回收、cold
      reopen和跨scope隔离均成立。
    - Chat/Responses/Anthropic最终适配后的完整prompt与断点通过对照及负控，provider
      source/reasoning、工具配对、Goal/原文/Guard/Stop/发布和持久化资格保持。
    - 同vendor/endpoint/protocol/model和固定workload前后对照记录真实usage、延迟、输入增长和压缩次数；只有重复对照支持收益才验收提高，未安排或未通过live则保留未验收，不宣称95%。
    - 最终冻结两仓执行适用AGENTS门禁、公开消费者/示例编译与独立审查，完整保留自然失败、锁/consumer变化和实际验证范围；合同通过不代替live/App/main
      CI。
out_of_scope:
  - 未经具名授权不自动发起live
    provider或App验证；不靠隐藏低值、改公式、调token计数或删除验收信息制造缓存收益。未完成live不得宣称收益验收。
  - 不将任意System自动降成User，不新增projection store、Goal权威、调度器或SQLite，不删除合理通用pub API。
  - 不自动升级SDK pin、接受新依赖风险、修改配置凭据、提交/推送/合并/发布。
design_ref: docs/supreme/specs/2026-11-07T1729-缓存前缀稳定性/design.md
design_sections:
  - ref: design.md § 审查结论与证据
    digest: sha256:146a2fd896bc5779054b4aea41ae21f7209c2badb1dc93f44d75cf9f98c24542
  - ref: design.md § 目标与范围
    digest: sha256:9f64f04843ed5e020110b20cd0243a1783e6edbae317e7560eb4f015b74de2ba
  - ref: design.md § 分层与复用
    digest: sha256:ca466c5e25102b082340a0ef0e31d4336daaef49bb52fb16963333f0f9ba767e
  - ref: design.md § 目标行为
    digest: sha256:8d3d4e021d140c4df143ed6066f5ce9ff1f237b71ca7827f784081b99cf58a4f
  - ref: design.md § 参考实现与候选取舍
    digest: sha256:f0699cb34ef3a668085d8b79069f898a9c2fb620ed3a8ec85c59426fc1a66604
  - ref: design.md § 验收标准
    digest: sha256:6373a72ff383ff5feebb58eee3f5b0607fc29f196c5206f811bcf086b9e61977
  - ref: design.md § 协作、文档与交付边界
    digest: sha256:4594834c76c23990b60be36b1889121c17218ca950734ea7e81dd445bc358dda
delivery_ref: null
todos:
  - id: capture-final-prompt-baseline
    summary: 沿同一生产请求构建链建立可复核的前缀失稳基线和语义分类。
    files:
      - echo-agent/src/agent/react/run/phases/compact.rs
      - echo-agent/src/agent/react/run/phases/think.rs
      - echo-agent/echo-core/src/llm/cache/diagnostic.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/compact_context.rs
      - echo-agent-cli/echo-agent-app-core/src/turn_context.rs
      - echo-agent-cli/echo-agent-app-core/src/unified_memory.rs
    acceptance:
      - 冻结HEAD、相关dirty字节、锁与工具链，绑定原run 877e196c及checkpoint
        e96ab169；原始失败和52次固定29440保留，不改写成已修复。
      - 用隔离合成/脱敏样本沿真实projector/adapter取得至少三次连续prompt，先观察budget/recovery更新导致早部差异；明确目前粗hash遗漏的字段。
      - 逐项分类固定规则和运行事实，列出各来源的owner、语义版本、保护和压缩需求；分类来自实际生产者，不从marker名称或第二消费者数量决定。
  - id: extend-original-projection-lifecycle
    summary: 扩展原通用projection主路径，支持动态版本只追加、重入去重与旧版本可压缩。
    files:
      - echo-agent/echo-core/src/compression.rs
      - echo-agent/echo-state/src/compression/mod.rs
      - echo-agent/src/agent/react/run/phases/compact.rs
      - echo-agent/src/agent/react/run/context.rs
      - echo-agent/src/agent/snapshot.rs
    acceptance:
      - 保留合理replace
        API；动态新版本在工具闭合的模型安全点追加，同owner/marker/version重入恰一次，清除为一次可恢复失效，不回删已见历史。
      - 最新必要状态protected，旧版可在原PreparedContextCompaction与CAS提交中归并；长运行不会因版本数增加形成永久protected增长。
      - checkpoint恢复、fork/subagent scope、required
        projector失败沿原权威处理；公共类型构造/消费者和原tool/source/reasoning配对负控通过。
  - id: switch-eclip-dynamic-producers
    summary: 在真实Eclip模型主路径切换内容分层，固定Goal/规则与高频运行事实分开。
    files:
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/compact_context.rs
      - echo-agent-cli/echo-agent-app-core/src/turn_context.rs
      - echo-agent-cli/echo-agent-app-core/src/unified_memory.rs
      - echo-agent-cli/echo-agent-app-core/src/infra/factory.rs
      - echo-agent-cli/echo-agent-app-core/src/project/prompt.rs
    acceptance:
      - Goal固定段不再嵌每步tokens_used；无界预算不重发完整预算；原文引用只随来源变更，后台状态和提醒只随对应事实变化。
      - 固定指令保留真实System权限，hot-memory规则与事实显式分类，不能自动降权；规则/工具/权限/模型变化允许新generation并及时生效。
      - 每个迁移切片切换真实consumer并删除已替代的动态前置重写，不保留第二套projection主循环；原Goal和输入authority不变。
  - id: qualify-provider-prefix-and-observation
    summary: 统一typed缓存分段和最终协议观测，防止抽象hash通过而wire仍变化。
    files:
      - echo-agent/echo-core/src/llm/cache/layout.rs
      - echo-agent/echo-core/src/llm/cache/diagnostic.rs
      - echo-agent/echo-integration/src/providers/openai.rs
      - echo-agent/echo-integration/src/providers/responses.rs
      - echo-agent/echo-integration/src/providers/anthropic.rs
      - echo-agent/src/agent/react/run/phases/think.rs
    acceptance:
      - cache分段与projection元数据一致，不按裸runtime_context或Eclip
        marker猜测；Anthropic不能将动态事实重新前置到system。
      - 最终prompt语义序列覆盖完整工具字段/parts/toolcalls/reasoning；description/strict等粗hash遗漏负控能定位首次差异，LCP不把字节数冒充token。
      - 生产日志仅有界摘要和字段路径，headers/密钥不进入比较，原始敏感正文不写通用日志；能力错误、native IDs与signed
        reasoning保持。
  - id: prove-cache-and-document-contract
    summary: 证明跨协议、压缩恢复与真实缓存收益，并同步正式合同和消费者说明。
    files:
      - echo-agent/echo-state/src/compression/mod.rs
      - echo-agent/src/agent/react/run/phases/compact.rs
      - echo-agent/src/agent/react/tests.rs
      - echo-agent/echo-integration/src/providers/
      - echo-agent/docs/zh/
      - echo-agent/docs/adr/
      - echo-agent/echo-agent-learning/examples/
      - echo-agent-cli/docs/zh/architecture/
      - echo-agent-cli/docs/zh/adr/
      - echo-website/
    acceptance:
      - 扩展已有unchanged_projection、tail_projection、required_projection、managed_compaction与oversized_steer/cold
        reopen回归；证明完整原文、最新Goal和必要动态状态可恢复。
      - usage归一化覆盖Chat inclusive、Anthropic disjoint、Responses
        details、缺usage、零值和cache write；同一规范化请求的生产loopback证据与纯合同分开。
      - 在独立允许的真实provider窗口内固定vendor/endpoint/protocol/model、工具和动态增长workload；调用前锁定每组请求数、重复次数、执行顺序与总调用预算。冷启动和同一运行连续增长分别报告，不反复发送相同完整请求代替原问题；保留每次自然结果，无改善或无live不验收收益。
      - 更新框架/应用各自ADR与中文说明，盘点并编译真实公共示例；SDK独立pin和官网量化宣传不自动继承本地合同结果。
artifact_id: plan:4c4f4942-5cf5-4ee8-97ae-ff8c06e2ef0f
lifecycle: ready
---
