---
id: material-ce3315
title: Verify backdrop blur preserves v1 defaults on physical DRM
status: done
priority: 2
size: m
created: 2026-09-01T14:36:13Z
updated: 2026-09-01T23:35:58Z
depends: []
tags: [migration, acceptance, hardware]
---

Outcome: a build carrying backdrop-blur proves that the default-off material path preserves the accepted v1 appearance on the real DRM/NVIDIA path. Acceptance evidence: run the existing physical DRM harness scenarios unchanged; require byte-identical paired settled frames and all applicable machine and operator gates; record the pinned source, binary, environment, capture hashes, observations, cleanup, and reconciled verdict. Sources: docs/materials/2026-08-29-material-backdrop-blur-design.md, docs/materials/2026-08-29-material-backdrop-blur-evidence.md, and docs/materials/2026-08-27-v1-drm-acceptance-design.md. Uncertainty: nested verification proves opt-out equivalence on Weston but cannot establish physical scanout or the existing paired-frame contract.

## Notes

- 2026-09-01T23:35:58Z (materials-26.04): All 19 machine gates, 44 captures and 9 operator observations passed against niri 26.04 (52f74f10). Every steady-state metric reproduced the accepted run exactly (diagnostic-tile 1057/45698/1124, reload-changed AE 58326.8, settled motion 423171 and 14652.1); only mid-animation frames differ, by scheduling. Recorded at niri-experiments 13d6dda; design and README status corrected in 8c6618ca.
