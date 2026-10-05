---
id: material-a00785
title: "Pipeline schema: sites, stages, and interactions as data, pinned to the code and generated to a file"
status: todo
priority: 2
size: m
complexity: mid
process: direct
created: 2026-10-05T09:36:02Z
updated: 2026-10-05T09:36:02Z
depends: []
parent: material-3aa1f2
tags: [material, cross-project, docs]
agent: claude-code
---

Renderer side of the pipeline schema design, prism docs/specs/2026-10-04-pipeline-schema-design.md (Section 2), accepted 2026-10-05 after three review rounds. Plan: docs/plans/2026-10-05-pipeline-schema.md in this repository; the prism side (vendored copy, rack validation, panel) is the prism plan of the same name and starts after Task 3 here produces resources/materials/pipeline.json.

Deliverables: niri-config/src/material/pipeline.rs with SITES, STAGES, INTERACTIONS statics and response_fields() beside Response; tests pinning them to all_params(), ORDER, the response block, and per-program hook calls in the shaders; material_pipeline_schema_matches_the_file generating resources/materials/pipeline.json under MATERIAL_DOCS_UPDATE=1; one paragraph in render-pipeline.md and one step in adding-an-optic.md. No behaviour change in the renderer.

Direct: the spec settles every table value and test; the remaining choices are serde derive placement and test file layout.
