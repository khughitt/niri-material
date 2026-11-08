# SDD ledger — plan: docs/plans/2026-10-09-transient-rigid-tilt.md

Spec: docs/specs/2026-10-09-transient-rigid-tilt-design.md (accepted round 4). Plan accepted round 4 (owner). Branch base for final review: 01dff3b7.
Setup: just setup ok; just test-fast selected 0 tests (no package changed); baseline niri-config + niri material run separately.
Baseline: niri-config 126/126 passed; niri material filter 125 passed (422 skipped). Build warm.

## Pre-flight scan (preflight-scan.md in this workspace: 7 Critical, 4 Important, 20 Minor; Tasks 1–5 and 7–9 interfaces agree with each other and the source)

Each ruling below is carried into the named task's dispatch.
- Preflight C1 (Task 6) Ruling: also implement `background_effect_visible` on niri-visual-tests/src/test_window.rs:134 TestWindow, returning false (no trait default) — explicit over defensive; clippy --all-targets must compile — cost if wrong: a visual test ignores a client blur region, cosmetic.
- Preflight C2 (Task 8) Ruling: update the pin at src/render_helpers/shaders/mod.rs:583 to the new `noise_post(glassColor, seed)` line in Step 5 — the plan missed an existing pin — cost if wrong: none, test text only.
- Preflight C3 (Task 9) Ruling: declare `bool g_wall = false;` (constant initializer) and audit every new g_view*/g_face*/g_wall global read outside a `g_view_on` branch for a constant initializer (Task 8 too) — the spec's rest path must be defined behaviour; Task 12 byte identity depends on it — cost if wrong: none.
- Preflight C4 (Task 10) Ruling: the geometry test takes the swing direction from the tile's actual swing (Task 6's `view_swing_start()` accessor), not `DVec2::NEG_X` — the gain direction depends on tile centres, which differ in y — cost if wrong: test asserts against its own read-out; the footprint's direction dependence is still pinned by Task 3's unit tests.
- Preflight C5 (Task 10) + I3 (Task 13) Ruling: Task 10's pane helpers (`pane_geometry`, `pane_rect`, `slab_frame`, …) locate the pane by its window's id, never by `tiles_with_render_positions().next()` (active column first) — the test must measure the pane it names — cost if wrong: helper signature change for Tasks 11 and 13.
- Preflight C6 (Task 11) Ruling: in the focus-loss and refocus sequences the second window gets a material/rule with view-tilt 0, so focusing it starts no swing of its own — the sequences test the first pane's swing — cost if wrong: the "second pane swings too" interaction goes untested in these sequences (Task 6 layout tests cover per-gain starts).
- Preflight C7 (Task 11) Ruling: the ineligible blur case adds its `background-effect { blur true; }` as a second window-rule in `Scene.after`, not inside the rule that already has a background-effect node — knuffel rejects the duplicate child — cost if wrong: none.
- Preflight I1 (Task 10) Ruling: the containment client sets an explicit window geometry equal to the window's own size (add the test-client call if missing), and each containment/near-clip test asserts its own precondition (texture extends past the slab; the near clip engages for at least one corner) so it cannot pass vacuously — spec §8.2 asks for these cases — cost if wrong: extra test-client surface.
- Preflight I2 (Task 13) Ruling: the still-sheet fixture gives both panes equal height so the swing is exactly horizontal, and the generator asserts the direction's y component is 0 — the owner judges horizontal reveals as the plan describes — cost if wrong: the sheet shows a fixture slightly different from the plan's 400×300 second pane.
- Preflight I4 (Tasks 10, 11) Ruling: Task 10 imports `Camera` and `VIEW_NEAR_CLIP`; Task 11 adds no duplicate import — compile fix — cost: none.
- Preflight m2 (Task 4) Ruling: the entry-depth tolerance follows the spec, 1/64 physical px (divide by the case's scale), not 1/128 logical — spec is binding — cost if wrong: a tighter test that may need the bisection count the spec already fixes.
- Preflight m8 (Task 9) Ruling: `a_swing_with_zero_rise_draws_no_nan` also asserts the coverage probe stays in [0, 1] — spec §8.2 — cost: none.
- Preflight m9 (Task 9) Ruling: the wall override sets both normals after the perturbation, so the wall uses its normal as both normals with distortion or ripple on — spec §4.3 — cost if wrong: wall shading differs from the plan's code under distortion only.
- Preflight m6, m5 Ruling: keep the plan's rest-identical edits outside branches (`g_seed_glide` in noise.frag, `seed` for `gl_FragCoord.xy`, the gated `v` remap) — values are exactly today's at rest and Task 12's byte identity is the guard; Task 14's contingency already undoes them — cost if wrong: Task 12 fails and those lines move into branches.
- Preflight m7/m20 Ruling: drop `viewHit`'s unused `p` parameter; keep `ElementView.grow`, `ViewPose::identity` and `Segment.h0_shift` only if read (gate test-only items `#[cfg(test)]`, delete what nothing reads) — no dead surface — cost: none.
- Preflight m16, m17 Ruling: keep the negative-gain probes and the ignored still-sheet generator as the plan's deviations section specifies (owner accepted them) — cost if wrong: test hooks in the production shader, two uniform-gated branches.
- Preflight m18 (Task 6) Ruling: `focus_loss_lets_the_swing_finish` also asserts the swing ends at settle — the name claims it — cost: none.
- Preflight m19 Ruling: where an existing helper does the same job (ring_pair::render_at, ring_look's open/gain/config, Solid::sigma, drawn_area's bounds), implementers reuse it rather than copying, when that changes no behaviour — cost if wrong: small refactors of test helpers.
- Preflight m1, m3, m4, m10–m14 Ruling: carried as notes to their tasks (m1 Task 4's Step 2 test passes at RED, expected; m3 literal is in fingerprint2; m4 avoid needless_return; m10 Task 12 picks a render-pipeline stage label not already used; m11 Task 12 runs the host-pointer check before removing the base worktree; m12/m13/m14 shell vars re-derived per call, keep the hook, test-fast before Task 13's commit) — cost: none.
Task 1: BASE 01dff3b7
Task 1: implementer dispatched (sonnet), agent a33519bb6b24f9e67
Task 1: implementer DONE a7139f22; review dispatched (sonnet)
Task 1: minor (deferred): material-config.md "Focus tilt" describes behaviour no runtime code reads until Tasks 6–9 land (plan-mandated, fine within the unreleased series)
Task 1: minor (deferred): upstream-divergence.md lib.rs line-count churn from the hook
Task 1: complete (commits 01dff3b7..a7139f22, review clean)
Task 2: BASE a7139f22
Task 2: implementer dispatched (sonnet), agent a7992cb62282fd038
Task 2: implementer DONE_WITH_CONCERNS 8520cb82 (skipped RED stub step); review dispatched
Task 2: ⚠️ seeded_direction parity resolved by controller: identical to d51825f9:src/render_helpers/material/view.rs:67-69 (hash01(seed ^ 0x5EED_D1EC)·2π)
Task 2: Ruling: accept the skipped RED stub step — the tests assert the spec's curve numbers and re-derive G independently, so a pass is meaningful without the stub run — cost if wrong: none observable; later dispatches restate that RED is required.
Task 2: minor (deferred): ViewPose::at weight = shape.abs() can exceed 1 by an ulp at the peak; consider .min(1.)
Task 2: minor (deferred): swing_shape clamps negative/NaN u silently (unreachable: settle ≥ 100 ms); debug_assert would document it
Task 2: complete (commits a7139f22..8520cb82, review clean)
Task 3: BASE 8520cb82
Task 3: implementer dispatched (sonnet), agent aa8254a25d22bef18
Task 3: implementer DONE 8fdb0816; review dispatched (opus: math vs spec §4.1/§6)
Task 3: minor (deferred): no test pins the projection's sign/formula (to_screen(o + t·r) ≈ s round trip; lobe test passes with λ=0) — carry to Task 8 (shader mirror) dispatch as a pointer; final review to triage
Task 3: minor (deferred): Camera::new accepts extent 0 (λ inf/NaN) — carry to Task 7 dispatch (the caller wiring Camera must refuse a degenerate slab, fail early)
Task 3: minor (deferred): cos_m1 accuracy near θ=0 untested
Task 3: minor (deferred): footprint allocates a Vec per swing frame
Task 3: minor (deferred): pre-existing unused MergeWith import warning in src/layout/tests.rs (present at base 01dff3b7)
Task 3: ⚠️ items routed: Task 8 must mirror cos_m1 as -2 sin²(θ/2) and λ as a uniform; Task 7 must merge resting area and intersect with the view crop (§6)
Task 3: complete (commits 8520cb82..8fdb0816, review clean)
Task 4: BASE 8fdb0816
Task 4: Ruling (m2 applied): entry-depth tolerance 1/1024 logical px (spec 1/64 physical px at scale ≤ 16) replaces the brief's 1/128 logical — spec §4.1/§8.1 binding, 24 bisection steps leave ample margin — cost if wrong: a test that fails where the spec's own counts cannot reach it, reported rather than loosened.
Task 4: implementer dispatched (opus), agent ad194859e6891877b
Task 4: implementer DONE e4a3a2cf (worst entry depth 1.15e-5 px); review dispatched (opus)
Task 4: review: Needs fixes — Important 2 (both plan-mandated), Minor 5
Task 4: Ruling: the wall rule evaluates Ψ at the rim depth with the quadrant of the piece that contains the rim depth (earlier piece on a boundary), not the entry piece's — spec §4.1 "Kinds of hit" classifies an entry below the rim through a seam's step face as a wall hit; reviewer reproduced 52/200 misclassified rays on the asymmetric fixture; at the identity there is one piece, so the rest identity holds — cost if wrong: Task 9's GLSL line and pin change with it (carried to Task 9).
Task 4: Ruling: zero_rise_is_the_box_or_the_plane counts resolved/skipped rays and asserts skipped*100 < resolved plus both hits and misses — tests over random rays must not pass vacuously — cost: none.
Task 4: minor (deferred): bracketing test lacks a minimum hit count; asymmetric test lacks a skip bound; analytic misses accept Unresolved (assert Certified::Miss); reference shares pieces/psi with trace (no independent seam check)
Task 4: fix round 1 dispatched (resume implementer) at FIX_BASE e4a3a2cf
Task 4: ⚠️ routed to Task 9: (a) sure_miss must reuse sd_rounded_box with [r_min;4] (m19); (b) the pin line `return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - r;` already exists in prelude.frag:259's sdRoundedBox, so make viewSigmaR call sdRoundedBox(xy - g_center, g_half, vec4(r)) (mirrors the Rust reuse) and pin a viewSigmaR-specific line; (c) the GLSL wall line `viewPsi(viewPoint(rim), entrySel)` and its pin change to the rim piece's sel per the Task 4 ruling
Task 4 → Task 8: view block must sit after cornerRadius/sdRoundedBox/sdRoundedBoxGrad (prelude.frag:237–263)
Task 4: fix commit 7efbb358; re-review dispatched (sonnet)
Task 4: fix round 1/5 (2 addressed, 0 open; commits e4a3a2cf..7efbb358)
Task 4: complete (commits 8fdb0816..7efbb358, review clean after 1 fix round)
Task 5: BASE 7efbb358
Task 5: implementer dispatched (opus), agent afd8c362c2121679f
Task 5: implementer DONE_WITH_CONCERNS, uncommitted: brief's outline() verbatim exceeds §4.5's bound only outside the outline (m>0 and exact>0), at thickness 200, 20°, κ1.5 axis swings, lifting side (edge-on wall; face end takes the minimiser; side worst 0.925 px, corner grid worst 0.9247); where coverage depends on m (m≤0 or exact≤0) worst 0.315 px at corners, sides within 1/16
Task 5: Ruling: the §4.5 error bound is asserted wherever coverage depends on m, i.e. at every point with m ≤ 0 or exact ≤ 0 (coverage = 1 − smoothstep(m+1) is exactly 0 for m ≥ 0, so a point with both positive renders identically); points with both positive are counted and must keep m > 0 (no sign flip), and the test comment cites this ruling — the bound exists to make coverage accurate and the formula is the spec's own (Ψmin/|∇Ψmin|), so changing the formula (option b) would rework Task 9 for no pixel difference — cost if wrong: m is not a true distance outside the outline at thick steep swings, which matters only if a later consumer reads m > 0 for something other than coverage (none in the spec; final review to confirm).
Task 5: Ruling: the seam-step continuity test uses refinement — a flagged step is subdivided (e.g. 1000×) and must fall below the threshold, so a real jump persists and a steep ramp (12.9/px at y=−49, the seam's 8.6 px step per spec §4.5 line 485) does not — §4.5 asks continuity within each piece and coverage in [0,1], not a slope limit — cost if wrong: a very steep but continuous ramp passes; the frozen renders (Task 11) still compare pixels.
Task 5: implementer DONE 3d040817 (sides worst 0.0027, corners worst 0.3150 on coverage-relevant points; seam refinement abs 1e-3; dropped the m>0 assertion as unfalsifiable); review dispatched (opus)
Task 5: review: Needs fixes — Important 1 (NaN m outside the outline is exempt, never asserted), Minor 2; ⚠️ coverage pub(crate) confirmed as controller-requested
Task 5: minor (deferred): coarse corner test does not count its matched cases (assert_eq!(n, 3))
Task 5: fix round 1 dispatched (resume implementer) at FIX_BASE 3d040817
Task 5: fix round 1/5 (1 addressed, 0 open; commits 3d040817..2c1ca6b1)
Task 5: complete (commits 7efbb358..2c1ca6b1, review clean after 1 fix round)
Task 6: BASE 2c1ca6b1
Task 6: implementer dispatched (opus) with C1, m18, m19 rulings; agent ad3dd1f5a290c6b57
Task 6: implementer DONE_WITH_CONCERNS d23e336f (transient dead_code warning on MaterialDynamics.view until Task 7 reads it)
Task 6: Ruling: no temporary allow(dead_code) for MaterialDynamics.view — Task 7 reads it in the next commit — cost if wrong: one commit with a warning.
Task 6: review dispatched (opus)
Task 6: ⚠️ footprint visibility routed to Task 7 review (swing leaves the signal_render_visible group; clear_signal_render_visibility resets view_swing_visible)
Task 6: ⚠️ slowdown: ViewSwing on now_unadjusted() — Ruling: accepted, spec §5.1 "follows the real animation clock", as the beam does — cost if wrong: swings ignore animations.slowdown.
Task 6: minor (deferred): background-effect eligibility gate untested (TestWindow always false)
Task 6: minor (deferred): settle edge pinned loosely (no 0.899 s ongoing / 0.9 s pre-drop assertion)
Task 6: minor (deferred): no resize-mid-swing layout test
Task 6: minor (deferred): surface_effect_visible doc says "drew at its last render", it reads options resolved at the last render_for_tile
Task 6: complete (commits 2c1ca6b1..d23e336f, review clean)
Task 7: BASE d23e336f
Task 7: implementer dispatched (opus) with degenerate-slab, visibility, m3, m4, m19, m20, RF3/RF4 notes; agent aa647588a6fa90d42
Task 7: implementer DONE 6bc3bade (deviations: empty crop early return; out_of_view rest fallback for slabs that cannot swing; seed basis covers all 8 transforms); review dispatched (opus)
Task 7: deviations accepted by reviewer: (i) empty crop early return, (ii) out_of_view falls back to the resting test for slabs that cannot swing (fixes a brief culling bug), (iii) 8-transform seed basis test verified against smithay ff5fa7d
Task 7: minor (deferred): fallback geo_rect remap at material/mod.rs:1025 can be uv_rect(self.frame.tex, self.drawn); divides by zero if smithay ever draws a zero-size element
Task 7: minor (deferred): tile.rs:1320 comment overstates first-frame coverage: a swing starting on a pane whose resting body is just off-view schedules no frames (§5.1 accepts)
Task 7: minor (deferred): pre-existing warnings in untouched code (MergeWith, too_many_arguments on render_inner, clone_on_copy, field_reassign_with_default in tests)
Task 7: complete (commits d23e336f..6bc3bade, review clean)
Task 8: BASE 6bc3bade
Task 8: implementer dispatched (opus) with C2, C3, m7, m6-kept, placement, m19, C5-helper, projection-sign (optional), renderer notes; agent a00e7efd8a191e31f
Task 8: implementer DONE 4617d716 (identity 0 px in 16 cases; renderer NVIDIA RTX 3070; rest dumps byte-identical to 6bc3bade); review dispatched (opus)
Task 8: ⚠️ routed: Task 9 must land view::solid::tests::the_shader_names_the_same_constants_and_formulas (pins VIEW_NEAR_CLIP and mirrored lines); render checks at nonzero tilt and seed tracking are Tasks 10–11
Task 8: minor (deferred): doubled blank line common.frag:66–67
Task 8: minor (deferred): projection test checks the sign only, not magnitude ζ/(1−ζ)
Task 8: minor (deferred): tap rebuilds the point as v * mat_area_size inside the view branch (prelude.frag:522; zero-at-identity, brief-prescribed)
Task 8: complete (commits 6bc3bade..4617d716, review clean)
Task 9: BASE 4617d716
Task 9: implementer dispatched (opus) with C3 g_wall init, rim-piece wall rule, σ/sure_miss reuse + pin replacement, m8, m9, pin test; agent a3cba849c09ed18cc
Task 9: implementer DONE_WITH_CONCERNS 1860780b (identity 0 px; rest dumps byte-identical glass_edge 54/54, ring_look 14/14, view_tilt 84/84)
Task 9 → Task 11 (carry): the zero-rise render test cannot see NaN/out-of-range coverage (NaN fails coverage>0 and drops the glass; removing viewPsiDz's guard left 12 120 px NaN coverage undetected). Task 11's −2 coverage probe must flag !(c >= 0.0 && c <= 1.0) and re-run that zero-rise swing asserting no flagged pixel.
Task 9: review dispatched (opus)
Task 9 → Task 11 (carry, Ruling): the −2 coverage probe renders swing_frames() for bevel0(STOCK) and thickness0(STOCK) (thickness 0 with chamfer > 0) and compares with coverage(outline(..)) from the mirror wherever the mirror predicts coverage > 0 — §8.2 "zero rise in motion" must be checked in the shader, not only on the mirror — cost if wrong: one more probe test (seconds).
Task 9: minor (deferred): a_swing_with_zero_rise_draws_no_nan never shown to fail on a shading NaN (no mutation demonstrated)
Task 9: minor (deferred): pins miss one-character comparisons (at > 0.0 && at < 1.0; c2 < c1; b <= a; !g_trace_hit && psi <= 0.0; golden fc < fd); pins are one-sided (prelude text only)
Task 9: minor (deferred): solid.rs:728 "coverage in [0,1]" assertion is effectively !m.is_nan(); comment overstates
Task 9: minor (deferred): identity exactness assumes GPU pow never returns X > 1 for y < 1; clamp min(…, 1.0) in both mirrors (tested on NVIDIA RTX 3070 only)
Task 9: note: reviewer assumed llvmpipe; Task 8 recorded the test renderer as NVIDIA GeForce RTX 3070
Task 9: complete (commits 4617d716..1860780b, review clean)
Task 10: BASE 1860780b
Task 10: implementer dispatched (opus) with C4, C5, I1, I4, m19 rulings; agent a59ff2cac472b176d
Task 10: implementer DONE_WITH_CONCERNS 2cb538d4 (rest dumps byte-identical; mutations bite; near clip reached with the brief's pose)
Task 10: deviations: no Tile::material_id (tile.material().id() already public); crop test rebuilt on grain-free LIVE with a reach precondition (brief's was vacuous); close-snapshot test landed here (pane footprint only, 0 px)
Task 10: Ruling: the close-snapshot test stays in Task 10; Task 11 drops its whole-frame copy (brief lines ~346–360) — it would fail on grain and on the other window's focus change, and duplicates the pane-footprint check — cost if wrong: the whole-frame close comparison is not run.
Task 10: Ruling: remove the ported test-client helpers no plan task calls (create_popup, ack_and_attach_popup, LayerSurface::attach_new_shm_pixels if unused) — YAGNI, no dead-code warnings; the plan names them only as part of the port — cost if wrong: re-port from 69431712 when a popup test is wanted.
Task 10: review dispatched (opus)
Task 10: review: Needs fixes — Important 2 (subsurface case vacuous for §6, plan-mandated; popup helpers still present), Minor 4; deviations (i)–(iii) accepted
Task 10: Ruling: the subsurface containment case moves to the lifting side beyond the window (e.g. [420, 100, 120, 80] at dir +X) and asserts that the bounding box of the 4 projected subsurface corners is contained neither in frame.area nor in the slab box's projection alone — §8.2's first containment case must fail on a footprint that ignores the texture (§6) — cost: none.
Task 10 → Task 12 (carry, Ruling): record a spec deviation: §8.2's "with grain on" cannot hold for offscreen (alpha/close snapshot) comparisons, since grain seeds from gl_FragCoord and moves with the offscreen origin (203 989 px, up to 59 levels at rest, pre-existing); those cases run on grain0 — the plan's deviations list and the evidence doc say so — cost if wrong: grain under offscreen paths stays untested.
Task 10 → Task 11 (carry): fixture_pane returns (Fixture, Pane); pane_glass/slab_frame/pane_rect/pane_geometry take a MappedId; drop the whole-frame close-snapshot copy; any offscreen comparison uses grain0 and a footprint-only comparison.
Task 10: minor (deferred): near-clip geometry bound holds by construction (assert right edge at the clip; assert drawn-up-to-clip range non-empty)
Task 10: minor (deferred): geometry test lacks a precondition that some mid-swing want differs from frame.area
Task 10: minor (deferred): close test tolerance max <= 2 unexplained (measured 0)
Task 10: minor (deferred): both attach_new_shm_pixels are dead-code warnings until Task 13 calls them
Task 10: fix round 1 dispatched (resume implementer) at FIX_BASE 2cb538d4
Task 10: fix commit 4cbddddc; re-review dispatched (sonnet)
Task 10: fix round 1/5 (2 addressed, 0 open; commits 2cb538d4..4cbddddc)
Task 10: complete (commits 1860780b..4cbddddc, review clean after 1 fix round)
Task 11: BASE 4cbddddc
Task 11: implementer dispatched (opus) with close-drop, C6, C7, I4, m19, zero-rise probe, m16 rulings; agent a7bea30f60ab86a56
Task 11: implementer DONE_WITH_CONCERNS dfbfa344 (no shader-mirror mismatch; rest dumps byte-identical; zero-rise probe fails 1 279–2 248 px/frame under the viewPsiDz mutation)
Task 11: Ruling: accept grain0(LIVE) and pane-geometry-scoped black counts in focus-loss/resize — grain leaves 467 pure-black rest pixels on LIVE, so a whole-frame black check is invalid — cost if wrong: grain under a swing is not checked in the sequences (Tasks 8–10 cover grain at rest/identity).
Task 11: Ruling: accept one render before the gain in the ineligible loop, plus an eligible control row — background-effect visibility is resolved at the last render, and a real pane is always drawn before it gains focus — cost if wrong: none observed.
Task 11: Ruling: accept the resize sequence's final-size assertion (560×420).
Task 11: Ruling: the −2 coverage probe also flags non-finite or out-of-range coverage (!(c >= 0.0 && c <= 1.0) → a sentinel colour, e.g. magenta), and the coverage probe tests assert no flagged pixel — on NVIDIA a NaN coverage reads as 255, so where the mirror predicts exactly 1 a NaN would pass; the probe is test-only and gated behind a negative gain — cost: none at rest.
Task 11: review dispatched (opus)
Task 11: review: Needs fixes — Important 3 (NaN sentinel ruling unmet; wall/face/grazing comparisons can pass vacuously, partly plan-mandated; resize "runs through" unasserted and finish-at-rest shown only as constancy, plan-mandated), Minor 8
Task 11: ⚠️ "opening" ineligible case — Ruling: covered by src/layout/tests/view_swing.rs an_opening_pane_takes_no_swing_for_that_gain (Task 6) — cost if wrong: no frozen render of an opening pane.
Task 11: Ruling: fix all three Importants as the reviewer specifies (per-pixel wall comparison against the mirror at pixel centres ≤ 1 px per edge plus mirror-predicts-a-wall preconditions; face-coverage counts pixels with mirror coverage in (0,1) and fails rather than skips on Segment::new None; grazing asserts band pixels and some coverage change, and uses expect instead of unwrap_or(INFINITY); resize asserts the same swing survives the commit and the 171 ms shot differs from REST; REST frames assert view_pose() is None) — spec §8.2 "exactly where the mirror predicts" and §5.1/§8.2 "a resize runs through" — cost: test time only.
Task 11: minor (deferred): focus-loss cut not asserted (pane_glass thickness after the loss); COVERAGE_TOL basis restated (f32 vs f64 of one formula + 8-bit, not §4.5's bound) incl. chamfer-12 corners; diagonal 10% count tolerance unexplained; reveal-threshold test passes with no probe; diff(&peak,&cut) assert lacks message; main.frag:73-75 comment overstates; WALL_SPILL doc omits negative probe values
Task 11: fix round 1 dispatched (resume implementer) at FIX_BASE dfbfa344
Task 11: fix commit c0e7589b (sentinel fires on real NaN/out-of-range coverage; viewPsiDz guard removal yields non-finite m that smoothstep clamps to 1.0, caught by the mirror comparison; resize sequence now holds the frozen clock via double_roundtrip_at — the brief's version never ran through a swing)
Task 11: Ruling: no view-path flag for a non-finite m before smoothstep — where the mirror predicts coverage 1 a clamped non-finite m renders the right pixel, and everywhere else the mirror comparison catches it; adding a flag means production shader code for a test — cost if wrong: a NaN m inside full coverage goes unnoticed (renders correctly).
Task 11: re-review dispatched (sonnet)
Task 11: fix round 1/5 (3 addressed, 0 open; commits dfbfa344..c0e7589b)
Task 11: minor (deferred): Task 10's close-snapshot test roundtrips mid-swing on the unfrozen clock; passes only because the snapshot lands on the first dispatch (move double_roundtrip_at to the fixture)
Task 11: complete (commits 4cbddddc..c0e7589b, review clean after 1 fix round)
Task 12: BASE c0e7589b
Task 12: implementer dispatched (sonnet) with m10, m11, m12, base check, evidence 'Execution rulings' section (spec-facing-rulings.md); agent a2b73dd0a7929666b
Task 12: implementer DONE 71e03795 (137/137 byte-identical base 23039f67 vs head c0e7589b; EV /mnt/ssd3/niri-material/material-6901e0/baseline-20261009T2350; tracy binary niri-tracy-23039f67; base worktree removed without --force, no host pointers); review dispatched (sonnet)
Task 12: review: Needs fixes — Important 4 (docs misstate: coverage applies to every face hit; grain seeding fully whole-pixel; visibility "slab band" vs footprint; unsupported "a few percent larger"), all in brief-provided doc text; Minor 4
Task 12: Ruling: the docs follow the code (face hits shade at h₀ and trace for coverage only within 4 physical px of the outline; a sure miss has coverage 0; tap jitter and grain at scale ≤ 1 on whole pixels, grain above scale 1 glides by the fractional remainder; the swing animates while its footprint is in view; footprint size depends on θ, κ and thickness, no fixed percentage) — the brief's prose predates the reviewed code — cost: none.
Task 12: minor (deferred): tasks/material-c73cf0.md done note carries a literal /mnt/ssd3 path (task notes are append-only; cannot be rewritten)
Task 12: minor (deferred): §1 step 3 "geometry is its footprint" should be "derived from its footprint" (merged with resting area, cropped, rounded); line not wrapped
Task 12: minor (deferred): evidence renderer claim: 16 of 34 dump JSONs record no renderer (cite the Task 8 note)
Task 12: fix round 1 dispatched (resume implementer) at FIX_BASE 71e03795
Task 12: fix commit cf720e2e; re-review dispatched (sonnet)
Task 12: fix round 1/5 (3 addressed, 1 open + 1 new — §4 visibility omits signal_render_visible; §7 view-distance direction inverted; commits 71e03795..cf720e2e)
Task 12: fix commit 911491a0; re-review dispatched (haiku-tier scope → sonnet floor)
Task 12: fix round 2/5 (2 addressed, 0 open; commits cf720e2e..911491a0)
Task 12: complete (commits c0e7589b..911491a0, review clean after 2 fix rounds)
Task 13: BASE 911491a0
Task 13: Ruling (I2 applied): the sheet's second pane is 400×900 (equal heights) so the swing is exactly horizontal; still_sheet_frames asserts |advancing.y| < 1e-9 — the owner judges horizontal reveals as the plan describes — cost if wrong: the sheet's second pane differs from the plan's 400×300.
Task 13: Ruling (I3 applied): the control's κ comes from the 1400×900 pane's slab by window id (camera at 2000 px as the spike's) — cost: none.
Task 13: Ruling: the implementer stops at the stills commit; the controller parks for the owner after the task review — the park is a gate the controller owns.
Task 13: implementer dispatched (opus); agent a80e89daf0761ef50
Task 13: implementer DONE_WITH_CONCERNS ce2da60d (EV stills-20261010T0009; swing (−1,0); κ control ≈1.42; sheet fixes after viewing; pane_swing absent → view_swing_start)
Task 13: concern: S2 (spill on) wall patch is darker than S1 (spill off): mean RGB (27.7,31.0,34.4) vs (28.7,32.4,36.3), up to 20 levels in a 152×40 px patch — investigation dispatched before the owner sees the sheet; review dispatched (opus) in parallel
Task 13: spill-darkening investigation (opus, read-only, agent a0783d132060d21e2) and task review (opus, agent a93f4d5283815a98d) running in parallel
Task 13: investigation result: not a shader defect. ring-beam-noise 0.55 wanders the beam head; its seed is jelly_seed (process-wide JELLY_SEED counter, material/mod.rs:200/:797) ^ beam start (tile.rs:943); sheet_frame builds a fresh fixture per frame, so every sheet frame (S1, S2 and all cells) draws a different head brightness. The wall spill contributes 0 at this pose/time (wall only at the left advancing edge; beam head at top centre); with noise 0 S1/S2/S1 are bit-identical.
Task 13: Ruling: (1) pin the seed in the harness: a #[cfg(test)] hook resets JELLY_SEED at the start of each sheet fixture so every frame of the sheet gets the same seed and keeps the accepted look's wander; (2) S1/S2 use a pose/time where the beam head crosses a wall (found with the −1 wall probe), and still_sheet_frames asserts S1 and S2 differ only on wall-probe pixels and S2 ≥ S1 there — the owner's spill choice and the cell comparison are meaningless otherwise — cost: one test hook beside view::hooks.
Task 13: review: Needs fixes — Important 2 (S1/S2 presented as added light on a wall but every changed pixel is darker on the top rim; per-cell walls ≤ 3 px are invisible at sheet scale), Minor 7; ⚠️ diagonal corner wall not visible in G/H; ⚠️ control slab from the inactive material (width independent of thickness per report)
Task 13: Ruling: promote Minor 3 (forced pose tilts the second window inside K/G/H), Minor 7 (K under the κ 10 header) and Minor 5 (sheet_material .replace fails silently) into the fix round — the first two mislead the owner at the gate, the third breaks the owner's fail-early rule — cost: small script/test edits.
Task 13: minor (deferred): rest-light, spill-off-light, control-light dumps unused by the script
Task 13: minor (deferred): task note records a literal /mnt/ssd3 stills path (repo convention for $EV notes)
Task 13: fix round 1 dispatched (resume implementer) at FIX_BASE ce2da60d
Task 13: fix commit a46ff9c3 (EV stills-20261010T0056; seed pinned, byte-identical rerender; S1/S2 identical at B2 for every sampled time — labelled so; spill shown on forced top-lift pose crops I/J, +11 levels on 1207 wall px; per-cell 4× edge grid; second-window crop at x 1448; K below a rule κ≈1.42; anchors asserted; corner wall 2/1 px); re-review dispatched (opus)
Task 13: fix round 1/5 (6 addressed, 0 open; commits ce2da60d..a46ff9c3)
Task 13: minor (deferred): S1/S2 "at 30 ms" not explained on the sheet; spill row does not say I/J are forced; I/J difference (2 rows, ≤ 11 levels) barely visible at 4× — consider an amplified difference tile
Task 13: review clean after 1 fix round; parked for the owner's verdict (controller)
Ruling: run the final whole-branch review now over 01dff3b7..HEAD (Tasks 1–13), while the owner judges the stills; Tasks 14–15 (cost script, clip script, evidence, possible contingency) get a scoped final review when they land — every code task is reviewed and the remaining tasks are gated on a person and an idle host — cost if wrong: a second, smaller final review pass later.
Final review: dispatched (opus) over 01dff3b7..9ba45d1a
Final review (Tasks 1–13): With fixes — Critical 0, Important 1 (I1: performance.md:209 claims "nothing at rest" before Task 14 measured the rest cost; hold merge until Task 14's verdict), Minor 8; all 37 deferred minors "leave"; every ruling agreed (caveats: m5/m6 rest on Task 14 passing; Task 13 ruling (2) superseded by forced-pose crops I/J since no B2 time has the head on a wall)
Ruling: the merge is held until Task 14's rest-path verdict (and Task 15's clip) — the branch already cannot merge before its remaining tasks; Task 14's contingency applies if the rest path costs — cost if wrong: none.
Task 13 ruling (2) note: superseded — at B2 no time puts the beam head on a lifted wall; S1/S2 are labelled identical and the spill is shown on a forced top-lift pose (I/J); the owner is told the spill never shows on a real horizontal B2 swing.
Final review: minor (deferred): M1 fullscreen/maximize toggled mid-swing keeps swinging (tile.rs:720, :982) — cut or document
Final review: minor (deferred): M2 render-pipeline.md:162 / evidence :49 imply slowdown stretches the beam (it does not)
Final review: minor (deferred): M3 render-pipeline.md:156 omits top hits on the bevel from outline coverage
Final review: minor (deferred): M4 unpinned GLSL lines (X inverse profile prelude:539, jy :701, g_decal_ok :489, centre-line at, bisection condition, viewHit face test, g_r)
Final review: minor (deferred): M5 no end-to-end check that grain/jitter stay on the glass under a swing (Normal and 90° outputs)
Final review: minor (deferred): M6 FocusOrigin holds a strong Output (focus_origin.rs:17); use WeakOutput or clear on removal
Final review: minor (deferred): M7 per-pass focus_with_output/tile_centre_on_output scan and 5 uniforms per draw at rest — for Task 14 to account
Final review: minor (deferred): M8 culling footprint uses buf_bbox vs physical-rounded offscreen geometry; few px near the clip
Final review: corrective round 1 dispatched (fix subagent, I1 doc fix)
Final review: corrective round 1/5 (I1 addressed, 0 open; commits 9ba45d1a..4362b107) — Tasks 1–13 final review clean
Resume point: Task 13 (material-7e3447) parked on the owner's stills verdict; then Task 14 (material-2283b8, needs quiet+nested) and Task 15 (material-c5f9d0, needs quiet+nested+owner); then a scoped final review of Tasks 14–15's diff; then finishing-a-development-branch. Keep this workspace until then.
Task 13: owner verdict B2, S2 recorded; complete (commits 911491a0..c7576a73, review clean after 1 fix round, owner accepted)
Task 14: BASE c7576a73
Task 14: implementer dispatched (sonnet): Step 1 script + one pilot attempt (expected headless refusal); m19 zones helper once; controller parks; agent a73a7ad6d5ce3ad07
Task 14: implementer DONE_WITH_CONCERNS 2e35382c (script + zones helper); pilot refused by await_load after 13 min (cold builds 8 min, load wait 5); run: note recorded; review dispatched (sonnet)
Task 14: review: Approved — Minor 7; ⚠️ swing eligibility under the nested config / baseline accepts b configs / baseline trace has the draw zone (the pilot settles these)
Task 14: Ruling: promote the swing case's 30 s trace margin (8 gains end at ~29 s), the zones helper's missing --input in capture_identity, and the timing header note into a fix — the pilot cannot reveal the margin and a late failure costs a full idle-host run — cost: a few lines.
Task 14: minor (deferred): final PASS line prints regardless of verdicts; zones helper drops the sibling's column/finite checks and the sharp_damage assert; probe-first-focus comment wrong (no swing during open animation); TERM to the script alone deferred until the foreground child ends
Task 14: fix commit 37604b97 (GAINS 6 full, per-gain sleep_until deadlines, --input zones helper, header timing note); re-review dispatched (sonnet)
Task 14: fix round 1/5 (3 addressed, 0 open; commits 2e35382c..37604b97); script reviewed clean; parked --reason quiet --needs headless --minutes 40 (pilot then full run); Task 15 (material-c5f9d0) waits on it
Resume point: Task 14 quiet run (pilot, full, judge; contingency on FAIL) → Task 15 (owner clip, nested lane, quiet + owner) → scoped final review of the Tasks 14–15 diff (37604b97 onward plus their evidence) → finishing-a-development-branch; keep this workspace.
Rewrite 2026-10-16 (material-5184bf): every commit ID above is pre-rewrite; trees are unchanged but old IDs are no longer ancestors of HEAD, so never use them in a range. Live ones: branch BASE 01dff3b7=e0d26fe9, Task 14 BASE c7576a73=955b36a7, 2e35382c=393f73ae, 37604b97=6f2ce207 (scoped final review: 6f2ce207 onward), 4362b107=ab3d2021. Full map: tasks/files/material-5184bf/2026-10-16-identity-commit-map.txt on materials-26.04.
Task 15: owner delegated the amplitude pick (2026-10-29); agent picked L3 (view-tilt 20, view-distance 3); sign and settle unchanged; recorded under Clip in fe44adca; material-c5f9d0 done; material-fb9efc filed for the L2 far-edge motion and the non-linear pull-in
Scoped final review: dispatched (opus) over 6f2ce207..fe44adca (Tasks 14-15), package review-final-6f2ce207..fe44adca.diff
Scoped final review (Tasks 14-15): With fixes — Critical 0, Important 2, Minor 9; reviewer claude-code/claude-opus-5-5; code split sound (rest program = base text but one comment)
Ruling: I1 accepted — the Clip analysis compared against a hinge formula; spec §3.2's centre pivot (D = κE/w) predicts the ladder within ~1 px except L2's far edge (+~2 px); evidence Clip rewritten with the model, material-fb9efc rescoped to that residual, L3 kept as a look judgement (L2 reads as shrinking, L3 as turning) — cost if wrong: the evidence keeps misstating the view path's geometry
Ruling: I2 accepted — the delegation covered amplitude only; sign and view-tilt-settle stay with the owner; material-c5f9d0 reopened and parked for a quiet run of tilt-swing-slow and tilt-swing-flipped at TILT=20 KAPPA=3, then the owner's verdict; prism-2a8fea corrected; merge held until that verdict — cost if wrong: Prism ships an unjudged sign/settle
Scoped final review: minor (deferred): m1 render-pipeline.md:169 and evidence:20 say the rest text equals the base; at HEAD it differs by the prelude.frag:403 comment
Scoped final review: minor (deferred): m2 performance.md:209 "at rest: nothing" omits the start-up cost (~4.5 s cold, ~23 ms warm) every config pays
Scoped final review: minor (deferred): m3 a failed view-variant compile disables all glass even at view-tilt 0 (material/mod.rs:844-846, warn only)
Scoped final review: minor (deferred): m4 evidence:23/:118/:154 cite pre-rewrite 2f86b3ac/f548b835; name f3167120/bbc5ca3c beside them
Scoped final review: minor (deferred): m5 swing-measured-sheet.py:180 max(moving) raises on a run with no motion; hard-coded Arch font path :29; ring-motion-clips.sh requirements omit numpy
Scoped final review: minor (deferred): m6 evidence:168 cites swing-review-sheet.png, which no committed script produces (the 0.43 s and 8 px misstatements were fixed in the I1 rewrite)
Scoped final review: minor (deferred): m7 ring-motion-clips.sh:3 "Twelve sequences" (default runs eight); :61 calls 10/6 the owner's pick
Scoped final review: minor (deferred): m8 BOUND_PROGRAMS records the selected program before draw, not the bound one (material/mod.rs:1149); cfg(test) vector never drained outside the identity test
Scoped final review: minor (deferred): m9 material-2283b8's 2026-10-16 review note says round 1 (should be 3); material-c5f9d0's round 2 note says reviewer human, corrected in prose only
Scoped final review corrective round 1/5: re-review (claude-code/claude-opus-5-5) — I1, I2 resolved; new Important 1 (verdict blamed L2's foreshortening; the cause is κ 6's weaker perspective), Minor 4
Ruling: I-new1, m1 (receding-edge wording), m2 (frame-angle model column, fb9efc 2-4 px), m4 (wrap) fixed in round 2; m3 (park c5f9d0 with the run recipe) done after the round's commit — cost: none
Scoped final review corrective round 2/5: re-review (claude-code/claude-opus-5-5) of 764ecb06 — all prior findings resolved, Clean; Tasks 14-15 final review clean after 2 rounds
Scoped final review: minor (deferred): evidence Clip: lines 174/192 wrap unevenly (192 is 86 columns)
Scoped final review: minor (deferred): evidence Clip :186-187 gives the model only at the peak (D = κE); write D = κE/w and h cos θ / (1 ± w sin θ / 2κ), w = 1 at the peak, to match the frame-angle column
Scoped final review: minor (deferred): material-fb9efc lists frame timing as a cause; timing can narrow the residual from ~4 px to ~2 px but not below the θp bound's 2 px
Resume point: material-c5f9d0 parked for the sign/settle clips at 20/3 (quiet, headless), then the owner's verdict; then finishing-a-development-branch (merge main into the branch: conflicts in upstream-divergence.md and src/tests/client.rs per a 2026-10-29 trial merge-tree; main 624 ahead), close material-6901e0; keep this workspace until then.
Owner verdict 2026-11-08: shipped sign, 900 ms settle; recorded in 3deed475; material-c5f9d0 done; deferred minors filed as material-fc84e5 (45281a91)
Base merge: materials-26.04 (684 ahead) merged into the branch at 418b0130; conflicts: upstream-divergence.md (regenerated by upstream-report --stage), src/tests/client.rs (both sides kept; duplicated subcompositor field/bind/Dispatch impls from independent additions removed, main's kept); rustfmt rewrapped view/solid.rs comments; pre-commit full route passed
Base merge review (418b0130): accept, Minor 1 (performance.md:209 cost figures predate material-03fdf3's grain speed-up); report merge-review-report.md; just test at 418b0130: 831 passed, 0 failed (incl. doctests)
Merge review: minor (deferred): performance.md:209 rigid tilt cost figures (0.66 ms at rest, 1.47 ms swing) were measured before material-03fdf3's grain shader speed-up; re-measure or label the revision
