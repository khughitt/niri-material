---
id: material-c854bd
title: Add mipmapped backdrop storage and glass roughness
status: todo
priority: 2
size: l
created: 2026-09-01T14:36:13Z
updated: 2026-09-01T14:36:13Z
depends: []
tags: [migration, feature, rendering]
---

Outcome: glass exposes a meaningful roughness control backed by damage-aware, buffer-owned mipmapped or prefiltered background and backdrop storage rather than an inert shader knob. Acceptance evidence: define and review the config and renderer contract; verify damage, cache invalidation, per-target isolation, default appearance, and bounded allocation/performance behavior with runnable tests and focused visual evidence; update the material reference. Sources: docs/materials/2026-08-22-v1-design.md, docs/materials/2026-08-24-v1-parity-design.md, and docs/materials/2026-08-27-v1-drm-acceptance-design.md. Uncertainty: the accepted designs establish this as the first post-v1 follow-up and roughness as the highest-value missing parameter, but do not choose mip generation, filtering, or cache policy.
