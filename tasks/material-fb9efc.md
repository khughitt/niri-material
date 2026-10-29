---
id: material-fb9efc
title: "Rigid tilt swing: L2's far edge pulls in ~2 px more than the centre-pivot geometry allows"
status: idea
priority: 2
created: 2026-10-29T19:19:22Z
updated: 2026-10-29T19:31:05Z
depends: []
tags: [rendering, camera]
agent: claude-code/claude-opus-5-5
origin: agent
---

The 2026-10-26 amplitude ladder (material-c5f9d0; evidence docs/materials/2026-10-09-rigid-tilt-evidence.md, Clip) measured the far edge of a 604 px pane at view-tilt 20 / view-distance 6 pulled in 12.2 px in the frame 0.13 s after the gain. The design's centre pivot (spec §3.2, camera at D = κE at the peak) puts a W/2 edge at h cos θ / (1 − sin θ / 2κ), which is 9.9 px at the full 20°, an upper bound for a frame near the peak. So about 2 px (more if the frame sits below θp) is unexplained: the measurement (edge-finding on a scanline, the 6 px rest tolerance), the culling footprint, or the projection. The other rungs (10/6, 20/3) match the model within a pixel. It matters only if a setting near κ 6 at 20° is wanted; the shipped pick (20/3) matches the model.

## Notes

- 2026-10-29T19:19:22Z (add): created
  provenance: {"harness_session":"claude-code:2db0caf2-46d5-402c-b4aa-03b22c49e8bc","harness_session_source":"CLAUDE_CODE_SESSION_ID","origin":"agent"}
- 2026-10-29T19:31:04Z (material-abc08c): Rescoped 2026-10-29 after the scoped final review (material-c5f9d0 round 3): the filed 'not linear / not a hinge' reading compared against a hinge formula; the spec pivots at the centre, which explains the non-linearity and most of L2's far-edge motion. Only the residual stays.
