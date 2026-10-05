---
id: material-a00785
title: "Pipeline schema: sites, stages, and interactions as data, pinned to the code and generated to a file"
status: doing
priority: 2
size: m
complexity: mid
process: direct
owner: material-a00785
created: 2026-10-05T09:36:02Z
updated: 2026-10-05T10:15:28Z
started: 2026-10-05T09:36:47Z
depends: []
parent: material-3aa1f2
tags: [material, cross-project, docs]
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
