---
id: material-e0d6b1
title: "Drop tools/identity-check: the global identity guard covers it"
status: todo
priority: 3
size: xs
complexity: low
process: direct
created: 2026-10-17T09:56:22Z
updated: 2026-10-17T09:56:22Z
depends: []
tags: []
source: ops-4c9eb4
agent: claude-code/claude-opus-5-5
---

ops bin/git-identity-check, run from a global git hook on every commit, refuses a repo-local user.* that differs from the profile identity (ops docs/specs/2026-10-16-commit-identity-guard-design.md). Remove tools/identity-check and its check-chain entry once the hook is live on every host.
