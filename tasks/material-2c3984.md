---
id: material-2c3984
title: "Design the ring beam: a light inside the glass tracing the face"
status: doing
priority: 2
size: m
complexity: high
process: planned
owner: materials-26.04
created: 2026-09-20T01:26:50Z
updated: 2026-09-20T09:24:58Z
started: 2026-09-20T01:26:50Z
depends: []
parent: material-a76720
tags: [rendering, signals]
agent: claude-code
spec: docs/specs/2026-09-19-ring-beam-design.md
plan: docs/plans/2026-09-19-ring-beam.md
---

Owner review of the live within ring and focus sweep (2026-09-19): the band reads as a flat bevel, the face floats above it, and the sweep flickers. Intent: a beam of coloured light moving through the glass, inside it and shaped by it, tracing the face with a gap, at a constant walking pace; then a quiet resting glow. Brainstormed 2026-09-19: A/A/A+C/A/1 (beam then quiet ring; px/s speed; comet with long tail; gap from the face edge; arc-length parametrization). Supersedes the sweep's pattern and ring-inset; subsumes material-9306b5's placement question.

## Notes

- 2026-09-20T01:26:50Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:49261570-0755-4b4b-ac00-f6343337242c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-20T01:30:09Z (material-2c3984): Spec written: docs/specs/2026-09-19-ring-beam-design.md — beam then quiet ring, ring-beam-speed px/s (replaces ring-sweep-ms), ring-gap from the face edge (replaces ring-inset), ring-glow, arc-length comet with edge spill, every gain starts a beam. Awaiting owner review.
- 2026-09-20T01:39:10Z (material-2c3984): Spec review 2026-09-19: four findings applied — P and L read per frame so resizes, gap reloads and jelly never strand or overrun a run (head never repositioned); tail tapered to zero at L with zero slope; circular head distance symmetric across the seam; rest defined as head=0, env=0 with the tail drain distinct in tests. Corrections: 24 s / 1440 frames example; shaders/mod.rs uniform registration _2f→_3f. Added the 'splash' envelope as a named alternative for the sheets and a pace pass at 900 px/s.
- 2026-09-20T01:42:27Z (material-2c3984): Spec review 2 (2026-09-19): tail expression wholly inside the 0<behind<L branch (float32 exp overflow → NaN at rest otherwise); Rust beam_perimeter takes the jelly-scaled face so timing and rendering share one P; envelope split into head cutoff (env, exactly 0 after the lap) and shared decay (plateau 1 / splash (1−t/run)²) — mat_sig_focus becomes vec4.
- 2026-09-20T01:53:39Z (material-2c3984): Plan: docs/plans/2026-09-19-ring-beam.md, six steps material-be2e27 → f2579f → e31d0d → 38bc81 → 19f0e9 (Prism) → 912dab. Awaiting review.
- 2026-09-20T09:24:58Z (material-2c3984): Plan review 2026-09-20: eight findings applied (GLSL uniform declarations; completion decided in material_dynamics on the frame's geometry with a 120 s backstop; face from the rendered MaterialFrame and the element's fitted radius; left edge from the bottom-left radius; head/tail split tests and the TL-arc end sample; bevel 0 carries the ring; Prism held on its branch until rollout; Prism depends on material-e31d0d). Task order: ring.rs first as its own commit; tests via just.
