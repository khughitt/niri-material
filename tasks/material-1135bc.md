---
id: material-1135bc
title: "Optic trait, registry, shader assembly; migrate saturation and noise byte-identically"
status: todo
priority: 2
size: l
created: 2026-09-10T09:33:23Z
updated: 2026-09-10T09:33:23Z
depends: []
parent: material-397fcb
tags: [material, api]
spec: docs/specs/2026-09-10-material-optics-design.md
---

Spec sections 1 to 5: the Optic trait and OpticFrame in niri, the ordered OPTICS table, prelude + optics + main shader assembly, optic values in the damage fingerprint and next_change in the tick deadline, niri-config/src/material/optics/ with saturation and noise moved in (inherit rule moves out of tile.rs). Done when just test passes and the noise/saturation smoke captures compare byte-identical before and after on the headless host, recorded in an evidence doc.
