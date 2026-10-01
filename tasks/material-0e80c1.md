---
id: material-0e80c1
title: Establish matched-state ring capture feasibility and current shift bounds
status: doing
priority: 2
size: s
complexity: high
process: direct
owner: materials-26.04
created: 2026-09-29T22:38:51Z
updated: 2026-10-01T07:58:11Z
started: 2026-10-01T07:52:48Z
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

## Notes

- 2026-10-01T07:52:48Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:601fb2a6-cb37-47f5-a964-59f3bb9f54af","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T07:58:00Z (materials-26.04): Offline trace (read-only subagent, 2026-10-01), formula results not measured displacements: shift cap main.frag:114-121 (depth 0.2*thickness, ior 1+(ior-1)*light-ior, cap 0.5*gap); chamfer normal rise=min(bevel,thickness) so bevel only matters when bevel>thickness. Cap binds for render-order within-dense (1.5/80/12/gap 5/lior 6: 9.14 vs 2.5 px), live Prism active (1.28/31.2/10/gap 2: 3.09 vs 1.0) and inactive (1.22/75.3/10/gap 2: 6.90 vs 1.0), ior 1.24/thickness 43.3 at light-ior>=6 (gap 8: 4.09/4.91 vs 4); not for the ior 1.02 pins (0.16/0.85/1.48 vs 2.5) nor stock 1.5/20/12/gap 8 (2.28 vs 4). Opaque bypass main.frag:13-16 is exact (alpha 1.0); ring light over the face scales with (1 - client alpha) (main.frag:223), zero only on fully opaque pixels. focus-ring-light.sh pins no light-ior/jelly/noise (inherits Prism 6/0.0066/0.23/0.03); its case_tiny comment cites the old slabChamfer gate (now hasLine, main.frag:112-113). Route B (no code): freeze a nested host with animations { slowdown 2147483647; } via load-config-file.
- 2026-10-01T07:58:00Z (materials-26.04): In-process Route A (.worktrees/material-0e80c1 a9f912a5, src/tests/ring_pair.rs): headless surfaceless GLES renders the material (tile material and program present) at a set_time-frozen mid-resize instant; identical-state repeat byte-equal (0 px) and a 10 ms shifted control differs (720 px, one edge column, max 191) at both stock and cap-binding geometry. Ring on vs off renders identically (0 px) even with update_keyboard_focus and refresh_and_flush_clients after each response-only reload, so the ring is not lit in this render yet and the pair is not measured.
- 2026-10-01T07:58:00Z (materials-26.04): parked (waiting on agent, capability): Agent, in .worktrees/material-0e80c1: find why focus "ring-light" draws nothing in ring_pair.rs's in-process render (dump on1/off as PNG; check the tile's focus/ring state and whether the response reload reaches MaterialState), then record the pair's on-off light map at stock and cap-binding geometry, write the brief update and close (note material-22d78f, material-a85a18). No quiet host needed.
  provenance: {"harness_session":"claude-code:601fb2a6-cb37-47f5-a964-59f3bb9f54af","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T07:58:11Z (materials-26.04): parked (waiting on agent, session): Agent, in .worktrees/material-0e80c1: find why focus "ring-light" draws nothing in ring_pair.rs's in-process render (dump on1/off as PNG; check the tile's focus/ring state and whether the response reload reaches MaterialState), then record the pair's on-off light map at stock and cap-binding geometry, write the brief update and close (note material-22d78f, material-a85a18). No quiet host needed.
  provenance: {"harness_session":"claude-code:601fb2a6-cb37-47f5-a964-59f3bb9f54af","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
