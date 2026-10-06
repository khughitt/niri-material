---
id: material-ccda38
title: Design one finite focus-loss cue from the dynamics brief
status: todo
priority: 2
size: s
complexity: high
process: planned
needs: [owner]
created: 2026-10-06T20:28:21Z
updated: 2026-10-06T21:03:54Z
depends: []
parent: material-53f873
tags: [dynamics, rendering]
source: docs/notes/2026-09-29-material-dynamics-brief.md
agent: codex
---

Design one finite focus-loss cue from docs/notes/2026-09-29-material-dynamics-brief.md. Start with the owner's concrete ring-drain example under material-d873bf, comparing the existing moving-beam cut plus resting-light crossfade against a short continuation/drain of existing beam light. Keeping current behavior is a valid outcome if the extra cue does not improve focus readability. General rotation/glints on open, move and resize remain in the original idea, outside this first design.

Done: a written design and implementation plan reviewed by the owner before renderer code. Settle exactly what light remains after loss, whether any cue starts when the gain beam is already finished, trigger/pacing and a finite end condition, repeated gain/loss and rapid refocus, material-name swaps and seeds, config reload, resize/jelly geometry, hidden/covered panes and other outputs, reduced/off motion and animations off. Preserve the accepted hard material cuts (material-8e3b73, 7f5ee9e2), signal identity/attention and client pixels. Reuse existing beam state and animation mechanisms when they suffice; a general profile algebra or state machine is not part of this task.

Where to look: src/layout/tile.rs::update_render_elements, focus_value, material_dynamics, advance_animations and are_animations_ongoing; src/render_helpers/material/ring.rs; src/render_helpers/signal.rs; src/render_helpers/material/mod.rs::apply_resolved; docs/specs/2026-09-19-ring-beam-design.md section 2; docs/materials/material-config.md; docs/materials/scripts/ring-motion-clips.sh; src/tests/ring_look.rs; docs/notes/2026-10-06-material-dynamics-findings.md.

Verification contract: define frozen-clock gain/loss/refocus sequences and neutral, policy-off and exact-rest checks, stable fingerprints/no deadlines after the finite run, and text/opaque-client identity. Propose a matched visual comparison with the current crossfade and owner acceptance before choosing defaults. Any later measured capture gets a separate preparation step, quiet need and smallest pilot through verdict; this design phase launches no compositor or live-host action.

Coordinate with material-77db8a's pending focus-gain view-tilt review; its branch's appearance is not accepted or on current main, and no cross-checkout renderer inspection is implied. Design may proceed from the current ring baseline; settle combined behavior only once the relevant branch is current and its review result is available. Related material-6d4de5 remains under its own existing scoping verdict and is not reprocessed.

On the design result, update the dynamics brief and run tasks note on material-d873bf with the finding in the same commit, so the idea can be reconsidered. Owner judgement is required for the design and visual contract; no new measurement is claimed.

## Notes

- 2026-10-06T21:03:54Z (materials-26.04): 2026-10-06: material-77db8a's optical-only focus-gain tilt was rejected and dropped unmerged; there is no combined behaviour with it to settle. A rigid tilt of window and glass is scoped as material-89fb6b then material-abc08c (docs/notes/2026-10-06-transient-depth-brief.md); coordinate with that design instead.
