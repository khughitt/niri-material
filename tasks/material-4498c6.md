---
id: material-4498c6
title: Correct the backdrop-blur plan path in the material index
status: todo
priority: 1
size: xs
complexity: low
process: direct
created: 2026-10-24T13:48:39Z
updated: 2026-10-24T18:57:15Z
depends: []
tags: [docs]
source: material-10a999
agent: codex
---

Why: docs/materials/README.md:73 names plans/2026-08-29-material-backdrop-blur.md, which does not exist beneath docs/materials. The retained tracked plan is docs/superpowers/plans/2026-08-29-material-backdrop-blur.md.

Done: change that index entry to ../superpowers/plans/2026-08-29-material-backdrop-blur.md and verify it resolves from the index. Do not rewrite the historical plan; material-4faf8a owns its separate machine-layout strings. Use normal documentation checks.
