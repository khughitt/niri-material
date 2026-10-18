---
id: material-f41c54
title: "Source: ambient baseline from time of day or scheme tone"
status: idea
priority: 2
size: s
reviewed: true
reviewed_at: 2026-10-18T23:01:23Z
created: 2026-09-02T12:09:35Z
updated: 2026-10-18T23:01:23Z
depends: [material-a54d89, material-415d0c]
parent: material-9b8bf9
tags: [signals, sources, prism]
---

Outcome: decide whether an ambient global baseline (attenuation color temperature by time of day or Noctalia scheme tone) is a prism parameter change or a signal, and implement the chosen path. Likely prism-owned. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

Established 2026-10-18: the evidence now backs the Prism lean. Signal slots carry per-window facts written by sources (signals design sections 1 and 2); an ambient baseline is one global appearance, so it is an in-place definition change (a config reload or a Prism apply), not a signal. Such a change cuts today (`apply_resolved`). material-415d0c designs the envelope that eases an in-place definition change, scoped to Aurora amount and colours, and leaves the time-of-day source here. The dynamics brief (docs/notes/2026-09-29-material-dynamics-brief.md) finds a static warm Aurora preset is config-only now and leaves the clock question open (activity, real time or an event, and what input idle freezes); the weather cycle material-f3e4e4 shares that question. prism-9298b9 (shelved) would activate Prism contexts from Noctalia's theme-mode and colours hooks and leaves time of day to this idea.

Remaining: whether the envelope extends to attenuation colour, which clock drives a time-of-day change, and which Prism transport applies it (a state profile or a timer).

Waiting on: material-415d0c's design result. Brief: docs/notes/2026-09-29-signal-sources-brief.md.

## Notes

- 2026-09-05T18:51:03Z (materials-26.04): Related 2026-09-05: prism-9298b9 (state profiles idea under prism-2f0b4b) covers hook-driven theme-mode and power-profile contexts; the time-of-day baseline decision stays here.
- 2026-09-29T22:07:57Z (materials-26.04): scope: briefed; ambient baseline remains a global Prism parameter versus per-window signal decision; no ambient transport or visual contract is established; brief: docs/notes/2026-09-29-signal-sources-brief.md
- 2026-10-18T23:01:23Z (materials-26.04): scope: briefed; recorded that the ambient baseline is an in-place definition change (Prism-owned), not a per-window signal; made this idea depend on the Aurora envelope design material-415d0c, whose result decides easing; attenuation colour, the clock (shared with material-f3e4e4) and the Prism transport remain; brief: docs/notes/2026-09-29-signal-sources-brief.md
