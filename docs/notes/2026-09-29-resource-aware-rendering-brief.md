# Resource-aware rendering: scoping brief

## Problem

Reduce material work that nobody can see, settle optional continuous motion
after input inactivity, and choose visual concessions from measured savings.
The 2026-10-06 follow-up pass covers `material-7afc31`, `material-7f6d0e`
and `material-f6e284`, under `material-5d6b2c`. Earlier handoffs for
`material-f86183`, `material-2ebf2c`, `material-a0cbb0` and
`material-a91346` remain associated here.
Sustained-optic settling now has an accepted design, implementation and
headless evidence; its remaining two-output lifecycle capture stays open
under `material-f86183`.

## Current behaviour and evidence

- Attention already gates on visibility and input activity. The
  [ring-motion design](../specs/2026-09-18-ring-focus-motion-design.md)
  §§2–3 remains in force; its focus sweep was subsequently replaced by the
  beam. `signal { idle-after-ms }` defaults to 30000; 0 disables the gate.
  The [smoke evidence](../materials/2026-09-18-ring-focus-motion-evidence.md)
  records zero redraws (its counts are `Niri::redraw`, not material draws)
  for hidden workspaces, hidden tabs, offscreen columns, DPMS-off and idle
  attention, plus a successful input resume.
- [Hidden-window attribution](../materials/2026-09-30-hidden-window-attribution-evidence.md)
  (`7f5746b6`) measured zero hidden material draws and prefilter rebuilds,
  and about 1 Hz hidden-client frame pacing. Hidden commits still queued
  output redraws; offscreen columns and opaquely covered tiles each rendered
  their window offscreen once per commit. The correction in `66e6b552`
  establishes that this preparation runs in `Tile::render_inner` before
  smithay's occlusion pass: an opaque-region declaration cannot skip it.
  `ScrollingSpace::render` visits every column; its `visible` gate skips
  hidden tabs, with an exception for alpha animations.
- `Tile::render` reports deadlines before preparing elements;
  `Tile::tick_deadline` tests slab/view intersection, not opaque coverage.
  Covered sustained-optic redraws remain unmeasured. `src/niri.rs` uses
  offscreen element bookkeeping for primary-scanout visibility; a culling
  change must preserve that bookkeeping and reveal damage.
- [Idle-budget evidence](../materials/2026-09-11-idle-budget-evidence.md)
  establishes finite-motion quiescence: the full trace on `721b8df7`
  passed 25/25 observations, including a 600 s hold with zero redraws/draws.
  The idle-edge fix at that commit avoids redraws when no attention motion
  changes. [Sustained-optic settling evidence](../materials/2026-09-30-optic-settling-evidence.md)
  now records active Aurora at 4 Hz (2 Hz reduced), followed by a 601.9 s hold
  with no recurring redraws or material draws and phase-continuous resume.
  Aurora uses the shared pausable logical timeline and the same 30000 ms
  input-idle threshold as attention; 0 disables both gates. The owner accepted
  the idle/resume clip on 2026-10-02. The remaining lifecycle task is
  `material-1af3c6`, the two-output removal capture.
  No watt saving is established by these captures; client damage continues
  with held optics, so video-case savings are expected to be negligible.
- Existing Tracy zones include `Niri::redraw` and the GPU zone
  `MaterialRenderElement::draw`. `niri-ipc/src/lib.rs` has no render-cost
  request/event. [The pipeline](../materials/render-pipeline.md) already
  supports separate focused/unfocused materials; blur and roughness pyramids
  are cached per output/target in `src/render_helpers/effect_buffer.rs`.
- `MaterialRenderElement` implements neither `damage_since` nor
  `opaque_regions`. Its per-target fingerprint includes the client commit,
  backdrop, mapping and visual inputs. `OffscreenRenderElement` has damage
  history but also declares no opacity: forwarding either fact needs a
  conservative coordinate and invalidation contract, not just a trait method.
- `docs/materials/adding-an-optic.md` section 4 already asks for frame-cost
  proof, but defines neither a matched baseline nor a cost-entry template.
  [The performance guide](../materials/performance.md) (`66e6b552`) supplies
  the units and shared-pass cautions needed for that documentation task.
- [The parameter sweep](../materials/2026-09-06-glass-parameter-sweep-evidence.md)
  and `docs/materials/scripts/glass-parameter-sweep.sh` use cropped Lab RMSE.
  Its 20 px periodic backdrop aliases displacement; `material-8867aa` and
  `material-343f27` retain that instrument question. Image difference alone
  establishes neither acceptable quality loss nor cheaper rendering.

The shared mindful source `a476e6bcd1fd4297b70824758235d821` could not be
retrieved locally. Preserved task bodies and notes supply the available intent.

## Constraints

Keep attention's static indicators and finite beams/impulses; preserve windows
visible on other lit outputs, overview/transitions and real client damage.
Input inactivity differs from the signal's Quiet level. Reuse the capture
protocol, its provenance and lane preflight, with an end-to-end pilot before
a longer matrix. Host GPU utilization cannot identify compositor work.
Reuse `material-31074f` for cost measurement; shared cache costs prevent assuming
that lowering one unfocused window's blur saves a whole blur pass.
The new boundary audit is source-only and needs no idle host. Later captures
declare `quiet` and use a nested compositor where possible; preparation is
separate from the capture. No global cost threshold is established.

## Alternatives

1. **Lean: audit the two boundaries before changing rendering.** Hidden
   offscreen work is reproduced, but safe early culling and final-draw damage
   narrowing need different information. Establish the smallest safe change
   and verification matrix first. Record cost entries with matched baselines
   and uncertainty; defer numeric regression gates until budgets are justified.
2. **Continuous observation.** Start by establishing whether consumers need
   redraw/pass counts or actual GPU time. An always-on GPU-query IPC collector
   needs defined units, sampling and overhead evidence; existing Tracy serves
   the current experiments.
3. **Inactive quality concessions.** Compare existing focus-split settings
   using measured whole-scene cost, repeatability and owner-reviewed images.
   Choose a stronger perceptual metric only if Lab RMSE and those comparisons
   fail to support the decision; do not select LPIPS/SSIM or tiers by name alone.

## Unanswered questions

- Does a hidden/covered material cause compositor work beyond the known gates,
  or is the reported load client-driven? Answered by `material-d09741`
  ([evidence](../materials/2026-09-30-hidden-window-attribution-evidence.md)):
  hidden material tiles draw nothing, the prefilter is never rebuilt, and the
  hidden client drops to the 1 Hz fallback cadence. The residue is one output
  redraw per hidden-client commit and, for an offscreen column or a tile under
  an opaque window, an offscreen-buffer re-render per commit that nothing
  draws. A sustained optic under an opaque cover and a second lit output were
  not captured.
- Sustained-optic participation and resume are settled by the accepted
  `material-0db905` design: Aurora participates by default, freezes logical
  time and resumes without catch-up or easing. Remaining lifecycle evidence
  now consists of `material-1af3c6`; the other lifecycle and review-minor
  children are closed in the current task tree.
- Where can visibility/coverage be known before offscreen preparation and
  deadline collection without dropping visible popups, slab bands, animations,
  snapshots or another render target? What reveal and frame-callback state must
  survive a skip? `material-82e4bc` answers from source and specifies the
  covered-optic capture; it does not claim new measurements.
- Can client-only damage and proven opaque content survive offscreen wrapping,
  fractional scale and resize/jelly transforms? Which input changes require
  full damage? `material-82e4bc` audits these facts separately from early culling.
- Should a cost threshold block an optic? Current recommendation: retain visual
  and capture-preflight verdicts, record cost deltas, and defer numeric gates
  until `material-31074f` establishes repeatability and a budget is accepted.
- Which inactive settings save measurable scene cost? `material-31074f` supplies
  measurements; the owner judges acceptable visual changes using review images.
- What continuous quantities do consumers require, and at what overhead budget?
  The cost study should record the need before an IPC design is justified.
- Does Lab RMSE miss a relevant visual difference? A bounded comparison of
  repeat captures, candidate reductions and owner judgment can answer this once
  there is a measured trade-off to evaluate; displacement needs an unaliased scene.

## Proposed decomposition

- `material-d09741` — completed hidden-window attribution; its findings woke
  `material-7afc31`. Keep its measured residue and unmeasured cases distinct.
- `material-82e4bc` — P1, medium, high complexity, direct source audit of safe
  culling and damage/opacity boundaries; wakes `material-7afc31` and
  `material-7f6d0e` with findings in the same commit. No live capture or renderer
  patch; a reviewed design follows only if the audit establishes that need.
- `material-f6e284` — scoped P2, small, low complexity, direct documentation:
  a cost-entry template and one existing-evidence example in the optic recipe.
  Matched baselines, provenance, units, repeatability and shared-pass attribution
  are required; numeric gates remain deferred. No new capture is needed.
- `material-0db905` — completed sustained-optic design and plan; execution
  Tasks 1–4 are complete and the Task 5 headless evidence is accepted.
  `material-f86183` remains open for the acceptance follow-ups above.
- Reuse `material-31074f` — existing P2 cost study, with a wake-up note for
  `material-2ebf2c`, `material-a0cbb0` and `material-a91346`. Completion records
  findings on each idea in the same commit so a later scope pass can reconsider it.

Reuse the existing lane and cost study. The only new follow-up is the bounded
source audit; both rendering ideas remain ideas until its findings readmit them.
