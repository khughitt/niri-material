---
id: material-068639
title: colorSource familiar leaves every terminal on the same resting ring color
status: todo
priority: 2
size: s
complexity: low
process: direct
created: 2026-09-22T15:37:43Z
updated: 2026-09-22T15:37:52Z
depends: []
tags: [signals, bug, prism]
agent: claude-code/claude-opus-5
---

Owner report 2026-09-22: with the ring color source set to `familiar`, the ring is the same color in every terminal. Expected: each window's ring takes its familiar's hue.

Diagnosis (read 2026-09-22, no code changed). The compositor half is in place and the Prism half is in place; the bridge between them is not.

- Prism `integrations/niri/render.js` `responseBlock`: `colorSource: "familiar"` emits `accent "ring"` and sets `ring-color` to the manual `glass.ring.color`, which is the *resting* color. It deliberately does not read the palette file (`test/niri-apply.test.js`, "a familiar-driven ring never reads the palette file").
- niri consumes a per-window accent through the material signal system (`set/pulse/clear-window-signal`), which `material-a54d89` delivered. With `accent "ring"` on and no signal set, `mat_sig_accent.w` (presence) is 0, so the shader keeps `mat_sig_ring_color` — the one static resting color — for every window. That is exactly the reported symptom, and it is the designed behaviour of an unfed accent channel, not a rendering fault.
- Nothing pushes those signals. `familiar`'s `bin/familiar-niri` and `integrations/niri/window.js` only maintain the session-to-window map in `niri-windows.json`; there is no `set-window-signal` call anywhere in familiar. That is material-930c55, still an idea.

So the fix for "expected: ring color ~ familiar hue" is the bridge (material-930c55), not a change in the ring. What is left for *this* task is the honesty gap in between: Prism offers `familiar` as a selectable color source (and `noctalia` is the def's default, so `familiar` is a deliberate pick) with no indication that nothing drives it yet. Decide and land one of: mark the option as unavailable until the bridge exists, or keep it and say so in the def description and the panel. Pairs with prism-b4d118, which is settling the same Ring rows.

Close this when a user selecting `familiar` either sees per-window hues or is told why not.

## Notes

- 2026-09-22T15:37:52Z (materials-26.04): Deliberately not depending on material-930c55: the bridge is one of the two acceptable outcomes, but marking the option unavailable closes this without it.
