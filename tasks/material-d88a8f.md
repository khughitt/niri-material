---
id: material-d88a8f
title: "Design: match window rules on signal level"
status: shelved
priority: 2
size: s
created: 2026-09-02T12:09:35Z
updated: 2026-10-19T15:37:57Z
depends: [material-a54d89]
parent: material-b5cbd6
tags: [signals, design]
---

Outcome: decide whether window-rule Match gains signal-level="demand" style matching in addition to signal-source and signal-tag, and what recompute cost it implies. Deferred from the v1 slice because is-urgent covers the native case. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

## Notes

- 2026-09-29T22:31:42Z (materials-26.04): scope: briefed; signal mutations already invalidate window rules; raw-fold versus rendered-level semantics, absent-versus-Quiet matching and a concrete rule use case remain undecided; brief: docs/notes/2026-09-29-signal-model-extensions-brief.md
- 2026-10-19T15:37:57Z (materials-26.04): shelved: A concrete window rule needs to select on external signal level: an appearance that existing responses, signal-source/signal-tag matches and is-urgent cannot express
