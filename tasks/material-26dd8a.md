---
id: material-26dd8a
title: "Ring of light focus response: embedded refracted filament with drift"
status: doing
priority: 2
size: m
owner: design/ring-light
created: 2026-09-05T10:16:06Z
updated: 2026-09-05T11:53:30Z
depends: []
parent: material-d1f471
tags: [material, focus-ring]
spec: docs/specs/2026-09-05-ring-light-focus-response-design.md
---

Winner of the focus ring light spike (docs/materials/2026-09-05-focus-ring-light-spike.md). Implement as a glass response, not a focus-ring option: an emissive filament inside the slab at a configurable inset and width, refracted per channel through a light-path index derived from the glass ior (the spike used 1 + (ior - 1) * 6 to make Prism's 1.02 visible; expose the multiplier as a glass parameter), attenuated by a fraction of the Beer-Lambert term so dense dark glass does not swallow it, with a slow travelling brightness (wave or caustic mode) and a breath on jelly activity. Needs a real active input to the material element beyond the is-active material swap, and a crossfade on focus change (see material-5a5fff). Config in the material response block: focus "ring-light" | "none", focus-inset, focus-width, focus-color, brightness mode, drift rate in Hz where 0 means static. Drift redraws must come from a timer at the chosen rate with time pinned when static or when animations are off, so the degrade path is a static refracted filament at zero per-frame cost. Measure against the DRM acceptance gates with drift running. Start from niri-experiments fixtures/focus-ring-light-probe.patch (probeRing, probeRefract, probeDepth) and the spike harness.

## Notes

- 2026-09-05T11:08:29Z (design/ring-light): design approved: fade the light only (swap interpolation stays material-5a5fff); one filament shared with the accent ring, accent tints it; gradient ring turned off explicitly by config; drift is a bucket-timer oscillator with ring-drift-hz, pinned at 0 / motion off / animations off; light-ior glass parameter default 6
- 2026-09-05T11:38:26Z (design/ring-light): spec review fixes: crossfaded accent presence (accent.w) with interrupted-fade start and fingerprint; band masked to the rendered bevel via slabSurface's inner face distance at the refracted position; explicit accent/focus/ring-pulse gates; ring-width 0 is a validation error
- 2026-09-05T11:53:30Z (design/ring-light): spec review round 2: accent RGB carried straight with a separate crossfaded presence (arrival holds the new color, expiry holds the last, live-to-live interpolates); bevel mask is chamfer > 0 ? smoothstep(0,1,di) : 0 at the displayed fragment, only the filament sampling is refracted
