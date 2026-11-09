# SSH 上下文启动修复：实现与验证

## 已落实的行为

本次针对“你好”保存输入到首个 Agent 状态事件耗时 32.499 秒的启动缺陷。首次创建与每条消息准备的重复文件访问、逐操作 SFTP 初始化、串行标志探测和实例表长锁已经在源码中修复。

1. 由实际 conversation driver 驱动的入口使用 `acquire_for_turn/agent_for_turn`。工厂只发现构建必需的角色定义，规则、结构与 Git 上下文由原 invocation 准备一次。普通原始消费者仍取得完整初始化；取消前未进入 driver 的实例在原对象上补全，不重建或丢消息。
2. 整次只读准备复用一个 SFTP 子通道，结束后沿原 close/child/reader/资源 owner 结算。读取失败仍关闭，关闭错误传播；取消观察者后原物理准备与 guards 持续到真实结束。
3. 项目根只解析一次，指令加载接收解析结果；已知标志按原优先级分批 STAT，最多 32 项在途。标志探测不枚举整个仓库或 HOME。批次不是单条网络请求，四级无标志目录一次搜索仍可能有 40 条 STAT。
4. 构造互斥与实例表锁分开。已有会话获取不等待远端构造。关闭/退休先封准入、短暂等待在途构造，再释放构造互斥等待原执行；配置模板更新与发布保持对应协调。
5. 成功准备后先发布完整事实，再确认原实例的初始化标记；标记按 Agent 实例身份匹配，外来 Agent 不能翻转原缓存。活跃追加输入、只读观察和失败后恢复已覆盖。
6. 显式项目根与产品数据目录、项目内 cwd 分开处理；嵌套 Git 仓库不重定向显式项目的 raw 补全。GUI、普通 IM、Continuation 接对应 driver，Planned Resume 保留完整初始化。

通用只读会话与有序路径探测属于 framework；Eclip 初始化、Workspace、SSH 认证和产品策略仍在应用。正式决策见 [ADR 0063](../../../echo-agent-cli/docs/zh/adr/0063-project-context-startup.md) 与 [ADR 0106](../../../echo-agent/docs/adr/0106-file-system-read-session.md)。本次使用现有 `PreparedProjectContext`、指令投影和运行时，没有新增任务状态机或第二套规则存储。

## 已取得的验证

所有新验证的原始日志、真实退出码、审查与源码摘要保存在主根 `.echo-semantic/local/ssh-startup-repair-20261106-r01/`。各测试集合有重叠，不累加为唯一测试总数。

| 验证 | 最新结果 |
| --- | --- |
| 启动筛选，包含 channels | 14 通过，0 失败，0 忽略 |
| 审查后的最终实际 native 启动 Outcome | 同一冻结运行逻辑，14 通过，0 失败，0 忽略 |
| Agent 池 | 81 通过，0 失败 |
| SSH 生命周期及本地真实标准 SFTP 协议 | 36 通过，0 失败；不连接真实服务器 |
| 规则加载 | 9 通过，0 失败 |
| 项目相关 | 20 通过，0 失败 |
| 显式项目 bootstrap、Resume 路由、无 Run 上下文 | 各选中 1 项并通过 |
| turn context | 7 通过，1 项原 Root 专用长时 loopback 夹具未执行 |
| channels lib/tests Clippy `-D warnings` | 自然退出 0，零警告 |
| framework 文档 | 11 通过及 7 项预期编译失败测试通过；2 项原示例忽略保留 |
| 显式 GUI 编译 | 自然退出 0 |
| GUI Rust 测试 | 141 通过，3 项原忽略；根文档另有 1 项原忽略 |
| 产品可靠性示例编译 | `lh6_product_soak` 显式 GUI feature 编译通过，零警告；未执行真实 Provider/长时 soak |
| 格式 | 两仓 cargo fmt/check 与 include 文件显式 rustfmt/check 均为 0 |

真实本地 SFTP 夹具复用生产通道初始化与关闭路径，使用 `/usr/libexec/sftp-server` 的 stdio，覆盖多次读取、71 个候选的批量处理、写入/逃逸拒绝和原资源结算。它不包含 SSH 登录或远程网络时延，不能当作真实服务器提速验收。

独立 reviewer 对最终实现返回 `pass`，Critical/Important 均为 0；一个示例无用 `mut` 的 Minor 已按建议删除并重新编译验证。原 21 路径摘要在 `review-inputs-r02.json`，追加示例与旧 doctest 的 23 路径快照在 `review-inputs-r03.json`；后续非语义确认有独立记录。

## 失败与修正记录

- 首次新增夹具缺少 `Arc` 导入，编译自然 101；已修正。
- 第二轮 8 项通过、3 项失败，原因是新增夹具没有安装正式 TaskRuntime 环境绑定；补齐同一 files/executor 的原绑定，没有放宽生产检查。
- 第三轮预检发现初始化接缝参数错误，主动 SIGTERM，保留真实信号记录，未当作通过。
- 新复合 active 测试曾栈溢出 SIGABRT。原池创建与真实工厂→driver 原生对照各通过 1 项；对大 future 边界装箱后新完整回归通过，没有提高线程栈或放宽断言。
- 首次 Clippy 发现嵌套条件样式，按提示修正后同一严格命令通过。
- 原 checkpoint doctest 错把 `ToolBatchEnd` 当正例；Git 历史及当前实现确认示例不符。只改为实现已支持的 `Cancelled`，全部非文档源码保持相同，完整 doctest 后继通过。

以上原件保留，不将旧失败改写为成功。

## 文档、示例与工作区边界

产品可靠性示例已同步实际 GUI 的原 lease/foreground/driver 入口，避免以后继续用旧 eager 初始化路径测量产品。framework 新 API 的编译文档已进入真实 doctest 链路。应用配置字段、SDK Protocol 没有新增或迁移。

官网镜像同步脚本要求干净、可标识的 source checkpoint；当前两仓有用户原有 WIP，不能绕过该 guard 或为同步擅自提交。正式文档已在所属子仓维护，官网按后续正式 checkpoint 同步，不把本地候选写成已发布 main。

源目录始终是唯一主根，沿用非 main 的当前协调分支。其它 Provider/品牌相关并行改动完整保留，不归为本任务成果。没有 stage、commit、push、merge 或发布。

## 构建包状态

新 App 在 `.echo-semantic/local/ssh-startup-repair-App-20261106-r01/` 构建。输入绑定两仓 3982 个实际源路径，无源码复制；真实前端 149 个资产已在本次构建中捕获。当前构建、签名与入口状态以该目录后继 `ready.json/entry.json` 为准；没有 ready 回执不能声称新包完成。

测试批次已保全 4 份实际 native 程序和 8691 件必要编译证据，确认自有 target 无读取者后官方 `cargo clean` 自然 0，原测试 target 已不存在。App 的独立 target 在构建结束、保全和读取者核对后另行清理。

## 验收界限

这些结果证明源码、受控本地消费者、生命周期与标准 SFTP 协议路径的修复，不证明真实 `ubuntu` 服务器的网络时延、实际 GUI 操作、真实模型服务或 main CI 已验收。32.499 秒量到首个 Agent 状态事件，包含意图分类，不等于纯文件读取时长。

实际复测应先完成 SSH 登录并进入同一目录，再分别记录新会话/后续消息的创建、项目准备、模型请求和首事件；把首次认证时间单独记录。新日志提供构造等待/创建、文件准备和 turn 项目准备耗时，便于继续归因。

与 moss-agent 的完整能力和架构取舍见[全面对比](ssh-eclip-moss-comparison-2026-11-06.md)。大仓库 grep 的目标侧执行、远端进程真实终止及常驻设备恢复属于该对比指出的后续能力方向，不能从本次启动修复外推出这些能力也已补齐。
