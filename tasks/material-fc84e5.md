---
id: material-fc84e5
title: Triage the rigid tilt plan's deferred review minors
status: todo
priority: 3
size: m
complexity: mid
process: direct
created: 2026-11-08T20:03:45Z
updated: 2026-11-08T20:03:45Z
depends: []
related: [material-3fb99a, material-c73cf0, material-2283b8, material-c5f9d0, material-fb9efc]
tags: [rendering, camera]
source: material-6901e0
agent: claude-code/claude-opus-5-5
origin: agent
---

Triage the minor findings the rigid tilt plan's task and final reviews deferred (plan docs/plans/2026-10-09-transient-rigid-tilt.md, goal material-6901e0). For each: fix it, or drop it with a reason in a note. Some may already be resolved by later tasks (material-3fb99a hardened the program-variant split); check the code at materials-26.04 before acting. Each line is the ledger's own text; the ledger is attached to material-6901e0 as rigid-tilt-ledger.md.

- Task 1: minor (deferred): material-config.md "Focus tilt" describes behaviour no runtime code reads until Tasks 6–9 land (plan-mandated, fine within the unreleased series)
- Task 1: minor (deferred): upstream-divergence.md lib.rs line-count churn from the hook
- Task 2: minor (deferred): ViewPose::at weight = shape.abs() can exceed 1 by an ulp at the peak; consider .min(1.)
- Task 2: minor (deferred): swing_shape clamps negative/NaN u silently (unreachable: settle ≥ 100 ms); debug_assert would document it
- Task 3: minor (deferred): no test pins the projection's sign/formula (to_screen(o + t·r) ≈ s round trip; lobe test passes with λ=0) — carry to Task 8 (shader mirror) dispatch as a pointer; final review to triage
- Task 3: minor (deferred): Camera::new accepts extent 0 (λ inf/NaN) — carry to Task 7 dispatch (the caller wiring Camera must refuse a degenerate slab, fail early)
- Task 3: minor (deferred): cos_m1 accuracy near θ=0 untested
- Task 3: minor (deferred): footprint allocates a Vec per swing frame
- Task 3: minor (deferred): pre-existing unused MergeWith import warning in src/layout/tests.rs (present at base 01dff3b7)
- Task 4: minor (deferred): bracketing test lacks a minimum hit count; asymmetric test lacks a skip bound; analytic misses accept Unresolved (assert Certified::Miss); reference shares pieces/psi with trace (no independent seam check)
- Task 5: minor (deferred): coarse corner test does not count its matched cases (assert_eq!(n, 3))
- Task 6: minor (deferred): background-effect eligibility gate untested (TestWindow always false)
- Task 6: minor (deferred): settle edge pinned loosely (no 0.899 s ongoing / 0.9 s pre-drop assertion)
- Task 6: minor (deferred): no resize-mid-swing layout test
- Task 6: minor (deferred): surface_effect_visible doc says "drew at its last render", it reads options resolved at the last render_for_tile
- Task 7: minor (deferred): fallback geo_rect remap at material/mod.rs:1025 can be uv_rect(self.frame.tex, self.drawn); divides by zero if smithay ever draws a zero-size element
- Task 7: minor (deferred): tile.rs:1320 comment overstates first-frame coverage: a swing starting on a pane whose resting body is just off-view schedules no frames (§5.1 accepts)
- Task 7: minor (deferred): pre-existing warnings in untouched code (MergeWith, too_many_arguments on render_inner, clone_on_copy, field_reassign_with_default in tests)
- Task 8: minor (deferred): doubled blank line common.frag:66–67
- Task 8: minor (deferred): projection test checks the sign only, not magnitude ζ/(1−ζ)
- Task 8: minor (deferred): tap rebuilds the point as v * mat_area_size inside the view branch (prelude.frag:522; zero-at-identity, brief-prescribed)
- Task 9: minor (deferred): a_swing_with_zero_rise_draws_no_nan never shown to fail on a shading NaN (no mutation demonstrated)
- Task 9: minor (deferred): pins miss one-character comparisons (at > 0.0 && at < 1.0; c2 < c1; b <= a; !g_trace_hit && psi <= 0.0; golden fc < fd); pins are one-sided (prelude text only)
- Task 9: minor (deferred): solid.rs:728 "coverage in [0,1]" assertion is effectively !m.is_nan(); comment overstates
- Task 9: minor (deferred): identity exactness assumes GPU pow never returns X > 1 for y < 1; clamp min(…, 1.0) in both mirrors (tested on NVIDIA RTX 3070 only)
- Task 10: minor (deferred): near-clip geometry bound holds by construction (assert right edge at the clip; assert drawn-up-to-clip range non-empty)
- Task 10: minor (deferred): geometry test lacks a precondition that some mid-swing want differs from frame.area
- Task 10: minor (deferred): close test tolerance max <= 2 unexplained (measured 0)
- Task 10: minor (deferred): both attach_new_shm_pixels are dead-code warnings until Task 13 calls them
- Task 11: minor (deferred): focus-loss cut not asserted (pane_glass thickness after the loss); COVERAGE_TOL basis restated (f32 vs f64 of one formula + 8-bit, not §4.5's bound) incl. chamfer-12 corners; diagonal 10% count tolerance unexplained; reveal-threshold test passes with no probe; diff(&peak,&cut) assert lacks message; main.frag:73-75 comment overstates; WALL_SPILL doc omits negative probe values
- Task 11: minor (deferred): Task 10's close-snapshot test roundtrips mid-swing on the unfrozen clock; passes only because the snapshot lands on the first dispatch (move double_roundtrip_at to the fixture)
- Task 12: minor (deferred): tasks/material-c73cf0.md done note carries a literal /mnt/ssd3 path (task notes are append-only; cannot be rewritten)
- Task 12: minor (deferred): §1 step 3 "geometry is its footprint" should be "derived from its footprint" (merged with resting area, cropped, rounded); line not wrapped
- Task 12: minor (deferred): evidence renderer claim: 16 of 34 dump JSONs record no renderer (cite the Task 8 note)
- Task 13: minor (deferred): rest-light, spill-off-light, control-light dumps unused by the script
- Task 13: minor (deferred): task note records a literal /mnt/ssd3 stills path (repo convention for $EV notes)
- Task 13: minor (deferred): S1/S2 "at 30 ms" not explained on the sheet; spill row does not say I/J are forced; I/J difference (2 rows, ≤ 11 levels) barely visible at 4× — consider an amplified difference tile
- Final review: minor (deferred): M1 fullscreen/maximize toggled mid-swing keeps swinging (tile.rs:720, :982) — cut or document
- Final review: minor (deferred): M2 render-pipeline.md:162 / evidence :49 imply slowdown stretches the beam (it does not)
- Final review: minor (deferred): M3 render-pipeline.md:156 omits top hits on the bevel from outline coverage
- Final review: minor (deferred): M4 unpinned GLSL lines (X inverse profile prelude:539, jy :701, g_decal_ok :489, centre-line at, bisection condition, viewHit face test, g_r)
- Final review: minor (deferred): M5 no end-to-end check that grain/jitter stay on the glass under a swing (Normal and 90° outputs)
- Final review: minor (deferred): M6 FocusOrigin holds a strong Output (focus_origin.rs:17); use WeakOutput or clear on removal
- Final review: minor (deferred): M7 per-pass focus_with_output/tile_centre_on_output scan and 5 uniforms per draw at rest — for Task 14 to account
- Final review: minor (deferred): M8 culling footprint uses buf_bbox vs physical-rounded offscreen geometry; few px near the clip
- Task 14: minor (deferred): final PASS line prints regardless of verdicts; zones helper drops the sibling's column/finite checks and the sharp_damage assert; probe-first-focus comment wrong (no swing during open animation); TERM to the script alone deferred until the foreground child ends
- Scoped final review: minor (deferred): m1 render-pipeline.md:169 and evidence:20 say the rest text equals the base; at HEAD it differs by the prelude.frag:403 comment
- Scoped final review: minor (deferred): m2 performance.md:209 "at rest: nothing" omits the start-up cost (~4.5 s cold, ~23 ms warm) every config pays
- Scoped final review: minor (deferred): m3 a failed view-variant compile disables all glass even at view-tilt 0 (material/mod.rs:844-846, warn only)
- Scoped final review: minor (deferred): m4 evidence:23/:118/:154 cite pre-rewrite 2f86b3ac/f548b835; name f3167120/bbc5ca3c beside them
- Scoped final review: minor (deferred): m5 swing-measured-sheet.py:180 max(moving) raises on a run with no motion; hard-coded Arch font path :29; ring-motion-clips.sh requirements omit numpy
- Scoped final review: minor (deferred): m6 evidence:168 cites swing-review-sheet.png, which no committed script produces (the 0.43 s and 8 px misstatements were fixed in the I1 rewrite)
- Scoped final review: minor (deferred): m7 ring-motion-clips.sh:3 "Twelve sequences" (default runs eight); :61 calls 10/6 the owner's pick
- Scoped final review: minor (deferred): m8 BOUND_PROGRAMS records the selected program before draw, not the bound one (material/mod.rs:1149); cfg(test) vector never drained outside the identity test
- Scoped final review: minor (deferred): m9 material-2283b8's 2026-10-16 review note says round 1 (should be 3); material-c5f9d0's round 2 note says reviewer human, corrected in prose only
- Scoped final review: minor (deferred): evidence Clip: lines 174/192 wrap unevenly (192 is 86 columns)
- Scoped final review: minor (deferred): evidence Clip :186-187 gives the model only at the peak (D = κE); write D = κE/w and h cos θ / (1 ± w sin θ / 2κ), w = 1 at the peak, to match the frame-angle column
- Scoped final review: minor (deferred): material-fb9efc lists frame timing as a cause; timing can narrow the residual from ~4 px to ~2 px but not below the θp bound's 2 px

## Notes

- 2026-11-08T20:03:45Z (add): created
  provenance: {"harness_session":"claude-code:f2252ded-1918-4960-a1f1-d29c8039757f","harness_session_source":"CLAUDE_CODE_SESSION_ID","origin":"agent"}
