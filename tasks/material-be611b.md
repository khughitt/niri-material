---
id: material-be611b
title: "Realistic glass edges: height-field bevel profile and Fresnel reflection"
status: doing
priority: 2
size: m
complexity: high
process: planned
owner: glass-edges
created: 2026-09-30T23:27:38Z
updated: 2026-09-30T23:31:51Z
started: 2026-09-30T23:28:03Z
depends: []
tags: [rendering, material]
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-09-30-glass-edge-optics-design.md
---

Outcome: glass edges read as clear, material-connected edges across tint depth and thickness, instead of a uniform shifted, tinted stripe.

Diagnosis (2026-09-30): the chamfer is a planar ramp with one normal across its width (prelude.frag slabSurface); refraction offset uses global thickness (tap), Beer-Lambert uses thickness/cos (main.frag), and Schlick at 45 degrees is ~f0. Every chamfer term is constant across the band, and the model treats the bevel as full thickness. At the live Prism settings transmission is ~0 on face and chamfer (attenuation-color near black, exponent 2.8 focused), so the only varying term is a flat 4-15% white glint.

Candidate fixes, ranked: (1) height-field bevel profile (linear chamfer to rounded bullnose) with local height driving refraction offset and attenuation path; (2) energy-conserving Fresnel with a reflected backdrop sample, untinted; (3) light-facing specular lobe along the edge; (4, later) TIR inner-wall lines. Ring spill and aurora read innerDist/chamfer and must follow the profile.

## Notes

- 2026-09-30T23:28:03Z (glass-edges): started
  provenance: {"harness_session":"claude-code:52bcce0a-1e96-461e-8253-b540ad01dd61","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-30T23:31:50Z (glass-edges): spec drafted: docs/specs/2026-09-30-glass-edge-optics-design.md (height-field bevel + bevel-profile, (1-F) composition, reflection and edge-highlight optics, Surface specular hook)
