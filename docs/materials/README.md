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
- `2026-08-22-v1-design.md`: v1 material architecture.
- `2026-08-24-glass-config-surface-design.md`: v1 configuration surface design.
- `2026-08-24-v1-parity-design.md`: frozen-reference parity design.
- `2026-08-25-v1-reference-static-preflight-design.md`: static-instrument correction design.
- `2026-08-27-v1-drm-acceptance-design.md`: physical DRM acceptance design.
- `2026-08-28-v1-daily-driver-rollout-design.md`: deployment and burn-in design, with the burn-in record.
- `2026-08-29-material-backdrop-blur-design.md`: backdrop blur design.
- `2026-08-29-material-backdrop-blur-evidence.md`: backdrop blur verification evidence.
- `2026-09-01-material-roughness-design.md`: native and Prism roughness design.
- `2026-09-02-material-roughness-smoke.md`: nested GLES, Tracy, damage-reuse, and calibrated overview evidence.
- `../specs/2026-09-02-material-noise-saturation-design.md`: native and Prism noise/saturation composition design.
- `2026-09-02-material-noise-saturation-evidence.md`: native automated, nested GLES, and Prism composition evidence.
- `../specs/2026-09-05-material-glass-noise-saturation-params-design.md`: optional per-material `noise` and `saturation` design; written values always apply.
- `2026-09-05-material-glass-noise-saturation-params-evidence.md`: nested GLES evidence that written noise and saturation render with backdrop blur off and under `blur { off }`.
- `scripts/glass-noise-saturation-smoke.sh`: headless harness for those captures.
- `../specs/2026-09-06-material-glass-noise-type-design.md`: optional `type=` on glass `noise`: white, fine (high-pass bell-shaped hash grain) and lightness (the same grain on Oklab L).
- `2026-09-06-material-glass-noise-type-evidence.md`: nested GLES evidence for the three types: each renders, fine has less low-frequency energy and a thinner top band than white, lightness holds Oklab chroma.
- `scripts/glass-noise-type-smoke.sh`: headless harness for those captures.
- `../specs/2026-09-06-glass-parameter-sweep-design.md`: parameter sweep script design; a measurement tool that reports where a glass parameter stops changing the render, not a gate.
- `../specs/2026-09-10-material-optics-design.md`: material optics API and generated parameter table.
- `adding-an-optic.md`: the recipe for a new pipeline stage.
- `2026-09-10-material-optics-evidence.md`: decoded-pixel-identical captures before and after the saturation and noise optic migration.
- `../specs/2026-09-30-glass-edge-optics-design.md`: the bevel as a height field (`bevel-profile`), Fresnel-weighted transmission, and the `reflection` and `edge-highlight` optics.
- `2026-10-03-glass-edge-optics-evidence.md`: frozen-clock render evidence for each step (outside and face identical, bevel changes reported) and the contact sheet.
- `scripts/capture/glass-edge-compare.py`: before/after region checks over the frozen-clock harness dumps.
- `scripts/capture/glass-edge-sheet.sh`: headless contact sheet of the probe's corner over a real wallpaper, crossing bevel-profile, looks, reflection, edge-highlight and roughness.
- `2026-09-06-glass-parameter-sweep-evidence.md`: sweep tables for `ior`, thickness, roughness, noise and saturation, and the global `blur` block, with a per-slider summary for Prism.
- `scripts/capture/glass-parameter-sweep.sh`: headless harness that renders one KDL parameter at N values and tabulates the neighbouring deltas.
- `2026-10-06-glass-warp-calibration-evidence.md`: offline calibration of a local displacement estimator against known warps and photometric-only controls; adopts it for the chamfer with a geometry mask, and names the flat-field render that separates bending from Fresnel and attenuation.
- `scripts/capture/warp-calibration.py`: that calibration, and the seeded aperiodic backdrop it measured.
- `2026-09-04-focus-glass-spike.md`: focus-state glass spike; transparent terminals over an `is-active` material swap remove the terminal seam.
- `2026-09-05-focus-ring-light-spike.md`: focus ring light spike; an embedded, refracted ring of light wins over motes, rays, shadow, noise, and canopy, and the static gradient ring is invisible on the dark focus glass.
- `2026-09-05-ring-light-focus-smoke.md`: ring of light focus response wakeup, capture and DRM acceptance evidence.
- `2026-09-02-material-signals-design.md`: per-window signal, IPC, and glass response design.
- `2026-09-03-material-signals-smoke.md`: suites, IPC, redraw, GPU, package, and physical DRM evidence.
- `../specs/2026-10-05-noise-placement-design.md`: noise at backdrop, glass or film, its shared scope and verification contract.
- `2026-10-05-noise-placement-evidence.md`: noise placement pixel, baseline, softening and cost evidence.
- `material-config.md`: material and glass configuration reference.
- `render-pipeline.md`: the material render pipeline, pass by pass, and which parameter acts where. Read before any rendering change.
- `performance.md`: where GPU, CPU, memory and power go in the material, which parameters are expensive, and the levers; a map onto the evidence docs.
- `scripts/focus-glass-spike.sh`: headless harness for the focus-state glass captures.
- `scripts/capture/focus-ring-light.sh`: headless harness for the focus ring light captures and the measured filament checks.
- `plans/2026-08-22-repository-migration.md`: repository migration procedure.
- `plans/2026-08-22-slice0.md`: renderer-seam plan.
- `plans/2026-08-22-slice1.md`: config-and-assignment plan.
- `plans/2026-08-23-slice2.md`: glass shader and composition plan.
- `plans/2026-08-24-slice3.md`: overview-crop correction plan.
- `plans/2026-08-24-glass-config-surface.md`: configuration-surface plan.
- `plans/2026-08-24-v1-parity.md`: frozen-reference parity plan.
- `plans/2026-08-26-v1-reference-static-preflight.md`: parity preflight and recapture plan.
- `plans/2026-08-27-v1-drm-acceptance.md`: physical DRM acceptance plan.
- `plans/2026-08-28-v1-daily-driver-rollout.md`: package and daily-driver rollout procedure.
- `plans/2026-08-29-material-backdrop-blur.md`: backdrop blur plan.
- `../plans/2026-09-01-material-roughness.md`: roughness implementation and verification plan.
- `../plans/2026-09-02-material-noise-saturation.md`: noise and saturation composition plan.
- `../plans/2026-09-02-material-signals.md`: material-signals plan.
- `../specs/2026-09-05-ring-light-focus-response-design.md`: ring of light focus response design.
- `../specs/2026-09-06-upstream-divergence-design.md`: upstream divergence document and rebase strategy design.
- `upstream-divergence.md`: what this fork changes relative to upstream, which changes are seam candidates, and the rebase procedure. Generated blocks come from `just upstream-report`; `upstream-baseline.toml` pins the baseline and `upstream-conflicts.toml` acknowledges expected conflicts.
- `../plans/2026-09-06-upstream-divergence.md`: implementation plan for that design, seven tasks from the baseline record to the weekly drift canary. The fork is permanent and upstreams seams only. Records the rewritten-ancestry trap: resolve the baseline from a pinned record and compare trees, never merge-base.
- `../plans/2026-09-05-ring-light-focus-response.md`: ring of light focus response implementation plan.

Research, results, and source fixtures live in the sibling
`niri-experiments` repository. Explicitly qualified pre-split evidence and the
frozen Quickshell reference remain in the legacy `niri-glass` repository.

## History and open work

Open material work is in the tracker (`tasks list --project material`). Each document
above carries its verification where it has one. The delivery history this file kept
until 2026-11-09 (the slices, the configuration surface and parity gate, DRM
acceptance, backdrop blur, the daily-driver burn-in, and noise and saturation
composition, with their commits and `niri-experiments` result commits) is preserved in
the project's records directory as docs/notes/2026-11-09-material-delivery-history.md.
