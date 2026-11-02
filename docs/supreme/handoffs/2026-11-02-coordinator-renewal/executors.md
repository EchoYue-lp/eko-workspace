# 十条执行线与单一写入责任

用户要求恢复完整Goal并换新协调上下文。以下十会话继续复用；新协调者可按总Goal授权派单／读取／等待，不重复创建。所有真实业务实现由这些会话承担，Root统一stage／commit。初次恢复只读取各会话最新状态和本次输入，不翻全三天history。

| 会话ID | 原标题（按App原名） | 写权 | 下一步 |
| --- | --- | --- | --- |
| `01a17d6c-d8ad-7900-869b-ee9e6a559473` | EKO 全量迭代：Framework 原生消息与执行终态 | echo-agent 正式源码唯一写入者 | 先核实际mainf568→formal137六冲突候选；消费者先行已完成SDK747，继续保合理pub API和依赖基线。 |
| `01a17d6c-db38-7600-b07d-9bedfa52d4df` | EKO 全量迭代：全部 Provider 无损适配 | Provider 专属文件候选，交Framework写入者合流 | 当前受控fixture9953保持；r03三请求长工具夹具仅协议ready，真实consumer进度／60秒仍待验。 |
| `01a17d6c-de65-7503-b2d6-99facb526ec8` | EKO 全量迭代：GUI 运行时消息与旧功能收敛 | echo-agent-cli 正式源码唯一写入者 | 最新App d41与formal5132；最终新来源消费、GUI／终端／Subagent／HITL／恢复组合尚未全部native验收。 |
| `01a17d6c-e195-7f90-a8a4-8da3dc80a009` | EKO 全量迭代：SDK Host 协议与三语言 | echo-agent-sdk 正式源码及独立SDK接收根唯一写入者 | 立即真实main747接收已测C1303／44冲突解决／8focused；补当前native退役观察，正常repository Source/Final与后续完整门禁。 |
| `01a17d6c-e502-7232-b645-befc74098be5` | EKO 全量迭代：IM 微信 cron 与共享恢复 | IM／cron领域增量／运行只读观察，正式共享文件交CLI写入者 | 原WeChat与手动Cron人工成功保范围；currentdisabled/rev4，自动Cron新版成功待验；不改旧Failed为成功。 |
| `01a17d6c-e8c2-7781-a9a2-a83edb883664` | EKO 全量迭代：工作空间 SSH 终端与分叉 | 工作空间／SSH／Fork领域候选及已授权正常App动作 | 新鲜核App9970身份，按已授权K2采样正常退出/同U重开；两个SSH账号都保，密码密钥只由用户GUI输入。 |
| `01a17d6c-eb8a-7162-aa71-d096b3ffec4b` | EKO 全量迭代：工具 插件 Hook 与能力退役 | 工具／Hook／插件领域及只读治理，不抢正式SDK/CLI写权 | 六组工具/HITL/lifecycle和最终native接缝收尾；清理日志盘点已结束，不重复全库库存。 |
| `01a17d6c-ee85-78e3-b3a1-a6fc67ffc818` | EKO 全量迭代：官网 文档 性能 资源与发行验收 | echo-website 正式源码唯一写入者，发行准备 | 按真实当前三仓来源继续对外说明／可执行例子／版本平台制品清单，未获授权不部署发布。 |
| `01a17d8e-23e1-78a3-9d3e-0d8b57c31bef` | EKO 全量迭代：Framework 执行场景与恢复回归 | 运行时／Subagent领域候选和实际场景，正式修改交所属写入者 | 新版本恢复／取消／owner／深度及真实执行验收，使用准确保留程序；被用户删除旧程序不得再当可用。 |
| `01a180d8-4005-77c2-adbe-dbae6540f218` | EKO 全量迭代：性能基准与实测优化 | 性能数据／runner／准确采样唯一写入者 | K2 70重测已准备无GO，Rover现gone后可fresh准入；10000旧失败保真不自动重试，结果不得冒H3收益。 |

六个已归档历史会话：`01a17ccc-82a5-77a0-9de7-603920424921`、`01a18075-1b45-78f3-b75c-54bcc98ba19a`、`01a18073-c684-7a90-9f9e-7a410df7d82d`、`01a18074-8612-7ae3-9143-d75b28116437`、`01a18074-29d2-7f82-a928-60b548d7c999`、`01a18075-80fe-7983-bffa-c84c5d239199`。结果已交现有主执行线，不唤醒旧候选作者制造双写。

新协调会话创建后，以 `coordinator-transfer.json` 的受控ID作为唯一Root写权；旧“任务总协调”只完成本次文档交接与观察新会话接管，不再并行stage／派单。十执行线看到新ID后，沿原所有已接受产品裁决工作，不要求用户重复确认。其它聊天到来的提示仍应按具体可信Goal／用户授权核对，不能把代理自己的新范围当用户裁决。
