---
id: material-7f85b8
title: Stop vendored tool re-copies from staling the upstream divergence report
status: todo
priority: 2
created: 2026-10-01T23:11:35Z
updated: 2026-10-01T23:11:35Z
depends: []
tags: [tooling]
agent: claude-code/claude-opus-5-5
---

Re-copying tools/tt from ops changes its +N/-0 row in docs/materials/upstream-divergence.md (class C files are listed with line counts), so the pre-commit upstream-report --check fails until `just upstream-report` is rerun and staged. Class A was already made count-only for the same reason. Options: list class C paths without line counts, or have the re-copy step regenerate the report.
