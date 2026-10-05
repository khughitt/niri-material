---
id: material-cf32e5
title: "Noise placement: a site attribute selecting backdrop, glass, or film grain"
status: doing
priority: 2
size: m
complexity: high
process: planned
owner: material-cf32e5
created: 2026-10-05T01:45:48Z
updated: 2026-10-05T15:21:05Z
started: 2026-10-05T12:22:37Z
depends: [prism-eef38f, material-a00785]
parent: material-3aa1f2
tags: [noise, rendering, cross-project]
agent: claude-code/claude-fable-5-1
plan: docs/plans/2026-10-05-noise-placement.md
---

First device that can move. Give the glass noise node a site attribute (backdrop | glass | film) and implement the two new placements:

- backdrop: grain the sharp effect-buffer texture before the Kawase blur and the prefilter pyramid (src/render_helpers/effect_buffer.rs), cached with the sharp texture. Per output, like blur passes: one amount, no focus split. Pre-blur grain is low-pass filtered, so heavy roughness erases it; that is expected and the prism design (prism-eef38f) records it as a dominance interaction.
- film: grain at the reserved post hook (render-pipeline.md stage 9), over transmitted light plus ring, aurora, and glint. Glass only; opaque client pixels bypass the shader.
- glass: today's behind placement, byte-identical for configs that name no site.

Settle: default (glass), invalidation of blur and pyramids when backdrop grain changes, unsupported-site error, seed coordinates and grain scale per site, and how future noise layers (material-3fcba2) would select a site, without implementing layers.

Prove the look before the contract: compare today's grain with pre-blur grain on one backdrop at several roughness levels. "Cheaper" is a hypothesis: measure static wallpaper, animated backdrop, and parameter dragging separately; backdrop grain adds a texture generation and invalidates the blur and pyramid caches.

The site vocabulary comes from the pipeline schema designed in prism-eef38f (sites, scope, composition law, coverage); do not start before that design is reviewed.

## Notes

- 2026-10-05T12:22:37Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:23285ff0-fcf8-4e52-93eb-5ccb34db3e4c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T13:02:51Z (material-cf32e5): Design approved in conversation (explorable placement; approach: site property on the noise node, grain pass on the shared sharp texture, agreement validation at load, film grain at post). §7.1 offline simulation run: backdrop grain keeps 32/15/8 % (white) and 19/7/5 % (fine) of its sd after 1/2/3 blur passes; glass grain unchanged; no site dropped
- 2026-10-05T13:04:51Z (material-cf32e5): parked (waiting on user, review): Spec review round 1 of docs/specs/2026-10-05-noise-placement-design.md (.worktrees/material-cf32e5), reviewer codex; on acceptance run writing-plans for the renderer plan
  provenance: {"harness_session":"claude-code:23285ff0-fcf8-4e52-93eb-5ccb34db3e4c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T13:20:50Z (material-cf32e5): review: spec round 1 — verdict: revise; findings: P1 2, P2 3; reviewer: codex
- 2026-10-05T13:22:22Z (material-cf32e5): Spec revised for round 1: grain option changes increment commit_counter (fifth in-process test: unchanged glass window re-renders on backdrop reload); every content read of the sharp texture incl. the blur.render draw moves to source(); noise_behind arithmetic kept byte-for-byte, byte identity checked against a baseline binary; roughness fixture at ior 1.5; ring-band ratios derived from the transfer curve (glass ≈ decode'(0.5)·encode'(band), film 1.0)
- 2026-10-05T13:22:22Z (material-cf32e5): parked (waiting on user, review): Spec review round 2 of docs/specs/2026-10-05-noise-placement-design.md (.worktrees/material-cf32e5), reviewer codex; on acceptance run writing-plans
  provenance: {"harness_session":"claude-code:23285ff0-fcf8-4e52-93eb-5ccb34db3e4c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T13:42:06Z (material-cf32e5): review: spec round 2 — verdict: revise; findings: P2 2; reviewer: codex
- 2026-10-05T13:42:45Z (material-cf32e5): Spec revised for round 2: the damage test asserts the effect buffer commit() and the unchanged tile's fingerprint advance across a backdrop-only reload (render_at draws with full damage, so pixels prove nothing); the ring fixture waits ring::run_length / speed so only the rest glow remains; roughness wording: normalized level, pyramid level = max_level * roughness
- 2026-10-05T14:01:58Z (material-cf32e5): Plan written: docs/plans/2026-10-05-noise-placement.md, seven child tasks (material-dcc79f config, 6f299e shader+schema, eab3ae grain pass, 0bed95 in-process tests, 2a1689 docs+smoke+evidence, 40b563 cost, e50969 gate+merge+hand-off); spec marked accepted for planning after round 2 and the owner's go-ahead
- 2026-10-05T14:01:58Z (material-cf32e5): parked (waiting on user, review): Plan review round 1 of docs/plans/2026-10-05-noise-placement.md (.worktrees/material-cf32e5), reviewer codex; on acceptance execute Tasks 1 to 7 natively
  provenance: {"harness_session":"claude-code:23285ff0-fcf8-4e52-93eb-5ccb34db3e4c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T14:06:11Z (material-cf32e5): parked (waiting on user, review): Plan review round 1 of docs/plans/2026-10-05-noise-placement.md (.worktrees/material-cf32e5), reviewer codex; on acceptance execute Tasks 1 to 7 natively
  provenance: {"harness_session":"claude-code:23285ff0-fcf8-4e52-93eb-5ccb34db3e4c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T14:20:07Z (material-cf32e5): review: plan round 1 — verdict: revise; findings: P1 2, P2 8; reviewer: codex/gpt-6-astra
- 2026-10-05T14:25:47Z (material-cf32e5): Plan revised for round 1: precision line before common.frag; cost script owns swaybg by PID (no pkill); Task 1 green alone (ResolvedNoise literals migrated, noise stage owns the parameter, JSON regenerated); film-grain before effect-saturation; coloured test buffer scaled to u32; tests find tiles by material name; smoke on the lib's real API with calibration, assert_about, own ring config (glow 2, rest 2, one speed node); GrainProgram owned by the buffer, lazy compile, retry after invalidation with a table test; cost section limited to measurements incl. material draw at equal workload; full capture matrix; just setup before the baseline build
- 2026-10-05T14:25:47Z (material-cf32e5): parked (waiting on user, review): Plan review round 2 of docs/plans/2026-10-05-noise-placement.md (.worktrees/material-cf32e5), reviewer codex; on acceptance execute Tasks 1 to 7 natively
  provenance: {"harness_session":"claude-code:23285ff0-fcf8-4e52-93eb-5ccb34db3e4c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T14:37:24Z (material-cf32e5): review: plan round 2 — verdict: revise; findings: P2 5; reviewer: codex/gpt-6-astra
- 2026-10-05T14:38:18Z (material-cf32e5): Plan revised for round 2: cleanup_cost carries $? past stop_walls; schema_renders_stably asserts the glass selector and checks omission on slab; fixture output 800 px, regions clipped to the output; -fuzz 0.392157%; shot_twice asserts determinism in every cell
- 2026-10-05T14:38:18Z (material-cf32e5): parked (waiting on user, review): Plan review round 3 of docs/plans/2026-10-05-noise-placement.md (.worktrees/material-cf32e5), reviewer codex; on acceptance execute Tasks 1 to 7 natively
  provenance: {"harness_session":"claude-code:23285ff0-fcf8-4e52-93eb-5ccb34db3e4c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T14:43:20Z (material-cf32e5): resumed
  provenance: {"harness_session":"codex:01a10c84-1457-7892-bb5d-ed0227e1d702","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-05T14:44:00Z (material-cf32e5): review: plan round 3 — verdict: accept; findings: none; reviewer: human
- 2026-10-05T15:21:05Z (material-cf32e5): Tasks 1–4 committed and verified. Nested captures refused quiet preflight; prepare builds, smoke and cost scripts, simulation, gate and review before parking the quiet run. No desktop process changed.
