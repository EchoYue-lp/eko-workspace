---
schema_version: 4
slug: eko-gui-product-convergence/tool-inventory-and-exa-boundary
outcome:
  summary: 形成覆盖定义、注册、模型曝光、消费者和结果产物的工具事实清单，并核对 Exa 独立提交及其暂不推送、不合 main 的边界。
  acceptance:
    - 在可识别源码提交上给出框架/EKO/SDK 的工具事实清单：区分定义、注册、实际曝光条件、调用消费者、状态/副作用 owner、结果读取；SDK
      accepted API、Host operation 和模型工具不能混为同一种计数。
    - 用真实调用路径核对已有 tool_search/按需 schema、分页/4K 结果预算、typed artifact
      与完整读取机制，以及直接执行、Subagent、验证和恢复的适用范围。源码事实、已有回归与本次实际测量明确区分，不虚报性能或真实服务验收。
    - Exa
      本机提交4343f0b736beb5c8606e31146528a2fe8be80510、保留引用及不推送/不合main的边界可复核；web_search
      与 web_fetch 继续区分，不把缺少远端引用写成可远端恢复。
    - 报告给出可定位的重复/过时/可复用项、具名退役与后续治理接缝，明确合理框架public选项保留和未证实项；完成独立事实复审与相应文档/语义验证。
out_of_scope:
  - 不修改生产工具、预算阈值、feature、SDK生成合同或执行/存储权威；本结果是事实清单，不实施工具收敛或A2A/LSP删除。
  - 不推送或合并Exa，不调用未经指定的真实服务，不用mock/静态数量冒充真实性能基准。
design_ref: docs/supreme/specs/eko-gui-product-convergence/design.md
design_sections:
  - ref: design.md § 2. 已固定基线与当前事实
    digest: sha256:280479f678d0b5f130eb4852b8ff24c0b120d8dd64931611fc40a8113665cfa4
  - ref: design.md § 4. 系统边界与唯一权威
    digest: sha256:ad6ce63a85bbe1cf1d9a175e78b800f0070d97aeac24069fd4c0ba455d1c817f
  - ref: design.md § 工具的统一语义
    digest: sha256:5b0b172fe617df162c9e94a499e31811b770495ace98a22d550885deb9654a1f
delivery_ref: docs/supreme/specs/eko-gui-product-convergence/plans/delivery-map.md#tool-inventory-and-exa-boundary
todos:
  - id: trace-tool-authorities
    summary: 从工具定义追到实际注册、曝光条件和消费者，形成可复核的分层清单。
    files:
      - docs/supreme/specs/eko-gui-product-convergence/audits/tool-inventory.md
    acceptance:
      - 每组工具记录源码位置与真实调用路径，区分框架机制、EKO策略、SDK合同及适配；有定义/有注册不自动等于主路径可达，无当前EKO调用不自动等于框架死代码。
  - id: trace-results-and-execution-boundaries
    summary: 核对现有输出/产物/错误与执行范围机制，明确治理复用点及证据缺口。
    files:
      - docs/supreme/specs/eko-gui-product-convergence/audits/tool-inventory.md
    acceptance:
      - 记录完整结果的真实读取入口、取消/错误与副作用归属，并标明直接、Subagent、验证、恢复的实际消费点；旧ADR0034/0039委派条款及T8/Host边界只按已证事实承接，不自行选择新政策。
  - id: freeze-exa-and-handoff
    summary: 核对独立Exa来源并把清单交给消费者去依赖、工具治理和基准结果。
    files:
      - docs/supreme/specs/eko-gui-product-convergence/audits/tool-inventory.md
    acceptance:
      - Exa保护SHA/分支及当前main差异有精确记录；报告所有结论能定位到所读源码或正式合同，复审无未处理的事实错误，已有生产/合同/保留引用没有变化。
artifact_id: plan:92c06a40-2352-4be1-893b-c30d70d83dee
lifecycle: ready
---
## Notes

在本机已审合同提交上实施：顶层 e0e8fdde15a25a84dc6b8fa57ccf442da87c8c25、framework c3b56b4ee8a73565e6a2a621b9a7ddfb644ed3aa、EKO 0de6960a8610002b471e99034ca39c3dad903ec7、SDK 961da0e0d17b815653d80cb4bf4973248f543770、官网 a157575354baea1648894de4b6ed9f0dd836f9ad。合同内容已经交付协调者并通过独立审查，main/CI 交付仍须按原门禁收口，不把本地贡献误写为主线完成。

复用先前只读准备中的完整工具与性能入口调查；当前只增量核对上述提交的相关源码字节。产物属于顶层跨仓阶段审计材料，长期框架/SDK/EKO事实源仍归各自子仓。报告写入者只拥有此审计文件，不接管生产文件。
