# Resource-aware rendering: scoping brief

## Problem

Reduce material work that nobody can see, settle optional continuous motion
after input inactivity, and choose visual concessions from measured savings.
This pass covers `material-7afc31`, `material-f86183`, `material-2ebf2c`,
`material-a0cbb0` and `material-a91346`, under `material-5d6b2c`.
Sustained-optic settling now has an accepted design, implementation and
headless evidence; its remaining lifecycle captures and review minors stay
open under `material-f86183`.

## Current behaviour and evidence

- Attention already gates on visibility and input activity. The
  [ring-motion design](../specs/2026-09-18-ring-focus-motion-design.md)
  §§2–3 remains in force; its focus sweep was subsequently replaced by the
  beam. `signal { idle-after-ms }` defaults to 30000; 0 disables the gate.
  The [smoke evidence](../materials/2026-09-18-ring-focus-motion-evidence.md)
  records zero redraws (its counts are `Niri::redraw`, not material draws)
  for hidden workspaces, hidden tabs, offscreen columns, DPMS-off and idle
  attention, plus a successful input resume.
- `src/layout/monitor.rs::update_render_elements` clears tile visibility
  before updating rendered workspaces. `Tile::render` reports deadlines;
  `Tile::tick_deadline` rejects an out-of-view slab. This establishes gates,
  but does not settle general occlusion, shared prefilter work or the reported
  hidden Kitty GPU activity. `src/niri.rs::send_frame_callbacks` uses primary
  scanout visibility; its fallback timer still services clients at about 1 Hz.
- [Idle-budget evidence](../materials/2026-09-11-idle-budget-evidence.md)
  establishes finite-motion quiescence: the full trace on `721b8df7`
  passed 25/25 observations, including a 600 s hold with zero redraws/draws.
  The idle-edge fix at that commit avoids redraws when no attention motion
  changes. [Sustained-optic settling evidence](../materials/2026-09-30-optic-settling-evidence.md)
  now records active Aurora at 4 Hz (2 Hz reduced), followed by a 601.9 s hold
  with no recurring redraws or material draws and phase-continuous resume.
  Aurora uses the shared pausable logical timeline and the same 30000 ms
  input-idle threshold as attention; 0 disables both gates. The owner accepted
  the idle/resume clip on 2026-10-02. Five lifecycle cases remain unverified.
  No watt saving is established by these captures; client damage continues
  with held optics, so video-case savings are expected to be negligible.
- Existing Tracy zones include `Niri::redraw` and the GPU zone
  `MaterialRenderElement::draw`. `niri-ipc/src/lib.rs` has no render-cost
  request/event. [The pipeline](../materials/render-pipeline.md) already
  supports separate focused/unfocused materials; blur and roughness pyramids
  are cached per output/target in `src/render_helpers/effect_buffer.rs`.
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

## Alternatives

1. **Lean: existing gates and bounded attribution first.** Reproduce any
   remaining hidden-window work before changing visibility. Design sustained
   optic settling around the existing activity flag; retain the 30 s threshold
   as the starting point, with participation and phase behavior reviewed.
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
  belongs to `material-f7eb0b`, `material-3acc86`, `material-80caf4` and
  `material-1af3c6`; deferred review minors belong to `material-285f81`.
- Which inactive settings save measurable scene cost? `material-31074f` supplies
  measurements; the owner judges acceptable visual changes using review images.
- What continuous quantities do consumers require, and at what overhead budget?
  The cost study should record the need before an IPC design is justified.
- Does Lab RMSE miss a relevant visual difference? A bounded comparison of
  repeat captures, candidate reductions and owner judgment can answer this once
  there is a measured trade-off to evaluate; displacement needs an unaliased scene.

## Proposed decomposition

- `material-d09741` — P1, small, mid complexity, direct attribution research;
  wakes `material-7afc31`. Recommend a focused fix only for a reproduced gap.
- `material-0db905` — completed sustained-optic design and plan; execution
  Tasks 1–4 are complete and the Task 5 headless evidence is accepted.
  `material-f86183` remains open for the acceptance follow-ups above.
- Reuse `material-31074f` — existing P2 cost study, with a wake-up note for
  `material-2ebf2c`, `material-a0cbb0` and `material-a91346`. Completion records
  findings on each idea in the same commit so a later scope pass can reconsider it.

No new goal, duplicate cost study or implementation task is needed in this pass.
