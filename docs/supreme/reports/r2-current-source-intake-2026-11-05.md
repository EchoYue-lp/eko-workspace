# R2按当前主根接收的差量准备

日期：2026-11-05。精确风险绑定仍待用户答复，本次只推进Goal第3项中可独立进行的源码核对，
不修改正式源码／锁／policy，不编译或启动HTTP／Popen／GUI／180秒预算。

## 当前发现与处置

R2专用 `r2_appcore_case_tests.rs` 自身没有temperature字段，`ConfiguredModel` 构造通过
`..Default::default()` 接收当前类型；旧完整组合的 `infra/tests.rs` 则仍有12处temperature引用。
不能因旧完整副本需要迁移，就对专用case删除不存在的字段或整文件回灌旧测试。

原AppCore四文件补丁在当前CLI上 `git apply --check` 自然0，但正式源尚无
`r2_observation` 等候选观察器，因此补丁可应用不等于可编译。正确后继是接收既有观察器／
运行时／Provider及AppCore依赖的最小差量，保留当前主根的publication、温度退役和0.38升级。

## 已形成的可审差量

在内存中按7份原已审补丁的精SHA依次组合，输出两份当前源码补丁，总计27个文件：

| 仓库 | 差量文件 | SHA256 | 实际适用检查 |
| --- | --- | --- | --- |
| Framework | `echo-agent-current-intake.patch`，195092字节 | `f795024f1e71835d4ba773aae6d4fd714bffb18729ecd5da0326df29eeeab687` | 0 |
| CLI | `echo-agent-cli-current-intake.patch`，148867字节 | `de081b8d65e9934f3f163e19be07b9296bc782a8e3380723e9f399cab7824b18` | 0 |

两份补丁及输入清单在根 `.echo-semantic/local/r2-current-source-intake-20261105-r01/`。
组合不写源码树，不创建新worktree／分支或完整源码副本；各新模块与原已审combined3951的
对应模块逐字节一致。原holder和Python8脚本不搬动、不执行，原9953 Model／627e gate与
T0／D／W／H／S／C规则不变。没有回灌旧publication、旧锁、旧vendor或旧infra测试文件。

对27个受影响路径回读，正式旧文件仍为组合前SHA，新模块仍未出现在正式树；
两份最终补丁在真实仓库再次适用检查各0。Framework当前8fb锁、CLI48ee锁和SDKc448锁不改，
正式风险baseline／policy仍保持原待裁决状态。当前接收只是准备，不是正式采用。

独立审查已pass、零findings：7份输入、27个文件前后SHA及输出补丁逐字节一致；
6个新增模块与原已审组合相同，当前Responses温度回归和publication WIP保留。
此审查只签源码准备，未运行编译或case。

## 验证边界与下一步

本次没有Cargo／metadata／实际list，也没有Final、Terminal、sourceEOF、物理join或cold运行证据。
未来接收会引入原候选的 `echo_integration → hyper-util` 边，仍须真实resolver与精确风险复核；
不得把当前0.38锁的资格转授R2，也不得用旧2140／6df锁或旧NoRun重标当前通过。

后继沿原精确设计33ac625和Plan dfcf7ea，先核当前组合的类型／新程序／具名selector，
再运行原holder及180秒case；`child_tool_effect_once=Unknown/not_reached` 和GUI／B缺口保持。
编译和运行依赖前置资格闭合后推进，不因自动Goal续接推断用户已同意精确风险绑定。

本次仅工程验收差量，没有产品API／examples／官网行为变化，正式对外说明无需新增。
原件：`composition.json`、`current-source-readback.json` 及两份补丁；检查证明局限于源码准备。
