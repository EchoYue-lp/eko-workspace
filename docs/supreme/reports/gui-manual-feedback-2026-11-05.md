# 2026-11-05 GUI 手验反馈与修复

用户已亲自开始验收。当前仍运行11月3日abdd修复版，不把源码验证写成已更新App或GUI通过。
本轮没有点击GUI、修改用户配置、重放模型请求或自动发送IM消息。

| 反馈 | 当前处置与证据 |
| --- | --- |
| 首次403／abuse_rule_rejected | 用户已说明选错Provider，按配置误选记录；不据此归因DeepSeek接口或某个工具 |
| 模型选择只写模型名 | 主聊天触发器／下拉、支线创建／模型选择均改为Provider配置名称加模型显示名；用UUID匹配，查询名称失败回退ID，仍提交原完整模型ID |
| 设置中的IM入口 | 基础与扩展增加原IM渠道项，基础配置移除IM区块及跳转按钮，渠道页继续保留设置分类导航和原控制／密钥处理 |
| Steer传expectedActiveTurnId=null | 正确最小复现实得RED；入口增加当前turn检查、保留队列，不调用不合法IPC；整份Hook102项通过 |
| 历史失败只剩泛化提示 | 后端原journal保留typed错误及失败reason；消息恢复已带executionStatus，但空行过滤隐藏原因。现在明确显示原reason，取消与重复正文不另显示、不生成最终回答；RED→GREEN |
| channel-runtime属于channel而非自身 | 默认pending协调只用于同一有效scope，保留None→runtime回退；真实File Store两轮回归RED→GREEN，10项同程序恢复／隔离／准入原回归通过，原错误scope仍拒绝 |
| Responses output item identity changed | 实际SSE字段仍未知，保留原一致性检查；增加有界事件类别／索引／ID是否变化／条目类型诊断，不输出正文、参数或实际ID；Responses92项通过，仅诊断已验证，不称实际兼容故障已修复 |

前端当前完整原入口：格式检查、npm test（含lint）、npm build均自然0，94文件／967项通过，
4项原性能opt-in仍跳过；新浏览器构建单独保留，原dist同卷恢复。当前源码与旧App前端
已出现本轮明确差量，不能继续使用旧498／498同源结论签新源码。前端与Framework局部修复
独立审查均pass／零findings；review后215项前端Outcome及runtime／诊断各1项再次通过，
Framework相关lib/tests Clippy -Dwarnings和fmt write/check均0。运行态和Responses问题
各自资格化，原GUI79及26项义务未完成。本次原件位于主根
`.echo-semantic/local/gui-manual-errors-20261105-r01/` 与 `model-provider-label-20261105-r01/`。

分层：Provider标签、设置导航、GUI错误与IPC前置条件属于EKO应用；持久化scope机制属于
Framework通用机制，使用原invocation／RuntimeStateStore／ConversationStore，不新建恢复
owner或并行状态机。examples和官网公开调用方式没有变化，官网来源同步仍受原dirty guard
约束，未绕过或将旧网站快照当新源码；正式产品说明同步在CLI中文配置指南。

## 新手验包已就绪，尚未启动

本次构建直接读取主根Framework／CLI的3976路径冻结清单，使用原冻结Rust1.99.1和
Tauri CLI2.12.4、jobs1、offline、locked，单独冷target；不再复制完整源码树。输入与原命令
保存在主根 `.echo-semantic/local/gui-manual-feedback-App-20261105-r01/`。

实际冷构建自然0，release编译39分16秒、零编译warning/error；两条no-sign提示与编译结果
分列。新包主程序为 `d5f1ea3a3c731cd169a5e1be3e0562cc558427abe282bf902b6173e7256ce57b`／
177013792字节，adhoc签名与deep strict校验各0。真实编译.d绑定HTML唯一主入口
`assets/index-Dtto7E7w.js`，149资产前后逐项一致，3976源路径集合／字节／模式与HEAD无漂移。
独立产物审查pass、零findings；静态启动入口检查0，未查询GUI进程或启动新版。

新包在 `retained/Eclip.app`，入口 `启动新版.command`，ready SHA256为
`799be51042d1e7c237d97819bf60f3ee22efd62a1b386a2b6772e61fb30f5053`。新入口沿原隔离数据路径，
旧Eclip存在时只提示正常退出，不自动退出、不读配置内容。App清理前另保全4881编译元数据
文件／306904418字节及.d原路径映射，原unsigned程序和签名App均在target外；官方clean0，
报告移除1340件／2.4GiB，target已不存在。完整包终态为本目录 `settlement.json`，SHA256
`d9348bc33383aac48f4e01ce19b0976a6b60dfedcfa2f3ed3691c0739530f034`。

旧abdd包保持原字节；当前30项索引只将App字段切到新候选、更新当前手验指南哈希，旧索引
与指南副本保留，不改原始proof-index.json。原49项CLI WIP中45项不变、4项按用户请求更新，
新增13项，当前62项逐字节保全。手验指南另纠正A06模型入口／本地保存与网络测试区别、C02
隔离数据根预期，避免把正确路径误报成缺陷；启动一致性检查改为读取入口页真实声明，
仅在临时文档副本中置入缺失路径的负控实际退出1，旧脚本仍存在也不能误报通过。

自有测试批次先同卷保全3643件／110815333字节及两个实际程序，再官方clean退出0、
移除903件／1007.5MiB，原target已不存在。保全时误将Cargo缓存标记一起移走，首次clean
因此被Cargo保护机制拒绝101；从已保全原件恢复同字节／模式标记后，原命令正常0。
原101和恢复票保留，不伪造新tag或手删目录。所有旧数据、配置、锁／policy和历史基线不变。
