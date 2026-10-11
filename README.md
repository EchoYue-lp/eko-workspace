# EKO Workspace

`eko-workspace` is the Git superproject for the EKO local AI assistant. It keeps the framework, SDK product, application, and website as four independent repositories while pinning a known-compatible commit of each repository in one place.

This repository is not a monorepo or a Cargo workspace. Each submodule keeps its own history, branches, releases, tests, and remote repository.

## Repositories

| Path | Repository | Responsibility |
|---|---|---|
| `echo-agent/` | [echo-agent](https://github.com/EchoYue-lp/echo-agent) | Reusable Rust Agent framework |
| `echo-agent-sdk/` | [echo-agent-sdk](https://github.com/EchoYue-lp/echo-agent-sdk) | Rust Protocol/Host, TypeScript/Python/Java SDKs, accepted contracts, documentation, and CI |
| `echo-agent-cli/` | [echo-agent-cli](https://github.com/EchoYue-lp/echo-agent-cli) | EKO CLI, TUI, desktop application, and frontend |
| `echo-website/` | [echo-website](https://github.com/EchoYue-lp/echo-website) | EKO website and public documentation |

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
