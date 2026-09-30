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
updated: 2026-09-30T23:48:19Z
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
- 2026-09-30T23:32:59Z (glass-edges): parked (waiting on user, review): User reviews docs/specs/2026-09-30-glass-edge-optics-design.md in .worktrees/glass-edges; on approval the agent runs writing-plans and files the Prism piece
  provenance: {"harness_session":"claude-code:52bcce0a-1e96-461e-8253-b540ad01dd61","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-30T23:47:37Z (glass-edges): resumed
  provenance: {"harness_session":"codex:01a0f4b2-bd63-71f2-9779-d6e88ec61afe","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-30T23:48:14Z (glass-edges): review: spec round 1 — verdict: revise; findings: P1 2, P2 2; reviewer: codex
- 2026-09-30T23:48:14Z (glass-edges): P1 — Spec lines 74–80: height is vertical, but tap multiplies a unit refracted ray by h without dividing by its downward z, while attenuation divides h by the surface-normal z instead. For profile 2, thickness 31.2, chamfer 12, u=.9, h=24.431 but the proposed optical path is 56.048 (face 31.2); an ior=1.28 ray to the base has length 25.914. Define displacement and attenuation from a consistent refracted-ray/base-plane intersection, with an explicit stable-normal policy; add thin/equal/thick cases.
- 2026-09-30T23:48:14Z (glass-edges): P1 — Spec lines 55–69: innerDist/chamfer is not 1 at the outer silhouette when jelly moves or resizes the inner face. With chamfer 12 and a 3 px face shift, opposite bevel widths are 9 and 15: one rim ends at u=.75; the other saturates into a 3 px plateau before the rim. Derive the coordinate and height gradient from the actual two boundaries, including deformed corners; verify movement and resize as well as settled captures.
- 2026-09-30T23:48:15Z (glass-edges): P2 — Spec lines 134–147: peak-normalized GGX is nonzero at the face join, on planar chamfers and on far sides. At roughness 1, alpha=.5 and edge-highlight=.5, the limit at a rounded face join adds .241348 linear brightness, versus zero on the gated face. A planar light-aligned normal 22.5 degrees from H also receives 48.27% of peak; thickness/chamfer=.414214 can align a planar corner normal exactly with H. Specify a continuous face transition and explicit profile/facing behavior; test roughness 0 and 1 and thin chamfers. Formula checked against PBRT Trowbridge-Reitz D.
- 2026-09-30T23:48:15Z (glass-edges): P2 — Spec lines 82–85 and 97–98: preserving within from the (1-F) multiply does not preserve its attenuation. Both ring terms and aurora currently consume pow(att,.2); replacing att with local-height attenuation changes their brightness even at profile 1. For thickness=chamfer=12, u=.5, attenuation=.1 and distance=10, the shared factor rises from .457708 to .676541. Specify whether interior attenuation remains at its existing depth or intentionally changes, and include ring/aurora-enabled evidence; current default-change captures disable both.
- 2026-09-30T23:48:15Z (glass-edges): parked (waiting on user, review): User reviews the draft spec together with spec review round 1; the design author revises sections 3.1 and 3.3, settles the interior-light attenuation contract, and resubmits the spec before an implementation plan.
  provenance: {"harness_session":"codex:01a0f4b2-bd63-71f2-9779-d6e88ec61afe","harness_session_source":"CODEX_SESSION_ID"}
