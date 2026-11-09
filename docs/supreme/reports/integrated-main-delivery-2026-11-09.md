# 2026-11-09 综合主线交付记录

## 交付范围

按用户当前授权，将IM、缓存、执行与保存、SSH、界面和文档的既有迭代分别归入独立子仓main，随后收敛主工程gitlink和旧分支。learning指`echo-agent/echo-agent-learning`，随框架交付。必要历史标签及原始失败、日志、验证来源继续保留；不以删除分支替代功能整合。网站部署、自动GUI、读取或覆盖原账号数据不在本次操作内。

## 当前主线与验证

| 仓库 | 已推送main | 实际验证 |
| --- | --- | --- |
| Framework（含learning） | `3413d64dae23353c6d02c546ac374deded855827` | 最终适用本地门禁通过；精确main CI 7/7成功 |
| CLI／Eclip应用 | `233516378c2b3e8e552ee0842cc8001bdfc6c407` | 原完整Rust/GUI八项自然0；Node22/26各1055P及4原skip；精确main CI 4/4成功 |
| SDK | `5bbaf9af09c7a8b232b1d7a2f3ebd3e1d193d7b4` | 适用Rust/Host/三语言完整门禁、原生成链及固定c0bc最终strict通过；精确main CI 9/9成功，官方全新冷仓复验通过 |
| 官网 | `1e42fc9f48b332c977d596036ec4404fa8773053` | 最终完整verify自然0：64单测、350静态路由、15浏览器通过及1原skip；fresh npm审计0；精确main CI 1/1成功 |

四份子仓已按功能覆盖、历史恢复标签和实际远端tip核对后清理至本地及远端仅main；主工程在本记录和已推送子仓gitlink提交后继续相同步骤。四仓仍独立管理，不把SDK的framework pin与superproject gitlink视为同一绑定。

## 已闭合的本轮CI故障

原CLI main的Node26原生Storage getter使Vitest没有采用真实jsdom Storage；测试层现绑定每文件真实对象，Node-only合同继续保持。Tailwind采用原版本范围内官方首个兼容4.3.1，完整锁差异经过审查，生产CSS与实际组件布局对照、两个Node完整构建结果均核验。没有降级CI Node或隐藏警告。

Linux七个正向测试以空.git目录或未创建cwd构造了无效前置，现使用真实git init、独立toplevel证明及TempDir原生命周期；原断言和预算保持。SSH后台命令在dash下的stdin丢失由父shell预先保存独立fd修复，中文和emoji输入、stderr及退出码完整保留。直接PID期限用例与None后代持管道负控分别证明原语义：后代尚存时有界返回Unknown、原owner/guard保留，释放自有门闩后沿原物理路径结算。Some继续使用原owned-group控制器，不为None新增Python前置。旧九项失败在精确main Linux日志全部闭合，2161P／0F／13原忽略。

SDK保持框架通用机制、应用产品策略和薄适配边界。缓存源字段由runtime_context到dynamic的四条精确处置由原唯一生成器实现，保留三语公开字段；原生消息和回放身份、ACK后缀并发测试的正确不变量、公开合同及生成物都经适用门禁验证。历史语义观察按可恢复Git版本定位，原task整体输入未知和before动态未知仍保留；原外部工具/schema未修改。冷CI通过真实base上的候选等树视图运行原producer，恢复准确candidate后执行原c0bc Action，无手写预检字段或删减检查。

## 精确风险与实际边界

用户仅接受CLI最终consumer `95b7d02ad130a5bbb886183e9c365d780b36ad35a7f9256163375d42cc744a97`／283输入与原锁 `48eece730daaa9eff539e416861454441c23c54abf02bacaa2c3bdb375bd6329`／769包的原五完整风险。采用后原八控制自然0，两项default fresh正式gate/caller各0，官方scanner各1、0漏洞、仅原五warning。glib0.18.5 unsound及四项unmaintained没有被修复；不接受未来源码、版本或新风险。原网络失败保留，恢复后的正式结果单独记录，未关闭fetch、yanked或添加过滤。

SDK独立接受Framework3413 pin与锁95e3／654包，原49组107duplicate身份仅新增quick-xml0.42.0后为49组108；原四项advisory ignore和严格策略保持。各仓接受互不继承。实际平台、本地合成Host、公开main CI与App／真实Provider／原账号手机分别验收。

## 统一App与人工验收

统一App已用primary Frame3413／CLI233官方冷构建自然0、无警告，4016源码、17工具及5运行闭包前后无漂移；实际frozen LLVM加载、同一次Tauri hook、152个新前端资产、编译dep-info／fingerprint、adhoc签名与strict签名核验均通过，已形成独立ready。签后程序SHA256为`ed081f0800d92a141832c200eea3d52c94423f6ed49c8989c71688052b41ddda`，522611392字节。Agent不启动App、不操作GUI、不读取原配置/MCP/auth/history。交付启动器沿原结构复用原数据，采用直接exec并继承同一个flock，防止两个进程同时写原数据；用户正常退出旧App后手动运行[启动综合同源候选（原数据）](../../../.echo-semantic/local/integrated-acceptance-20261108-r01/integrated-app-r01/启动综合同源候选（原数据）.command)。只检查原路径存在，不读取配置内容，不恢复或复制历史数据。

当前原账号微信无回复的业务根因仍UNKNOWN；手机A/B各一次回复、重连C和保存重开D仍待人工实际验证。界面紧凑行距、左栏宽度、主栏适量留白、顶部和右栏滚动以及/goal、/side已进入同一源码主线，不把合成浏览器或App构建当作原生人工验收。

## 清理与原件

已完成的原共享target、Framework／CLI默认target、SDK最终target及CLI最终target均在必要程序、完整.d／fingerprint、源／工具和真实日志／出口保全、reader0后官方清理准确目录。SDK最终clean报告9.6GiB，CLI最终clean报告8.8GiB；df前后为点时观察，APFS克隆与并发活动使它们不等于物理独占回收计量。原资料和未知私有.eko目录不猜测删除。最终App自有target也已在3600件完整编译元数据、152资源、raw native及签后包hash回读一致、reader0后官方clean自然0，准确目录不存在。签后包和启动器位于target外，清后ready与程序SHA完全保持。

完整本地原件位于主根`.echo-semantic/local/integrated-acceptance-20261108-r01/`及`.echo-semantic/local/dependency-exact-binding-20261109-r02/`；它们不冒作已上传的CI附件。CI实际原件分别见[Framework](https://github.com/EchoYue-lp/echo-agent/actions/runs/42522881689)、[CLI](https://github.com/EchoYue-lp/echo-agent-cli/actions/runs/42559606758)、[SDK](https://github.com/EchoYue-lp/echo-agent-sdk/actions/runs/42552499514)和[官网](https://github.com/EchoYue-lp/echo-website/actions/runs/42563463541)。文档、示例和官网适用同步已随所属仓检查；主工程本记录只记跨仓交付，不建立新的产品行为权威。
