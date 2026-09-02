---
id: material-c854bd
title: Add mipmapped backdrop storage and glass roughness
status: done
priority: 2
size: l
owner: feat/material-c854bd
created: 2026-09-01T14:36:13Z
updated: 2026-09-02T10:08:21Z
depends: [material-ef9eec]
tags: [migration, feature, rendering]
---

Outcome: glass exposes a meaningful roughness control backed by damage-aware, buffer-owned mipmapped or prefiltered background and backdrop storage rather than an inert shader knob. Acceptance evidence: define and review the config and renderer contract; verify damage, cache invalidation, per-target isolation, default appearance, and bounded allocation/performance behavior with runnable tests and focused visual evidence; update the material reference. Sources: docs/materials/2026-08-22-v1-design.md, docs/materials/2026-08-24-v1-parity-design.md, and docs/materials/2026-08-27-v1-drm-acceptance-design.md. Uncertainty: the accepted designs establish this as the first post-v1 follow-up and roughness as the highest-value missing parameter, but do not choose mip generation, filtering, or cache policy.

## Notes

- 2026-09-02T00:13:41Z (feat/material-c854bd): Scope includes Prism definition/manifest/KDL wiring; native roughness defaults to 0 for v1 appearance, Prism restores historical default 0.08; roughness composes after backdrop-blur selects its input.
- 2026-09-02T00:23:42Z (feat/material-c854bd): Design direction: EffectBuffer-owned lazy prefilter pyramids using the existing blur downsample shader; reject GL mipmaps for GLES2/NPOT and transform-dependent LOD concerns, and reject per-roughness full-size caches as unbounded duplication.
- 2026-09-02T02:54:05Z (feat/material-c854bd): Design review adds a GL-free PrefilterPolicy, once-per-dirty-period failure suppression, realistic and degenerate allocation bounds, worst-case 96-fetch profiling, requested-level optimization path, and a measured overview softness criterion.
- 2026-09-02T03:06:24Z (feat/material-c854bd): Final design review renames the GL-free policy holder to PrefilterState, makes EffectBuffer ownership explicit, and scales the overview tolerance to one captured screen pixel at roughness 0.5.
- 2026-09-02T03:15:15Z (feat/material-c854bd): Implementation plan split into material-26a496, material-2793ca, material-768b40, material-46f6b7, and material-ef9eec; the final acceptance step remains open until the dependent Prism commit lands.
- 2026-09-02T03:38:53Z (feat/material-c854bd): Native implementation through 26c14954 passes GLSL validation and the full workspace suite; Prism handoff commits 2540fad and f1a65dd pass 141 Node plus Lua tests in a writable clone. Final closure awaits applying those commits to ~/d/prism and focused GPU evidence.
- 2026-09-02T10:08:21Z (feat/material-c854bd): Cross-repository delivery is complete: native implementation and acceptance evidence are on feat/material-c854bd; Prism roughness landed at c3c459d; full automated, nested GLES, Tracy reuse/performance, and calibrated overview gates pass.
