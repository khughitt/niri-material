---
id: material-415d0c
title: Design an Aurora envelope for one finite overcast-to-sun transition
status: todo
priority: 2
size: m
complexity: high
process: planned
needs: [owner]
created: 2026-10-17T02:04:23Z
updated: 2026-10-19T15:10:07Z
depends: []
parent: material-53f873
tags: [rendering, lighting]
source: "docs/notes/2026-09-29-material-dynamics-brief.md#organic-light-feasibility-material-ffd61f"
agent: claude-code
---

Design one finite overcast-to-sun Aurora transition: an envelope that eases a material's Aurora amount and colors from the rendered values to newly resolved ones on an in-place definition change, ending exactly on them. Research material-ffd61f found this is the only missing behavior for the warm organic-light experiment: Aurora amount and colors are per-definition constants today (AuroraOptic::values reads ResolvedAurora) and an in-place edit cuts (apply_resolved). The endpoints are reusable presets now.

Done: a written design and implementation plan, reviewed by the owner before renderer code. Start from the proposal in docs/notes/2026-09-29-material-dynamics-brief.md#organic-light-feasibility-material-ffd61f, which already frames trigger (in-place amount/color change on a same-named material; name swaps and drift-hz changes keep cutting), duration (6 s on niri's Animation, clips compare 3/6/12 s), reduced/off motion, hidden and input-idle behavior, pilot scenes S1–S12 (S10–S12 cross amount 0), neutral/overcast/sun controls and the deterministic checks (byte-equal endpoints against seed-matched controls, continuity through zero, tile are_animations_ongoing frame requests, settling, idle, motion policy, neutral). The design must settle:
- the clock: ordinary Animation clock (finite through input idle, as crossfades) versus the optic timeline (holds a change landing at idle);
- mid-transition retrigger from rendered values, as crossfade_origin does;
- that Aurora's rate, phase and next_change read the rendered amount, not the resolved target: AuroraOptic::rate returns 0 for amount 0, which pins the phase to 0 and arms no deadline, so a fade-out keyed on its target would jump the field and stall the drift while still lit;
- how controls share the scene's jelly seed (rendering through the scene's material state, or a test-only seeded constructor), since MaterialState::new draws a fresh seed from a global counter and aurora.frag uses it even at drift-hz 0;
- whether to bucket envelope redraws as drift-hz does (a measured answer, from a later quiet run piloted first);
- review media: real-time 25 fps clips per D, with S1 stills for appearance only.
Leave the chosen look and acceptable ongoing cost to the owner at the sheet review. No weather clock, cover sequence, transport, particles or time-of-day source (material-f41c54, material-54bcac keep theirs).

Where: src/render_helpers/material/optics/aurora.rs and mod.rs (apply_resolved), src/render_helpers/shaders/material/aurora.frag, niri-config/src/material/ (pipeline.rs lists no Aurora response), src/layout/tile.rs (optic_frame, tick_deadline, are_animations_ongoing); constraints in docs/specs/2026-09-29-sustained-optic-settling-design.md and docs/materials/render-pipeline.md.

Ideas it wakes: on the design result, update the brief and run `tasks note <id> --no-reviewed "<finding>"` on material-1c5a30, material-f3e4e4 and material-f41c54 (for f41c54, say whether the envelope could carry attenuation colour and which clock was chosen), before this result's branch merges.

## Notes

- 2026-10-17T02:15:39Z (materials-26.04): From brief review round 4 (on material-ffd61f's proposal): added scenes S10–S12 through amount 0 with continuity and exact-termination checks, and required seed-matched controls for byte comparisons; both carried into this body.
- 2026-10-18T23:01:49Z (materials-26.04): Scope pass 2026-10-18: material-f41c54 (ambient baseline) now depends on this design; added it to Ideas it wakes. Scope unchanged: no time-of-day source here.
- 2026-10-19T15:10:07Z (materials-26.04): Brief checks corrected (9f32a0c2, from owner review 2026-10-19): S10 and S12 end lit at drift-hz 4, so they are exempt from full quiescence; from the end on the envelope requests no frames while the drift keeps the static sun preset's deadlines and phase and holds at idle. S12, like S4, retriggers at D/2 and ends at 1.5 D; its endpoint, frame-request and settling checks sample from there.
