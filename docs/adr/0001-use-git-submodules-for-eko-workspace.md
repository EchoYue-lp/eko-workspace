# ADR 0001: Use Git Submodules for the EKO Superproject

- Status: Accepted
- Date: 2026-08-24

## Context

EKO is developed across three repositories with different ownership boundaries: the reusable `echo-agent` framework, the `echo-agent-cli` application, and `echo-website`. They need shared project instructions, cross-repository architecture documentation, and a reproducible way to identify compatible revisions without losing their independent histories and release workflows.

Before this decision, the three repositories happened to live under one local directory, while the shared `AGENTS.md` and `docs/` content had no parent Git repository. Directory co-location alone could not reproduce the complete workspace on another machine or pin a compatible set of revisions.

## Considered Options

### Symbolic links

A new repository could contain symbolic links to separately cloned sibling directories. Git would only version the link targets, not the child repository revisions. Absolute links would be machine-specific, and relative links would still require an undocumented external directory layout. This option does not provide reproducible clones or CI setup.

### Merge the repositories into a monorepo

A monorepo would allow atomic cross-project commits, but it would combine histories, branching, permissions, releases, and CI boundaries. That is a much broader migration and would weaken the explicit separation between the reusable framework, the EKO application, and the website.

### Git superproject with submodules

Git submodules preserve each child repository while allowing a parent repository to record its URL, path, and exact commit. This matches the current ownership model and makes the complete workspace reproducible.

## Decision

Use `eko-workspace` as a Git superproject with these submodules:

- `echo-agent`
- `echo-agent-cli`
- `echo-website`

The superproject owns only cross-project concerns: `AGENTS.md`, shared architecture and planning documents, workspace onboarding, and pinned child revisions. Product code, examples, repository-specific documentation, tests, and releases remain in their owning child repositories.

Local agent state, generated worktrees, `agent-browser`, and the local
`todolist.md` are outside this superproject and are ignored explicitly.

## Tradeoffs

Benefits:

- A recursive clone reconstructs the complete EKO workspace.
- Parent commits pin a known-compatible revision of every child repository.
- Child histories, releases, CI pipelines, and framework/application boundaries remain independent.
- Existing relative path dependencies between `echo-agent-cli` and `echo-agent` continue to work.

Costs:

- Cross-repository changes require child commits followed by a parent revision commit; they are not atomic.
- Contributors must initialize and update submodules explicitly.
- A freshly initialized submodule can be in detached-HEAD state, so contributors must select the intended branch before editing.
- Updating a child branch does not update the parent revision automatically.

## Impact

- Clone with `git clone --recurse-submodules` or run `git submodule update --init --recursive` after cloning.
- Commit and push changes in each child repository before updating its pointer in `eko-workspace`.
- Keep framework features in `echo-agent`, EKO product policy in `echo-agent-cli`, and public website content in `echo-website`; the superproject does not become another implementation layer.
- This repository-structure change does not modify product code or public behavior, so no `examples` or `echo-website` content changes are required for this decision.
