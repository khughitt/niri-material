---
id: material-26dd8a
title: "Ring of light focus response: embedded refracted filament with drift"
status: todo
priority: 2
size: m
created: 2026-09-05T10:16:06Z
updated: 2026-09-05T10:16:06Z
depends: []
parent: material-d1f471
tags: [material, focus-ring]
---

Winner of the focus ring light spike (docs/materials/2026-09-05-focus-ring-light-spike.md). Implement as a glass response, not a focus-ring option: an emissive filament inside the slab at a configurable inset and width, refracted per channel through a light-path index derived from the glass ior (the spike used 1 + (ior - 1) * 6 to make Prism's 1.02 visible; expose the multiplier as a glass parameter), attenuated by a fraction of the Beer-Lambert term so dense dark glass does not swallow it, with a slow travelling brightness (wave or caustic mode) and a breath on jelly activity. Needs a real active input to the material element beyond the is-active material swap, and a crossfade on focus change (see material-5a5fff). Config in the material response block: focus "ring-light" | "none", focus-inset, focus-width, focus-color, brightness mode, drift rate in Hz where 0 means static. Drift redraws must come from a timer at the chosen rate with time pinned when static or when animations are off, so the degrade path is a static refracted filament at zero per-frame cost. Measure against the DRM acceptance gates with drift running. Start from niri-experiments fixtures/focus-ring-light-probe.patch (probeRing, probeRefract, probeDepth) and the spike harness.
