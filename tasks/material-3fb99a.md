---
id: material-3fb99a
title: Harden the rigid tilt program-variant split (review minors of the contingency)
status: todo
priority: 3
size: s
complexity: low
process: direct
created: 2026-10-16T12:33:51Z
updated: 2026-10-16T12:33:51Z
depends: []
parent: material-6901e0
tags: [rendering, camera]
agent: claude-code/claude-opus-5-5
---

Minors from the review of f548b835 (perf(material): rigid tilt view path as a second program variant), all accepted as non-blocking: (1) put the rest-is-base check in the suite: variant() in src/render_helpers/shaders/mod.rs asserts no preprocessor line other than the exact #ifdef MATERIAL_VIEW / #else / #endif forms, and the_view_uniforms_are_declared asserts variant(material_source(), false) has no mat_view, g_view, g_seed_glide or slabFrame(; (2) the identity test in src/tests/view_tilt.rs cannot observe which program bound: add a cfg(test) accessor for the chosen ProgramType and assert MaterialView for the forced render, Material for rest; (3) the_view_uniforms_at_the_identity_pose_correct_nothing skips mat_view_dir (assert [0.6, 0.8]); (4) reword 'at rest' comments inside view-only blocks in prelude.frag (initial values, Surface.view +z) to 'at the identity pose'; (5) a comment at slabSurface's #else naming slabFrame as its twin; (6) note: Shaders::init now compiles the material program four times (normal/debug x rest/view) — time startup once on the NVIDIA host.
