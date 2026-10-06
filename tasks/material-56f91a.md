---
id: material-56f91a
title: Determine the smallest embedded reading-texture proof and resource requirements
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-10-06T20:04:11Z
updated: 2026-10-06T20:04:11Z
depends: []
parent: material-3aa1f2
tags: [rendering]
source: "docs/notes/2026-10-06-material-texture-composition-brief.md#texture-feasibility"
agent: codex
---

Question: Can existing backdrop sampling support the embedded reading-texture goal, and what additional material-local texture behavior is actually required?
Where to start: docs/notes/2026-10-06-material-texture-composition-brief.md; material-05c901 and material-599548; docs/materials/render-pipeline.md; niri-config/src/material/mod.rs::ResolvedMaterial and pipeline.rs; src/render_helpers/shaders/material/prelude.frag::sampleBackground/tap; src/render_helpers/material/mod.rs::draw; src/render_helpers/effect_buffer.rs; src/tests/ring_pair.rs and the test client's shm/layer helpers. Read the existing shared-asset task records ops-c8b158, ops-1b5a9c and forge-e0743c, and related mind6-5c0193/mind6-b4d838, for ownership and consumer questions; do not infer an implemented provider from those ideas.
Bound: Trace one existing background texture through sampling/refraction and compare that route with one proposed material-local paper/stone texture. Identify coordinate anchoring, physical/sample depth, alpha/linear-light composition, readability through client transparency, reload/resize/scale behavior, and whether cache sharing remains valid. Recommend the smallest frozen-pixel proof that distinguishes background wallpaper from a genuinely embedded reading layer; specify an existing or generated input with reproducible provenance, but do not select the user's preferred aesthetic. Produce the fixture proposal and minimum consumer requirements, not a production optic, image loader, new sampler, generic layer stack, asset service, shared identity vocabulary, live desktop experiment or timed/power capture.
Expected result: Record one supported recommendation, exact remaining pipeline boundary and a bounded proof proposal on this task and in the brief. Describe the consumer's minimal resource needs (for example decoded 2D image, color/alpha semantics, dimensions, addressing and reload/cache lifetime) so existing provider work can evaluate them. Distinguish internal-glass/behind-client coverage from changing opaque text; if content treatment is necessary, coordinate with material-d257d9 rather than duplicating it. State which visual/placement choices require an owner-reviewed artifact before design or implementation, and defer shared registry integration until a provider contract is established. No assumption that noise layers imply a general material compositor.
Ideas it wakes: On completion, run tasks note on material-05c901 and material-599548 with the finding, in the same commit as this result; update the brief. Report any concrete second-material composition requirement against shelved material-197db1, but do not wake it merely because a texture proof is proposed.
