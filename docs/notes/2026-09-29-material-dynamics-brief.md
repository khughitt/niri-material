# Material dynamics scoping brief

Updated 2026-10-06. Handoff for `material-53f873`, not an approved design.

## Problem

Give glass convincing movement and warm changing light while settling completely.
This pass covers the five oldest eligible ideas in the dynamics lane:
`material-5a5fff`, `material-6d4de5`, `material-1c5a30`, `material-f3e4e4`
and `material-9be53d`. Completed research narrows the movement work; lighting
still needs a bounded experiment before implementation.

## Current behaviour and evidence

### Drag baseline finding (`material-b3ce14`)

`material-4354cf` shipped drag follow-lag (`f8a890d3`); the owner accepted
its clips on 2026-10-02. `src/layout/tile.rs::motion_residual` now combines
native animation residuals and drag lag. `material_dynamics` still caps flex
at one quarter of bevel depth; `jelly_state` saturates it, while ripple
perturbs the normal before refraction. Cap, whole-slab shear and ripple
scaling remain separate questions. The [motion sweep](../materials/2026-09-11-jelly-motion-sweep.md)
measures response and settling, not a preferred deformation.

### Focus swap finding (`material-8e3b73`)

The owner kept hard focus cuts (`7f5ee9e2`, 2026-10-02).
`src/render_helpers/material/mod.rs::apply_resolved` updates one definition
in place and replaces state, seed and offscreen on a name swap; layout motion
survives. Named responses and `material-signal` crossfades already exist.
[Detailed drag and focus findings](2026-10-06-material-dynamics-findings.md)
retain the original traces, parameter inventory and capture provenance.

### Organic light feasibility

`src/render_helpers/material/optics/aurora.rs` has amount, two colors and
bucketed drift over a 600 s field loop. A moving warm field is a candidate;
this does not establish a cloudy-to-sunlight brightness envelope, particles
or mouse interaction. The registry suppresses optic deadlines while settled
and holds rendered values through idle damage (`63702813`).
The [power re-run](../materials/2026-09-11-idle-budget-evidence.md#re-run-on-48ba40a1)
measured Aurora at +0.96 W (4 Hz) and +0.77 W (2 Hz) in its pinned scene;
those are not estimates for new lighting. The three referenced mindful
thoughts remain unavailable locally; their task bodies and sources are preserved.

## Constraints

Preserve the [render order](../materials/render-pipeline.md), opaque client
pixels, finite settling, visibility and reduced/off motion behavior, and the
[accepted sustained-optic idle contract](../specs/2026-09-29-sustained-optic-settling-design.md).
`material-39a46f`'s power verification and `material-0db905`'s settling
design are complete; do not reopen those questions. Fireflies
`material-54bcac` and ambient transport `material-f41c54` keep their existing
goals. `material-77db8a` still awaits owner review. No host capture is needed
for the new feasibility task; later measured runs declare `quiet` and run a pilot first.

## Alternatives

1. **Current lean: reuse existing animations, responses and optics.** Keep
   the accepted focus cut and drag response; examine one warm-light experiment.
2. Design a targeted deformation or light envelope only after a demonstrated
   gap establishes the required look, timing and checks.
3. Build named animation profiles or a general state machine now. Shelve
   that approach until a wanted behavior exceeds current mechanisms.

## Unanswered questions

- **Deformation:** which current cap, shear or ripple appearance is unacceptable?
  The owner supplies a concrete example before `material-6d4de5` becomes a design task.
- **Lighting:** what can current optics express, and which finite light or
  weather experiment has the smallest missing behavior? `material-ffd61f`
  answers feasibility; the owner later judges the look and acceptable ongoing cost.
- **Clock:** should weather follow activity, real time or an event, and what
  should input inactivity freeze? The feasibility proposal frames these choices;
  no weather clock or ambient-source ownership is chosen here.

## Proposed decomposition

- `material-5a5fff`: propose drop, supported by `material-8e3b73` and
  `7f5ee9e2`; the idea remains open pending disposition.
- `material-6d4de5`: briefed, limited to the remaining deformation questions;
  its original captured request is retained below the updated body.
- `material-1c5a30` and `material-f3e4e4`: briefed, waiting on
  `material-ffd61f` (P2, small, mid complexity, direct): trace current optics
  and propose one bounded organic-light experiment, with no rendering changes
  or capture run. Completion updates this brief and adds finding notes to both ideas.
- `material-9be53d`: shelved until a reproducible wanted behavior and acceptance
  check demonstrate a gap in existing animations, responses or optic clocks.
- Reuse the existing lane; no new goal or animation subsystem. Historical
  baseline tasks `material-b3ce14` and `material-8e3b73` are complete.
  Previously shelved `material-e6036d` remains related context, untouched.
