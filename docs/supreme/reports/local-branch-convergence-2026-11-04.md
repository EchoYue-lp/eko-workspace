# 已撤销的本地分支清理记录

**当前状态：下述61条分支删除已全部撤销。** 用户指出清理前没有证明代码已经吸收，也没有判断旧迭代是否仍值得合入；Root承认将“可恢复”误当成“可以删除”的依据。五仓本地分支已恢复为71条，61条原尖端逐一与删除前SHA一致，HEAD、当前分支、index和WIP不变。恢复回执位于原件目录的 `restore-all61-after-user-objection-20261104-r01/summary.json`，SHA256 `f2ec0be279a447b9cf9e17553b35e953615ba74e388224c0648e9aadef06b638`。原删除表及恢复包作为操作历史保留，不能作为当前已收敛状态或已合入证明。

当前只读核验初步确认44条尖端属于当前验证分支或本地main祖先；另外17条需按当前源码、squash/重写、后继优化与具名退役分别判断。它们均已恢复，不机械合并、不再次批量删除。分支引用删除本身几乎不释放磁盘；磁盘占用另按构建缓存、验证源码副本、保全程序及其它目录实际盘点。

以下记录2026-11-04最初执行的清理过程，随后已按上段完整恢复。用户希望在验证阶段使用统一测试分支，但这一偏好不授权跳过合入与保留价值判断。

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
