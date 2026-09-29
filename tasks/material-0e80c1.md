---
id: material-0e80c1
title: Establish matched-state ring capture feasibility and current shift bounds
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-09-29T22:38:51Z
updated: 2026-09-29T22:38:51Z
depends: []
parent: material-49871a
tags: [harness]
source: "docs/notes/2026-09-29-glass-measurement-brief.md#matched-state-ring"
agent: codex
---

Question: Can the existing headless renderer and frozen-clock fixtures capture ring-on/off at identical resize geometry, client pixels and signal state, and which current light-ior fixtures actually bind the shared shift cap?
Where to start: src/tests/animations.rs::set_time and set_up; src/animation/clock.rs; docs/materials/scripts/focus-ring-light.sh::case_resize_flex; src/render_helpers/shaders/material/main.frag and prelude.frag; material-3db428; docs/notes/2026-09-29-glass-measurement-brief.md.
Bound: Trace the fixture-to-render path and try one matched mid-resize pair with fixed client content, including an identical-state repeat and a deliberately shifted-state negative control. Pin or explicitly freeze the independent focus-beam time. Recompute cap binding for ior 1.02, 1.24 and 1.5 with explicit thickness, bevel, gap and light-ior, including ior 1.24 / bevel 9; these are formula results, not measured pixel displacements. If the existing fixture cannot produce the pair, record the missing seam and smallest proposed change; no production clock IPC, cap redesign, broad sweep or new capture framework. Reuse material-3db428 for old-script geometry corrections rather than duplicating its work.
Expected result: Record reproducibility, all varying state, remaining error and a recommendation on this task and in the brief. Replace the obsolete zero-light-on-translucent-face premise with current slab coverage and opaque-client bypass constraints; propose a quantified motion check without inventing a passing tolerance. Later visual tuning retains owner review.
Ideas it wakes: On completion, run tasks note on material-22d78f and material-a85a18 with the findings, in the same commit as this result, and update docs/notes/2026-09-29-glass-measurement-brief.md.
