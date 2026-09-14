---
title: echo-agent Eval workspace generation 隔离设计
artifact: design
carrier: markdown
---

# echo-agent Eval workspace generation 隔离设计

## 问题与目标

`EvalRunner::setup_fixture`当前把fixture复制到`workspace_root/case.id`，目标存在时先递归删除；无fixture的case直接把共享`workspace_root`交给Agent。两个并发run使用同一case ID时可以互删，两个无fixture run也可互相污染。`ImprovementLoop`又使用进程全局`temp/improve_i`并在early-stop前break，导致跨loop冲突和稳定残留；`AbComparator`创建UUID parent但不清理。

目标是让每次`EvalRunner::run`拥有唯一workspace generation，fixture、Agent effect与criteria都只访问该generation。已经结算的run必须显式验证cleanup；尚未结算的timeout/caller-drop必须保留隔离目录并交给Issue #48，而不是删除仍可能被producer使用的路径。

## 目标行为

- `EvalRunner::workspace_root`只表示调用方选择的generation父目录，不再直接作为Agent cwd，也不包含case ID权威。
- 每次`run`先在父目录内原子创建随机唯一`EvalWorkspaceGeneration`；有无fixture都使用generation path作为唯一cwd。
- Fixture只复制进当前空generation，不查找、删除或复用同名case目录；重复case ID与并发run彼此隔离。
- Generation创建时默认禁用隐式Drop删除。成功、typed Agent error和criteria完成属于settled路径，显式`close()`并观察删除错误；cleanup失败使EvalResult失败并保留诊断。
- Eval timeout只触发既有cancel并返回失败，generation通过`keep()`保留，Violation包含可清理路径。原因是Issue #48已确认stream producer可能尚未settled。
- 调用方直接drop/cancel `run` future时，generation Drop保留目录并写warning；没有terminal receipt时不得假设可以删除。
- `ImprovementLoop`复用一个以系统temp为父目录的`EvalRunner`，删除`improve_i`命名和手工`remove_dir_all`。Early-stop不再绕过每个run自己的settled cleanup。
- `AbComparator`同样使用系统temp作为generation父目录，不创建无人拥有的`ab_compare_<uuid>` parent。

## 范围与非目标

范围包括`src/eval/runner.rs`的generation authority和cleanup disposition、`src/improve/loop.rs`与`src/eval/comparator.rs`调用方收敛、Eval/Improve测试、双语Eval正式文档、ADR、语义材料与Issue #50。

以下不在本设计内：

- 不修复Issue #48的Turn timeout settlement，也不宣称timeout generation会自动回收。
- 不为eval加入worktree、container、sandbox manager或应用层Workspace策略；这里仅隔离本地临时目录。
- 不改变EvalCase、EvalResult、EvalReport的序列化结构或SDK facade。
- 不并行化`run_all`，也不改变Agent factory、grader、trace与score语义。
- 不修改echo-agent-cli；echo-website继续遵循当前冻结策略，framework docs投影留到统一文档收敛阶段。
- 不处理TrajectorySaver等不属于Eval workspace的临时目录。

## 系统边界

Generation创建、fixture复制、cwd绑定和cleanup disposition是任何echo-agent复用方都需要的通用Eval机制，属于framework。Embedding application只选择父目录和何时调用Eval，不另建case目录authority。

```text
configured workspace_root (parent, never recursively reset)
                    |
                    v
      create unique EvalWorkspaceGeneration
                    |
          +---------+---------+
          |                   |
    copy fixture           empty cwd
          |                   |
          +---------> Agent / criteria
                              |
                +-------------+-------------+
                |                           |
          settled terminal           timeout / caller drop
                |                           |
         explicit close()             keep + diagnostic
         cleanup Result               Issue #48 owns settlement
```

## 核心结构与数据流

私有`EvalWorkspaceGeneration`持有`Option<tempfile::TempDir>`。Builder以固定`eval-`前缀、随机后缀和`disable_cleanup(true)`在父目录创建目录。私有结构只提供path、显式settled close和显式retain；Drop若仍持有TempDir，只warning并让目录保留。

`EvalRunner::run`在任何Agent或fixture effect前创建generation。Fixture setup只接收generation path作为destination。Agent invocation与TestPass/其它criteria共享同一个cwd。Result score与trace处理结束后再选择cleanup disposition；cleanup error进入同一个EvalResult，不建立第二结果对象。

## 异常与边界场景

- Parent不存在：run创建父目录；失败返回fixture/setup violation，不启动Agent。
- Generation创建失败：返回typed EvalResult failure，不回退共享root。
- Fixture不存在或复制部分失败：不启动Agent，显式close generation；setup与cleanup错误都保留。
- 相同case ID并发：两个随机generation都含各自fixture副本，任一close不影响另一目录。
- 无fixture并发：仍使用不同空generation，不把共享parent暴露为cwd。
- Agent成功或typed error：future已返回，完成trace/criteria后显式close；删除失败使结果失败。
- Timeout：cancel后producer未证明terminal，keep generation并记录路径；不运行依赖稳定最终文件的success criteria。
- Caller drop：私有guard的Drop只warning并保留，不递归删除未知live producer路径。
- Early-stop：ImprovementLoop的局部EvalRunner不拥有固定iteration目录，break不影响已完成run的显式cleanup。

## 关键取舍与业界依据

OpenAI Evals的ML Agent benchmark在每个`eval_sample`内创建`TemporaryDirectory`，workspace和logs都位于该sample私有目录。Inspect AI明确每个sample获得独立sandbox instance，避免sample之间互扰；其cleanup合同也单独处理interrupted和禁用cleanup场景。Rust tempfile文档说明TempDir使用随机名称、Drop默认删除但忽略错误，`close()`可观察删除失败，`keep()`可显式保留。

- OpenAI Evals per-sample TemporaryDirectory: <https://github.com/openai/evals/blob/8eac7a7de5215c907fbddc30efdaf316913eccdd/evals/elsuite/hr_ml_agent_bench/eval.py#L61-L85>
- Inspect AI per-sample sandbox: <https://github.com/UKGovernmentBEIS/inspect_ai/blob/b6589d81f449112bb9942b0003b6fd9b54f1c48e/docs/sandboxing.qmd#L176-L183>
- Inspect AI cleanup interruption contract: <https://github.com/UKGovernmentBEIS/inspect_ai/blob/b6589d81f449112bb9942b0003b6fd9b54f1c48e/docs/extensions-sandboxes.qmd#L112-L149>
- tempfile TempDir: <https://docs.rs/tempfile/latest/tempfile/struct.TempDir.html>

按case ID手工拼UUID仍需要自行处理Drop/caller cancellation和删除错误，被排除。所有分支一律RAII自动删除也被排除，因为Issue #48证明timeout producer尚未settled。为timeout新增后台reaper被排除，它需要真实Turn terminal receipt，属于Issue #48而非generation隔离。

## 公共合同与兼容

`EvalRunner::new(PathBuf)`与public字段形状不变；行为合同收紧为“PathBuf是generation parent，每次run的cwd是其唯一子目录”。调用方不应依赖case结束后workspace内容仍在固定`root/case.id`。需要诊断timeout时，从EvalResult violation取得保留路径。

该变化不新增Rust public identity、wire字段或三语言SDK映射。双语Eval文档必须说明parent、per-run隔离、settled cleanup和timeout retention。Website镜像本阶段不改，统一文档投影阶段按source-aware流程同步。

## 复用与实现约束

- 复用现有`tempfile = 3`、`AgentInvocationContext.working_dir`、EvalResult violation/recompute、copy_dir和Issue #48，不新增依赖或状态schema。
- Generation是私有生命周期原语，不成为第二Eval store或public Workspace类型。
- 不再调用`remove_dir_all(workspace_root/case.id)`，不再生成`improve_i`或`ab_compare_<uuid>`。
- Cleanup必须显式区分settled close与unsettled retain；不得用Drop静默吞删除错误。
- 测试不得依赖进程全局TMPDIR修改或固定sleep竞争；使用可控Agent与channels记录cwd/terminal。
- 不使用unwrap、expect、panic、unreachable、todo、字节字符串切片或Worker术语。

## 验收标准

- 同一runner、同一case ID的两个并发run观察到不同cwd和完整独立fixture；完成后两个generation都已删除。
- 两个无fixture run同样获得不同cwd，且共享parent不作为Agent cwd。
- Success与typed error显式cleanup；可控cleanup失败进入EvalResult violation并标记失败。
- Timeout结果包含保留generation path且路径仍存在；测试自行删除，Issue #48保持open。
- ImprovementLoop early-stop与多次并发loop不创建固定`improve_i`，没有成功路径残留；AbComparator不遗留自建parent。
- Eval/Improve既有测试、Clippy、相关feature、双语文档合同、SDK零diff、semantic strict/change-evidence、Issue reconciliation和独立review全部通过。
