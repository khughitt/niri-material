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
updated: 2026-10-01T00:48:38Z
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
- 2026-10-01T00:14:15Z (glass-edges): resumed
  provenance: {"harness_session":"claude-code:52bcce0a-1e96-461e-8253-b540ad01dd61","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T00:14:15Z (glass-edges): spec revised for round 1: all four findings verified numerically and accepted; ray-to-base-plane path L=h/max(-t.z,.25) for displacement and attenuation (structural normal), two-boundary u=innerDist/(innerDist-outerDist) with blended gradient, highlight tilt weight (continuous for k>1, planar facets flash by design), interior light keeps pow(att,.2) on the new path (face exact, bevel brightens; old-path-for-interior rejected); Rust mirror bevel.rs, ring/aurora-on and jelly motion evidence added
- 2026-10-01T00:14:15Z (glass-edges): parked (waiting on user, review): User (or a reviewer) runs spec review round 2 on docs/specs/2026-09-30-glass-edge-optics-design.md in .worktrees/glass-edges; on approval the agent runs writing-plans and files the Prism piece
  provenance: {"harness_session":"claude-code:52bcce0a-1e96-461e-8253-b540ad01dd61","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T00:48:32Z (glass-edges): resumed
  provenance: {"harness_session":"codex:01a0f4b2-bd63-71f2-9779-d6e88ec61afe","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-01T00:48:33Z (glass-edges): review: spec round 2 — verdict: revise; findings: P2 3; reviewer: codex
- 2026-10-01T00:48:33Z (glass-edges): Round 1 disposition: all four findings addressed by fc5b6f6a. Recomputed all eight section 3.1 table rows within 0.01, highlight facet ratios (.000117 and .482696), interior factor .754321, and the two-boundary derivative. No implementation or live capture was run in this review.
- 2026-10-01T00:48:33Z (glass-edges): P2 — Spec lines 89–96 and 294–295: the two-boundary coordinate fixes the endpoints, but its exact gradient is not continuous everywhere under deformation. The outer rounded-box SDF has nearest-side switches inside its corner; a small-radius displaced inner face can expose those switches inside the bevel. Reproducer: outer half-size (56,56), radius 12, inner half-size (44,44), radius 0, inner offset (-3,0), R=12,k=2; approach p=(43,43) along p=(43+epsilon,43-epsilon) from both signs. At epsilon->0 the normals tend to (.107010,0,.994258) versus (.092865,.014287,.995576), a nonzero .020148 jump; radius 1 also reproduces. Reflection amplifies the direction change into a sample jump. Specify smoothing or explicitly accepted corner creases, and test across this nearest-side switch rather than only the visible arc.
- 2026-10-01T00:48:33Z (glass-edges): P2 — Spec lines 136–138: universal monotonicity is false for the chosen ray model. At bevel 12,k=2,ior=1.28,thickness=75.3, L is 75.30 on the face, 72.69 at u=.9, 74.09 at .97 and 78.10 at the capped rim. The configured thickness 200 grows from 200 to 231.95 at the rim. Conversely, thin/equal glass has h=0 at u=1 and therefore displacement 0, after a nonzero interior maximum; rounded-edge displacement cannot grow all the way to the rim in those cases. Keep the ray-consistent formula, narrow the prose and acceptance guarantees, and add these regimes to the reference cases instead of enforcing monotonicity.
- 2026-10-01T00:48:34Z (glass-edges): P2 — Spec lines 115–120 and 261–262: the critical-angle cosine is relative to the surface normal, not global z. For the downward incident ray and upward-facing normals, -t.z >= 1/ior; at ior=3 and slope cap 20, -t.z=.379591, contradicting the claimed >=sqrt(1-1/9)=.942809. The .25 floor never binds for structural ior in [1,3]; larger effective chromatic-aberration indices and perturbed normals need their own domain statement. Also, the face-displacement increase at 20 degrees is 2.82% at ior=3, so the under-1% statement needs an ior qualification. Correct the bounds and include base/effective-index endpoints in the numeric checks. Reference: PBRT 4e Specular Reflection and Transmission, Snell law (angles about the surface normal).
- 2026-10-01T00:48:34Z (glass-edges): parked (waiting on user, review): User reviews spec round 2 findings; design author settles the small-radius deformed-corner contract and corrects the ray-envelope claims and reference cases, then resubmits the spec before implementation planning.
  provenance: {"harness_session":"codex:01a0f4b2-bd63-71f2-9779-d6e88ec61afe","harness_session_source":"CODEX_SESSION_ID"}
