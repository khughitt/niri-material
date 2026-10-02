---
id: material-8867aa
title: Aperiodic backdrop for the parameter sweep
status: idea
priority: 2
created: 2026-09-07T00:03:24Z
updated: 2026-10-02T00:54:22Z
depends: []
parent: material-49871a
tags: [harness, rendering]
---

glass-parameter-sweep.sh paints a 20px grid backdrop. material-48dc76's thickness run showed that cumulative RMSE against the first capture aliases with that period for displacing parameters, while neighbouring deltas are only partly exposed. Replace or supplement the grid with an aperiodic field (blue noise, or a texture with no repeat within the maximum displacement) so cumulative is valid for thickness and any future displacing key. Keep the current backdrop reproducible: generate it deterministically. Depends on whether material-343f27 builds its own instrument first; if it does, this may fold into it.

## Notes

- 2026-09-29T22:38:51Z (materials-26.04): scope: briefed; 20 px backdrop still aliases displacement; share backdrop calibration with the ray-bending study before selecting a sweep change; brief: docs/notes/2026-09-29-glass-measurement-brief.md
- 2026-10-02T00:54:22Z (materials-26.04): Consumer: src/tests/ring_look.rs (accepted_ring_look) renders over flat gray, so refraction of the backdrop never shows in the accepted-look reference. An aperiodic backdrop there would let it cover edge refraction (material-be611b) too.
