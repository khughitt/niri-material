---
id: material-1135bc
title: "Optic trait, registry, shader assembly; migrate saturation and noise byte-identically"
status: dropped
priority: 2
size: l
created: 2026-09-10T09:33:23Z
updated: 2026-09-10T10:23:44Z
depends: []
parent: material-397fcb
tags: [material, api]
spec: docs/specs/2026-09-10-material-optics-design.md
---

Spec sections 1 to 5: the Optic trait and OpticFrame in niri, the ordered OPTICS table, prelude + optics + main shader assembly, optic values in the damage fingerprint and next_change in the tick deadline, niri-config/src/material/optics/ with saturation and noise moved in (inherit rule moves out of tile.rs). Done when just test passes and the noise/saturation smoke captures compare byte-identical before and after on the headless host, recorded in an evidence doc.

## Notes

- 2026-09-10T09:53:35Z (feat/material-397fcb): Review 2026-09-10: optic values are smithay Uniform (PartialEq at ff5fa7df), no OpticValue enum; per-optic neutral (saturation 1, emissive hooks additive returning vec3(0)); optic tick deadlines evaluated independently of signal_frame_cache, with a layout test for an unfocused signal-free aurora tile
- 2026-09-10T10:23:44Z (feat/material-397fcb): superseded by the plan steps under material-397fcb (docs/plans/2026-09-10-material-optics.md)
