---
id: material-a00785
title: "Pipeline schema: sites, stages, and interactions as data, pinned to the code and generated to a file"
status: done
priority: 2
size: m
complexity: mid
process: direct
owner: material-a00785
created: 2026-10-05T09:36:02Z
updated: 2026-10-05T11:43:01Z
started: 2026-10-05T09:36:47Z
completed: 2026-10-05T11:43:01Z
depends: []
parent: material-3aa1f2
tags: [material, cross-project, docs]
model: claude-opus-5-5
agent: claude-code
plan: docs/plans/2026-10-05-pipeline-schema.md
---

Renderer side of the pipeline schema design, prism docs/specs/2026-10-04-pipeline-schema-design.md (Section 2), accepted 2026-10-05 after three review rounds. Plan: docs/plans/2026-10-05-pipeline-schema.md in this repository; the prism side (vendored copy, rack validation, panel) is the prism plan of the same name and starts after Task 3 here produces resources/materials/pipeline.json.

Deliverables: niri-config/src/material/pipeline.rs with SITES, STAGES, INTERACTIONS statics and response_fields() beside Response; tests pinning them to all_params(), ORDER, the response block, and per-program hook calls in the shaders; material_pipeline_schema_matches_the_file generating resources/materials/pipeline.json under MATERIAL_DOCS_UPDATE=1; one paragraph in render-pipeline.md and one step in adding-an-optic.md. No behaviour change in the renderer.

Direct: the spec settles every table value and test; the remaining choices are serde derive placement and test file layout.

## Notes

- 2026-10-05T09:36:47Z (material-a00785): started
  provenance: {"harness_session":"claude-code:23285ff0-fcf8-4e52-93eb-5ccb34db3e4c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T09:36:47Z (material-a00785): halt override: started past material-cd7782 by 23285ff0-fcf8-4e52-93eb-5ccb34db3e4c: plan document and task records only, no renderer code this session; independent of the pre-commit latency incident material-cd7782
- 2026-10-05T09:45:27Z (material-a00785): parked (waiting on user, review): Plan review with prism-eef38f (the prism plan names the same gate); on acceptance execute Tasks 1 to 3 here, then hand resources/materials/pipeline.json to prism's Task 1
  provenance: {"harness_session":"claude-code:23285ff0-fcf8-4e52-93eb-5ccb34db3e4c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T10:00:34Z (material-a00785): review: plan round 1 — verdict: revise; findings: Important 3; reviewer: codex
- 2026-10-05T10:01:06Z (material-a00785): Plan review details: (1) lines114-142 response_fields_decode_and_count compares only two handwritten lists, not Response fields. Adding an optional struct field while leaving both lists unchanged passes. Tie coverage to the actual struct and demonstrate the omitted-new-field failure. (2) lines190-205 selectors check only present parameter/variant groups, so absent enum variants and different optics across variants pass. Validate the full variant set exactly once and one optic per selector parameter; include negative fixtures because current stages have no selectors. (3) lines482-499 hook checks use find/contains and accept commented-out calls or duplicate calls; other-program checks also accept definitions without calls. Require exactly one actual call, ignoring comments/definitions, and prove removal/comment, duplicate, and reorder mutations fail. No unsafe host actions found. Native execution with a fresh whole-branch review per repository is recommended after plan corrections.
- 2026-10-05T10:15:28Z (material-a00785): resumed
  provenance: {"harness_session":"claude-code:23285ff0-fcf8-4e52-93eb-5ccb34db3e4c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T10:16:09Z (material-a00785): Plan revised for review round 1: response_fields_match_the_struct destructures Response exhaustively (compile-time coverage, proven in Step 4b); check_selectors() enforces every variant exactly once and one optic, with negative fixtures; hook pins require exactly one real call per program, comments stripped and definitions excluded, with three drift demonstrations
- 2026-10-05T10:16:09Z (material-a00785): parked (waiting on user, review): Re-review docs/plans/2026-10-05-pipeline-schema.md (.worktrees/material-a00785) after round 1 corrections, with the prism plan; on acceptance execute Tasks 1 to 3 natively
  provenance: {"harness_session":"claude-code:23285ff0-fcf8-4e52-93eb-5ccb34db3e4c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T10:20:57Z (material-a00785): review: plan round 2 — verdict: revise; findings: Important 1, Minor 1; reviewer: codex
- 2026-10-05T10:21:15Z (material-a00785): Plan round 2 details: selector completeness/one-optic checks and exact non-comment hook calls resolve original findings. Exhaustive Response destructuring also catches the originally requested mutation (new field, unchanged metadata/tests); no stronger macro-based mechanism is required for acceptance. Minor: lines115-117 overclaim that editing the destructuring also forces live metadata equality; describe it as a compile-time review tripwire, since the pattern and live list are independent. Important: Step4b line541 suggests git checkout -- material/mod.rs before Step5 stages Task1 changes. It would erase pub mod pipeline and response_fields() as well as the probe. Remove only ring_probe with a targeted edit, or restore an exact pre-probe snapshot, and verify the intended Task1 diff remains. Fix probe cleanup before execution.
- 2026-10-05T10:43:22Z (material-a00785): resumed
  provenance: {"harness_session":"claude-code:23285ff0-fcf8-4e52-93eb-5ccb34db3e4c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T10:43:25Z (material-a00785): Plan revised for review round 2: Step 4b cleanup deletes only the probe line; exhaustive destructuring described as a compile-time review guard, not an automatic list update
- 2026-10-05T10:43:25Z (material-a00785): parked (waiting on user, review): Re-review the renderer plan (.worktrees/material-a00785/docs/plans/2026-10-05-pipeline-schema.md) after round 2; on acceptance execute Tasks 1 to 3 natively
  provenance: {"harness_session":"claude-code:23285ff0-fcf8-4e52-93eb-5ccb34db3e4c","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T10:49:55Z (material-a00785): review: plan round 3 — verdict: accept; findings: none; reviewer: codex
- 2026-10-05T10:49:55Z (material-a00785): Scoped re-review of 43e6ff90: response probe cleanup removes only its added line, preserving Task1 changes; exhaustive destructuring is accurately described as a compile-time review guard. Prior plan findings remain resolved. Accepted for native execution first, with a fresh whole-branch review before integration.
- 2026-10-05T11:33:32Z (material-a00785): review: impl round 1 — verdict: revise; findings: Important 1, Minor 3; reviewer: claude-code/opus
- 2026-10-05T11:33:32Z (material-a00785): Impl review fix: spec Section 1 rows disagree with the signal code (glass_signal_inputs). Flash boosts fringing/distortion/tap count; ripple raises jelly activity (ripple normal, ring glow); accent presence re-tints. Tables now mark distortion, fringing, directional-blur, tint animated; ping/done/error on distortion, ripple, fringing, directional-blur, ring; fringing and directional-blur read backdrop-blur and roughness (taps sample the prefilter). Pinned by pipeline_signal_driven_stages_are_declared (niri) and taps_stages_read_the_prefilter_controls (config). The prism spec Section 1 needs the same correction before prism vendors version 1.
- 2026-10-05T11:39:41Z (material-a00785): review: impl round 2 — verdict: accept; findings: Minor 3; reviewer: claude-code/opus
- 2026-10-05T11:43:01Z (material-a00785): done
  provenance: {"harness_session":"claude-code:1bcbf2fb-4f36-4e05-91c0-2ceace0817f5","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-05T11:43:01Z (material-a00785): pipeline.rs tables (11 sites, 19 stages, 1 interaction) pinned to all_params(), ORDER, Response, shader hook calls, optic GLSL reads and glass_signal_inputs; resources/materials/pipeline.json generated; impl review fixed five spec rows against the signal code (prism spec Section 1 still needs the same correction); just gate green
  provenance: {"harness_session":"claude-code:1bcbf2fb-4f36-4e05-91c0-2ceace0817f5","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
