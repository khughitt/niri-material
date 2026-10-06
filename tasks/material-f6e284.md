---
id: material-f6e284
title: Cost entry as part of adding an optic
status: todo
priority: 2
size: s
complexity: low
process: direct
created: 2026-09-11T23:39:09Z
updated: 2026-10-06T20:11:18Z
depends: []
parent: material-5d6b2c
tags: [performance, docs]
---

Why: New optics already need frame-cost proof, but docs/materials/adding-an-optic.md does not define a comparable baseline or a cost-entry shape. Make the existing evidence requirement repeatable rather than introduce a new cost collector.

Done:
- Extend the Documentation and proof section of docs/materials/adding-an-optic.md with a compact cost-entry template and the capture-protocol/performance-guide links.
- Require matched neutral/previous-baseline and enabled configurations, exact stimulus and observation duration, output/window sizes, renderer/host, source and binary identity, capture.json/evidence references, units, absolute and relative deltas, repeat counts and observed variation or resolution limits.
- Distinguish per-call GPU timings from summed work per stimulus and redraw cadence. Document shared blur/grain/prefilter attribution; report power only when a dedicated capture actually measured it. Label unsupported CPU, memory or power claims as unmeasured.
- Keep existing non-zero visual and capture-preflight verdicts. Cost deltas are recorded for review; a numeric regression gate remains deferred until material-31074f supplies repeatability and an accepted budget.
- Link one existing evidence example (such as docs/materials/2026-10-05-noise-placement-evidence.md#tracy-costs) and check that its baseline, units and provenance can populate the template without inventing measurements. No new capture is needed for this documentation task.

Where to look: docs/materials/adding-an-optic.md section 4; docs/materials/performance.md sections 2, 3 and 9; docs/specs/2026-09-11-material-capture-protocol-design.md; docs/materials/scripts/glass-optic-smoke-lib.sh; docs/notes/2026-09-29-resource-aware-rendering-brief.md.

Verification: review the filled existing-evidence example and all links, run tasks check and the repository documentation commit gate. Approach and check are established, so use direct process.

## Original idea

Extend docs/materials/adding-an-optic.md so every new optic or pass ships with a cost measurement under the capture protocol (material-bae9c9) and a comparison against the previous baseline. Makes the resource-aware goal a habit rather than a one-off audit; decide whether a threshold breach should block or merely be recorded.

## Notes

- 2026-10-06T20:11:17Z (materials-26.04): scope: scoped; P2/s/low/direct documentation task for a repeatable optic-cost entry using existing capture evidence; record deltas and uncertainty before any numeric threshold; brief: docs/notes/2026-09-29-resource-aware-rendering-brief.md
