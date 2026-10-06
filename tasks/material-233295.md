---
id: material-233295
title: Concise performance guide for the material pipeline
status: done
priority: 2
size: s
complexity: low
process: direct
owner: material-233295
created: 2026-09-11T23:34:15Z
updated: 2026-10-06T16:47:07Z
started: 2026-10-06T14:59:39Z
completed: 2026-10-06T15:07:49Z
depends: []
parent: material-5d6b2c
tags: [quick-add, performance, docs]
source: "mindful:thought:a476e6bcd1fd4297b70824758235d821"
---

Write a short guide to where GPU, CPU, and memory go in the material: per-pass costs in the render chain, prefilter pyramids and their cache reuse, per-window vs per-frame work, focused vs unfocused split, which parameters are expensive and why, and the known levers for reducing cost. Keep it concise and reference the evidence docs under docs/materials/ rather than repeating them.

Source: mindful:thought:a476e6bcd1fd4297b70824758235d821

## Notes

- 2026-09-12T04:47:03Z (material-bae9c9): When writing the performance guide, point readers to docs/specs/2026-09-11-material-capture-protocol-design.md for comparable capture provenance and environment metadata.
- 2026-09-12T04:53:15Z (material-bae9c9): The future performance guide should reference docs/specs/2026-09-11-material-capture-protocol-design.md for comparable capture provenance and quietness checks.
- 2026-09-12T19:24:28Z (materials-26.04): Complexity low: The deliverable is a concise synthesis of existing render-pipeline and evidence docs, with the capture-protocol reference identified in the notes; verify each cost claim against those sources and current code.
- 2026-10-06T14:59:39Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:6f0031a6-34a9-4a30-bf8b-47dd178d115f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T14:59:43Z (materials-26.04): process direct: the body fixes the deliverable (a concise guide over existing evidence docs) and the check (each cost claim verified against the docs and current code); no design choice is open.
- 2026-10-06T14:59:52Z (material-233295): resumed
  provenance: {"harness_session":"claude-code:6f0031a6-34a9-4a30-bf8b-47dd178d115f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T15:07:49Z (material-233295): done
  provenance: {"harness_session":"claude-code:6f0031a6-34a9-4a30-bf8b-47dd178d115f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T15:07:49Z (material-233295): docs/materials/performance.md: a concise map of where GPU, CPU, memory and power go (backdrop chain, per-window, per-fragment), redraw sources, focus split, hidden windows, parameter costs and open levers, each claim cited to an evidence doc or verified in code; indexed in the materials README and linked from render-pipeline.md, whose two stale claims (resolve_material per frame, backdrop grain 'one texture per output') are corrected. Follow-ups: material-7f6d0e, material-1debaa.
  provenance: {"harness_session":"claude-code:6f0031a6-34a9-4a30-bf8b-47dd178d115f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T16:42:26Z (materials-26.04): review: impl round 1 — verdict: revise; findings: P2 5, P3 2; reviewer: unknown (pasted into session)
- 2026-10-06T16:47:07Z (materials-26.04): correction to the round 1 review note: reviewer was codex/gpt-6-astra, not unknown
