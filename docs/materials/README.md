# niri material

This branch carries the native material system inside the full niri fork.
Upstream project documentation remains in the repository root and `docs/wiki/`.

## Branches

- `patched-26.04`: niri 26.04 plus the two #4147 IPC commits.
- `materials-26.04`: production material work, stacked on `patched-26.04`.

All future production rebases and material commits happen in this repository.

## Material documentation

Every measurement run writes `capture.json` through `tools/capture-meta`; paste
`tools/capture-meta show <run-dir>` into the evidence doc's environment section
rather than retyping it (protocol:
`../specs/2026-09-11-material-capture-protocol-design.md`).

- `2026-08-22-repository-split.md`: repository ownership and provenance.
- `2026-08-22-v1-design.md`: accepted v1 material architecture.
- `2026-08-24-glass-config-surface-design.md`: implemented and accepted v1 configuration surface.
- `2026-08-24-v1-parity-design.md`: executed frozen-reference parity design.
- `2026-08-25-v1-reference-static-preflight-design.md`: implemented static-instrument correction.
- `2026-08-27-v1-drm-acceptance-design.md`: implemented and passed physical DRM acceptance design.
- `2026-08-28-v1-daily-driver-rollout-design.md`: implemented deployment and passed burn-in design.
- `2026-08-29-material-backdrop-blur-design.md`: implemented backdrop blur design.
- `2026-08-29-material-backdrop-blur-evidence.md`: retained backdrop blur verification evidence.
- `2026-09-01-material-roughness-design.md`: implemented and verified native and Prism roughness design.
- `2026-09-02-material-roughness-smoke.md`: passing nested GLES, Tracy, damage-reuse, and calibrated overview evidence.
- `../specs/2026-09-02-material-noise-saturation-design.md`: implemented native and Prism noise/saturation composition design.
- `2026-09-02-material-noise-saturation-evidence.md`: passing native automated, nested GLES, and Prism composition evidence.
- `../specs/2026-09-05-material-glass-noise-saturation-params-design.md`: optional per-material `noise` and `saturation` design; written values always apply.
- `2026-09-05-material-glass-noise-saturation-params-evidence.md`: passing nested GLES evidence that written noise and saturation render with backdrop blur off and under `blur { off }`.
- `scripts/glass-noise-saturation-smoke.sh`: headless harness for those captures.
- `../specs/2026-09-06-material-glass-noise-type-design.md`: optional `type=` on glass `noise`: white, fine (high-pass bell-shaped hash grain) and lightness (the same grain on Oklab L).
- `2026-09-06-material-glass-noise-type-evidence.md`: nested GLES evidence for the three types: each renders, fine has less low-frequency energy and a thinner top band than white, lightness holds Oklab chroma.
- `scripts/glass-noise-type-smoke.sh`: headless harness for those captures.
- `../specs/2026-09-06-glass-parameter-sweep-design.md`: parameter sweep script design; a measurement tool that reports where a glass parameter stops changing the render, not a gate.
- `../specs/2026-09-10-material-optics-design.md`: implemented material optics API and generated parameter table; iridescence, aurora, and their presets are merged into `materials-26.04`, while cracks, ice, and Prism mappings remain open.
- `adding-an-optic.md`: the recipe for a new pipeline stage.
- `2026-09-10-material-optics-evidence.md`: decoded-pixel-identical captures before and after the saturation and noise optic migration.
- `2026-09-06-glass-parameter-sweep-evidence.md`: sweep tables for `ior`, thickness, roughness, noise and saturation, and the global `blur` block, with a per-slider summary for Prism.
- `scripts/glass-parameter-sweep.sh`: headless harness that renders one KDL parameter at N values and tabulates the neighbouring deltas.
- `2026-09-04-focus-glass-spike.md`: passing focus-state glass spike; transparent terminals over an `is-active` material swap remove the terminal seam.
- `2026-09-05-focus-ring-light-spike.md`: focus ring light spike; an embedded, refracted ring of light wins over motes, rays, shadow, noise, and canopy, and the static gradient ring is invisible on the dark focus glass.
- `2026-09-05-ring-light-focus-smoke.md`: ring of light focus response wakeup, capture and DRM acceptance evidence.
- `2026-09-02-material-signals-design.md`: implemented and verified per-window signal, IPC, and glass response design.
- `2026-09-03-material-signals-smoke.md`: passing suites, IPC, redraw, GPU, package, and physical DRM evidence.
- `material-config.md`: material and glass configuration reference.
- `render-pipeline.md`: the material render pipeline, pass by pass, and which parameter acts where. Read before any rendering change.
- `scripts/focus-glass-spike.sh`: headless harness for the focus-state glass captures.
- `scripts/focus-ring-light.sh`: headless harness for the focus ring light captures and the measured filament checks.
- `plans/2026-08-22-repository-migration.md`: repository migration procedure.
- `plans/2026-08-22-slice0.md`: implemented renderer-seam plan.
- `plans/2026-08-22-slice1.md`: implemented config-and-assignment plan.
- `plans/2026-08-23-slice2.md`: implemented glass shader and composition plan.
- `plans/2026-08-24-slice3.md`: completed overview-crop correction plan.
- `plans/2026-08-24-glass-config-surface.md`: executed configuration-surface plan.
- `plans/2026-08-24-v1-parity.md`: executed frozen-reference parity plan.
- `plans/2026-08-26-v1-reference-static-preflight.md`: executed parity preflight and recapture plan.
- `plans/2026-08-27-v1-drm-acceptance.md`: executed physical DRM acceptance plan.
- `plans/2026-08-28-v1-daily-driver-rollout.md`: executed package and daily-driver rollout procedure.
- `plans/2026-08-29-material-backdrop-blur.md`: executed backdrop blur plan.
- `../plans/2026-09-01-material-roughness.md`: executed roughness implementation and verification plan.
- `../plans/2026-09-02-material-noise-saturation.md`: executed noise and saturation composition plan.
- `../plans/2026-09-02-material-signals.md`: executed and verified material-signals plan.
- `../specs/2026-09-05-ring-light-focus-response-design.md`: approved ring of light focus response design.
- `../specs/2026-09-06-upstream-divergence-design.md`: upstream divergence document and rebase strategy, implemented.
- `upstream-divergence.md`: what this fork changes relative to upstream, which changes are seam candidates, and the rebase procedure. Generated blocks come from `just upstream-report`; `upstream-baseline.toml` pins the baseline and `upstream-conflicts.toml` acknowledges expected conflicts.
- `../plans/2026-09-06-upstream-divergence.md`: implementation plan for that design, seven tasks from the baseline record to the weekly drift canary. The fork is permanent and upstreams seams only. Records the rewritten-ancestry trap: resolve the baseline from a pinned record and compare trees, never merge-base.
- `../plans/2026-09-05-ring-light-focus-response.md`: ring of light focus response implementation plan.

Research, results, and source fixtures live in the sibling
`niri-experiments` repository. Explicitly qualified pre-split evidence and the
frozen Quickshell reference remain in the legacy `niri-glass` repository.

## Progress

Slice 1 is merged into `materials-26.04` at `ec0824c5`. Its nested-winit
verification, run against `2a55ab14`, is recorded at `niri-experiments`
commit `d155e90` in `docs/results/2026-08-22-slice1.md`.

Slice 2 final-review fix `7e287517` is an ancestor of `materials-26.04`. Its
accepted nested-winit verification, including the final Slice 1
per-render-target fix, the transparent-workspace regression, and the matched
opaque final-fix pair is recorded at `niri-experiments` commit `a27eb8f` in
`docs/results/2026-08-23-slice2.md`; it closes the earlier texture-creation
and matched GPU-time measurement debt.

Slice 3 production fix `01a4259c` is an ancestor of `materials-26.04`. Its
nested-winit verification — characterization, live verification, performance
record, and final write-up — is recorded at `niri-experiments` commit
`3038246` in `docs/results/2026-08-24-slice3.md` (building on characterization
at `c37c653` and live verification + Tracy record at `50fb0e8`). The
coordinate-space half of the original G9 HACK premise did not reproduce on
current shaders; the shipped fix instead opens the strip-end per-workspace
crop bounds.

The glass config surface is implemented in `e579dae5`, `0c5f809f`,
`6cd06de2`, `73a733db`, and `b8fe7b84`. Two independent corrected
frozen-reference captures pinned to `niri-experiments` capture base
`b851e5208b54cc466d99bf3ae664cc5a52c2317f`, then graded at result commit
`c4b71a4ebfbe3c82c56f964bfc24d4f7de1bde4f`, passed integrity, all 28
implementation rows, and all 14 combined parameters. The config-surface
review and parity gate are complete. The corrected physical DRM run passed all
19 machine gates and all nine physical observations at `niri-experiments`
result commit `c0caa944db2edc5dc4844e6720951d7f32652b76`. Native
materials v1 is accepted.
A frozen-reference geometry-only preflight at `niri-experiments` commit
`5d3dfd26c4d4a4ef2c464c1d649dba533ccb27c5` subsequently passed 3/3 at
80-pixel gaps, identifying the recorded shift failures as output clipping.
The corrected complete capture incorporates that geometry scene.

The `backdrop-blur` glass parameter is implemented and nested-verified at
`eb4f1bd6`; the material refracts the blurred backdrop rather than the sharp
one, gated by the global `blur` block. Its evidence is recorded in
`2026-08-29-material-backdrop-blur-evidence.md`. The default-preserves-v1 DRM
acceptance regression passed 2026-09-01 on all 19 machine gates and all nine
physical observations, reproducing every accepted steady-state metric exactly.

Daily-driver deployment and burn-in passed 2026-08-30 against `52f74f10`, with
the package, Prism ownership handoff, two cold starts, journal review, and
operator PASS recorded in `2026-08-28-v1-daily-driver-rollout-design.md`.
The accepted burn-in also exposed an open cosmetic defect: an interactively
dragged window loses frost while stationary windows reading the same texture do
not. The investigation is narrowed to sampling geometry and remains active in
`2026-08-29-material-backdrop-blur-design.md`.

Global blur noise and saturation compose in native glass at `7c702e58` when
backdrop blur is effective. Nested GLES acceptance is recorded in
`2026-09-02-material-noise-saturation-evidence.md`; Prism description commit
`bcedf4cd` is on its requested `main` checkout.
