# EKO／Eclip 总 Goal 并行执行会话

本表是协调者的分工索引，当前事实仍以[唯一交接](eko-current-handoff.md)和各线最新回执为准。唯一总目标是[完整迭代总 Goal](../goals/eko-eclip-complete-iteration-2026-10-27.md)，包含旧26结果的全部未完成与新增原生执行切换。

用户明确要求“你是协调者，真正任务执行者应该新创建会话”，因此以下为新创建的独立执行会话，使用 lp-agent 已有 local 项目，均在唯一主目录执行，未创建外部 worktree。协调会话 `01a17c1f-939d-7311-bba9-41e529912138` 维护总目标、依赖、文件写权、冻结、集成和本地 stage／commit，模块实现由执行会话承担。

| 线路 | App实际标题 | 会话 | 创建后观察 |
| --- | --- | --- | --- |
| framework | EKO 全量迭代：Framework 原生消息与执行终态 | `01a17d6c-d8ad-7900-869b-ee9e6a559473` | active |
| providers | EKO 全量迭代：全部 Provider 无损适配 | `01a17d6c-db38-7600-b07d-9bedfa52d4df` | active |
| eko | EKO 全量迭代：GUI 运行时消息与旧功能收敛 | `01a17d6c-de65-7503-b2d6-99facb526ec8` | active |
| sdk | EKO 全量迭代：SDK Host 协议与三语言 | `01a17d6c-e195-7f90-a8a4-8da3dc80a009` | active |
| im-cron | EKO 全量迭代：IM 微信 cron 与共享恢复 | `01a17d6c-e502-7232-b645-befc74098be5` | active |
| workspace | EKO 全量迭代：工作空间 SSH 终端与分叉 | `01a17d6c-e8c2-7781-a9a2-a83edb883664` | active |
| governance | EKO 全量迭代：工具 插件 Hook 与能力退役 | `01a17d6c-eb8a-7162-aa71-d096b3ffec4b` | active |
| delivery | EKO 全量迭代：官网 文档 性能 资源与发行验收 | `01a17d6c-ee85-78e3-b3a1-a6fc67ffc818` | active |
| runtime-acceptance | EKO 全量迭代：Framework 执行场景与恢复回归 | `01a17d8e-23e1-78a3-9d3e-0d8b57c31bef` | active |

## 文件责任与正式源窗口

- Framework 主执行会话：Framework 原生消息／统一结束、Core类型／ReAct／事件、内置Think/Final退役、历史读取适配、Frame文档ADR例子，并整合provider／专属退役作者的公共文件变更。
- Provider作者：全部provider的无损解析／归一化／回放与流非流对照，负责 `echo-integration/src/providers/` 专属内容；正式源窗口由Framework主执行会话协调，不与其抢同仓活跃预检。
- EKO主执行会话：GUI／chat_driver／TaskRuntime／Steer Goal／Subagent／完整原文与工具历史等现WIP，整合AppState、Tauri共享注册、conversations、ChatPanel／MessageBubble／store／endpoints／types等共享文件。
- SDK主执行会话：SDK完整Host／协议／三语言／合同／pin／示例／正式文档与独立验证。
- IM／cron作者：渠道、个人微信、scheduler与专属面板／命令／恢复，公共状态和接口变更交EKO主写入执行会话合流。
- 工作空间／SSH／Fork作者：项目、SSH／终端、Worktree／资源迁移和分叉完整结果，公共AppState／conversations／chat UI交EKO主写入执行会话合流。
- 工具／插件／Hook／退役作者：独立能力治理、消费者验证和专属退役；Frame Think/Final和公共facade归Frame主写入者，EKO共享文件归EKO主写入者。
- 官网／工程作者：官网正式源，及性能／资源／问题／发行证据和可合入中文材料；总Goal／唯一交接／Master／next／本表由协调者维护。
- 执行场景／恢复作者：后继新建第9个真实执行会话，制作真实ReAct的进度续跑、工具优先、校验、取消与持久恢复等完整场景回归和必要修复精确增量；新测试先在本线候选交付，既有core/types/phases/react_loop/snapshot仍由Framework主写者独占。由其及时整合并在同批稳定target执行，不另造executor或抢repo预检。

当前本机预检记录每repo只允许一个活跃owner（已读安装器 `record-preflight.mjs` 的owner匹配合同），所以各repo正式源由上述主执行会话持真实SID，领域作者并行交真实实现和定向验证，主写入执行会话及时整合。不能伪SID、改外部工具或用这个局部约束让所有工作等待；独立repo及候选实现／数据／回放／文档／审查持续并行，候选不得无限积压。

## 证据、资源和授权

各线在主根 `.echo-semantic/local/parallel-executors-20261027/<slug>/coordination.md` 和 `evidence.json` 记录实际成果／依赖／准确候选或源指纹／命令自然出口。协调者用App的wait/read观察，执行会话无需主动向其它聊天发送消息。

所有旧编辑绑定已备份并正常释放，仅释放编辑，不声明完成：`.echo-semantic/local/new-execution-chats-transfer-20261027/scope-transfer-receipt.json`。旧Frame16、EKO raw9／readonly7／tool14、SDK94、两Fork候选与全部原失败保留；新会话必须先核当前实际Git／进程／指纹，不能直接回放旧整个文件。

最多5条focused Cargo／各jobs≤5／debug0，完整重型矩阵与正式性能测量独占。每条独立target／TMP／TSRS，启动前先在本线coordination记精确命令／依赖冻结和窗口需求；协调者安排窗口。保全日志／自然exit／必要制品与哈希，成功核reader后独立准确clean，失败停止后续清理。人工扫码／认证／收件／新原生和外部push／merge／发布／部署各保真实资格；先准备可审结果，不扩大风险或排除。

