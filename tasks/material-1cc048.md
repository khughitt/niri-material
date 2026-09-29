---
id: material-1cc048
title: "Design: per-kind impulse envelope shapes"
status: shelved
priority: 2
size: s
created: 2026-09-02T12:09:35Z
updated: 2026-09-29T22:31:40Z
depends: [material-a54d89]
parent: material-b5cbd6
tags: [signals, design]
---

Outcome: decide whether Ping, Done, and Error need distinct attack and decay constants, and expose them if so. The v1 solver uses one shape for all kinds. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

## Notes

- 2026-09-29T22:31:40Z (materials-26.04): shelved: A repeatable visual comparison of Ping, Done and Error shows a specific usability problem with the shared 80 ms attack / 350 ms decay / 1.5 s lifetime, and identifies which kind needs different timing.
- 2026-09-29T22:31:40Z (materials-26.04): scope: shelved; per-kind selectors already differ, but no timing shortcoming is recorded; wake on a repeatable visual comparison identifying a shared-envelope usability problem; brief: docs/notes/2026-09-29-signal-model-extensions-brief.md
