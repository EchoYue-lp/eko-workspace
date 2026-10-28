# Eclip Workspace /ˈɛk.lɪp/

`eko-workspace` 是 Eclip 本地个人 AI 助理的顶层 Git 仓库。**Eclip /ˈɛk.lɪp/** 读作 **EK-lip**，两个音节。**听见你的声音，让想法飞跃。**

本仓库统一固定四个独立子仓库的已知兼容提交；应用的品牌与图标说明由 [产品仓库](echo-agent-cli/design/branding/README.md) 长期维护。

This repository is not a monorepo or a Cargo workspace. Each submodule keeps its own history, branches, releases, tests, and remote repository.

## Repositories

| Path | Repository | Responsibility |
|---|---|---|
| `echo-agent/` | [echo-agent](https://github.com/EchoYue-lp/echo-agent) | Reusable Rust Agent framework |
| `echo-agent-sdk/` | [echo-agent-sdk](https://github.com/EchoYue-lp/echo-agent-sdk) | Rust Protocol/Host, TypeScript/Python/Java SDKs, accepted contracts, documentation, and CI |
| `echo-agent-cli/` | [echo-agent-cli](https://github.com/EchoYue-lp/echo-agent-cli) | Eclip 桌面应用、共享应用核心与前端（旧终端入口处于退役迁移期） |
| `echo-website/` | [echo-website](https://github.com/EchoYue-lp/echo-website) | echo-agent 与 Eclip 官网及对外文档 |

Cross-project instructions and architecture documents live in `AGENTS.md` and `docs/`.

The SDK Host pins its own immutable framework revision. Its build pin is independent of the
framework gitlink in this workspace; ordinary framework development does not run SDK gates.
Only accepted cross-language contracts define SDK compatibility. The complete Rust public
inventory is non-blocking drift telemetry.

## Clone

Clone the superproject and its pinned submodule revisions together:

```bash
git clone --recurse-submodules git@github.com:EchoYue-lp/eko-workspace.git
cd eko-workspace
git submodule status
```

If the repository was cloned without `--recurse-submodules`, initialize it with:

```bash
git submodule update --init --recursive
```

## Development Workflow

Make and verify changes inside the owning submodule. Commit and push that repository first, then record its new commit in the superproject:

```bash
git -C echo-agent status --short
git -C echo-agent-sdk status --short
git -C echo-agent-cli status --short
git -C echo-website status --short

git add echo-agent echo-agent-sdk echo-agent-cli echo-website
git -c commit.gpgsign=false commit -m "chore: update workspace revisions"
git push
```

Only stage submodule paths that intentionally changed. A superproject commit records exact child commit IDs; it does not include or replace child repository commits.

After pulling a superproject update, synchronize the checked-out submodules with the recorded revisions:

```bash
git pull --ff-only
git submodule update --init --recursive
```

Freshly initialized submodules may be in detached-HEAD state. Switch to the intended child branch before starting development, but do not advance a pinned revision implicitly.

## Architecture

The repository-composition decision and its tradeoffs are recorded in [ADR 0001](docs/adr/0001-use-git-submodules-for-eko-workspace.md).
