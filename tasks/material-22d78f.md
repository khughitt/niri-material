---
id: material-22d78f
title: Deterministic mid-flex probe for filament face confinement
status: idea
priority: 2
created: 2026-09-05T16:12:46Z
updated: 2026-10-01T09:55:04Z
depends: []
parent: material-49871a
tags: []
---

resize-flex in focus-ring-light.sh compares two nested hosts mid-resize; independent clocks skew 2-5 px so the 2-level face bound cannot be measured. Needs a way to capture both filament-on and filament-off at identical animation progress (clock stepping or a single-host toggle).

## Notes

- 2026-09-29T22:38:51Z (materials-26.04): scope: briefed; independent-host resize skew remains; old zero-face-light requirement was superseded by the within ring; investigate matched-state capture using existing frozen-clock fixtures; brief: docs/notes/2026-09-29-glass-measurement-brief.md
- 2026-10-01T09:55:04Z (material-0e80c1): material-0e80c1 result: a matched pair needs no production change. src/tests/ring_pair.rs renders ring on/off at a frozen 500 ms instant of a linear resize, byte-reproducibly (repeat 0 px; a 10 ms shifted control changes 22-24k px). Under flex the band centroid moves inward on the resizing axis only, +0.44 px at jelly-flex 0.0066 and +0.96 px at 0.02. Opaque client: 0 px lit inside the window. The face-bound premise is obsolete (light over a translucent face scales with 1 - client alpha). Remaining: an owner-set tolerance for the proposed centroid motion check; case_resize_flex's two-host comparison can give way to this fixture. Brief: docs/notes/2026-09-29-glass-measurement-brief.md#matched-state-ring-findings
