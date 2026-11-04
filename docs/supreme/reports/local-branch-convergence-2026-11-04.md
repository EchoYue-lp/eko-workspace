# 本地验证分支收敛记录

2026-11-04，用户要求在验证阶段删除多余分支，每个项目只留一个用于稳定测试的分支。本次保留各仓 `main` 和当前验证分支，不切换源码或改写主线。当前完整迭代的修复、验证与打包继续使用这一个验证分支。

| 仓库 | 清理前 | 删除 | 清理后 | 保留的验证分支 |
| --- | ---: | ---: | ---: | --- |
| 顶层 eko-workspace | 6 | 4 | 2 | `chore/Echoyue/workspace-maintenance` |
| echo-agent | 22 | 20 | 2 | `chore/Echoyue/root-workspace-convergence` |
| echo-agent-cli | 29 | 27 | 2 | `chore/Echoyue/root-workspace-convergence` |
| echo-agent-sdk | 8 | 6 | 2 | `chore/Echoyue/root-workspace-convergence` |
| echo-website | 6 | 4 | 2 | `fix/Echoyue/website-source-sync` |
| 合计 | 71 | 61 | 10 | 每仓另保留 `main` |

先为每仓制作独立完整 bundle，包含所有本地分支和标签，执行 `git bundle verify`，再导入新的临时 bare repository，逐一核对全部分支尖端提交。恢复核验成功后才删除历史本地分支；临时恢复仓已移除，bundle、分支到提交的对应表及原始执行回执继续保存在唯一主目录内。

完整原件位于 `.echo-semantic/local/local-branch-convergence-20261104-r01/`，总回执 `summary.json` 的 SHA256 为 `4d6e9e03d32287f8afd4aae66f431774a562d4efaab74ed1cde6dcdb9f7c17b3`。恢复包总计约 94 MB，无需把历史分支继续显示在日常分支列表中。未合入的 Exa 等历史成果同样保存在恢复包中，没有借清理将其合并或推送。

各仓实际 HEAD、当前分支、index tree、未提交与未跟踪状态、标签、stash、远端引用和 worktree 列表均在删除前后核对一致。`pre-retirement-2026-10-24` 仍保留；已有宿主 worktree 不属于本次分支清理。当前源码资格与历史验证回执继续绑定原精确提交和源文件摘要，分支引用的清理不等于重新验证或发布。

需要取回某一历史分支时，从对应仓的 `before.json` 获取名称和提交，在该仓使用下列形式恢复即可。恢复操作本次未执行到正式仓：

```bash
git fetch /绝对路径/对应仓/all-local-branches-and-tags.bundle \
  refs/heads/历史分支名:refs/heads/历史分支名
```
