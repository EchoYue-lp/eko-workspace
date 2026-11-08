# 新会话首条排队、重复输入与 Responses 失败

本次只追用户指定的 `conv-1793886223809-nl5rpi`，对应远程工作空间
`ubuntu-ssh-test-20261102`。证据来自用户截图及该会话原journal；未改用户配置、消息或队列，
未由Agent重放远程命令。原件位于root local/new-conversation-queue-20261105-r01。

## 原会话的实际结果

- 21:43:43：首个输入持久化；21:43:58开始原turn，21:44:12实际drain；21:44:16失败。
- 21:44:41：同样内容进入第二个不同input ID；21:44:55 drain；21:44:58再次失败。
- 原始记录确有两个user和两个空assistant，不能把两条输入解释成单纯重复渲染。
- 两次都在 `response.content_part.added` 失败：同一item ID，父项已知reasoning，旧适配器却猜成message。
  本会话没有实际工具调用记录，不把它当成远端文件或进程查询成功。

## 根因与修复

首次发送创建conversation后，观察effect随null→持久ID重挂载。此时原send仍在等待启动回执，
恢复流程读到尚未启动的快照和刚持久化输入，便再次dispatch同一输入、生成另一messageKey，
抢掉原pending投影。原始启动回执因此不能清原草稿或呈现后续错误，用户看见就绪与排队并存。
新增真实组合控制首先复现同一次发送产生两个IPC调用。现恢复与microtask复用原pending边界，
不抢原发送；同root快照不提前创建第二个assistant；保留原journal重放与精确finally。
启动回执前只投影“正在提交”，不改后端运行状态；当前已接受输入不被显示成Steer追加项。

Responses内容事件不声明父项类型。私有ledger现在保留未知类型，等待明确类型单向绑定，
不把reasoning误猜为message；ID变化、跨索引复用及已知类型变化继续拒绝。正控完整接收
reasoning、正文及唯一终态；原94项Responses回归、严格Clippy与fmt全部通过。

冷历史里的空assistant元数据没有executionStatus，错误重放时也没有活跃assistant引用，
原路径因而忽略失败。现从原error事件以真实ID、时间做只读显示，重放去重，不autosave或
重写canonical。错误按原时间稳定插回既有消息之间，保留canonical相对顺序，避免错误重试
归到后续输入。终态仍只由原turn_status决定。

用户新增的“最近任务不能折叠”已用原生button、aria-expanded/controls和局部状态修复，
默认展开，可再次展开；原列表、导航与请求路径保留。

## 当前资格

最新前端完整975P／4原性能opt-in跳过／0F，格式、lint、类型及浏览器构建0；review后
207项相关Outcome0。Framework94项Responses及相关Clippy/fmt0，Outcome原复现1P。
两个独立审查均pass／零findings。旧RED、首次历史位置finding及中间真实失败原件保留，
不回填成功。源码当前本地资格不代替新App、真实Provider、手机或main CI验收。

新包已直接以主根3976路径冷构建成功，root local/new-conversation-repair-App-20261105-r01；
主程序177032960字节，SHA256 `a4405b14aa38a5edd6568fbd5fa8b3d203fbd60962cb4409ac576f1e6e39c279`，ready SHA256 `3e03211350410bb771196627b357261aba6636a614c6c5b8644c7ccefad70bcf`。
实际149新资产、HTML主JS及编译.d逐项绑定，源路径／字节／模式零漂移；
本机adhoc签名、deep strict及默认静态启动器检查均0，实际包独审pass／零findings。
签名App、原unsigned和4881件必要编译原件（306917407字节）
已在target外保全，准确官方clean0后target不存在；旧d5包身份保持。
当前30项索引和验收入口切至本包，旧文档原字节已备份；未启动新版或退出用户App，
未改实际会话与配置。GUI／真实Provider验收仍待用户，原79项及26项义务继续未完成。
examples／官网调用API没有变化，官网原入口仍成立，产品流程已同步CLI中文配置指南；
不绕过官网原源码dirty guard。源码测试、包资格与真人验收分别记账。
