---
id: material-5d854f
title: "Glass response: progress fill on the ring"
status: idea
priority: 2
size: s
reviewed: true
reviewed_at: 2026-10-19T15:37:30Z
created: 2026-09-02T12:09:35Z
updated: 2026-10-19T15:57:47Z
depends: [material-a54d89, material-9faf34]
parent: material-0a4093
tags: [signals, rendering, design]
---

Outcome: the ring fills proportionally to a progress channel. Requires a continuous progress value in the signal model, which the v1 slice excludes; design that extension first (a per-slot progress: Option<f32> is the likely shape). Acceptance: design note, then response name in the glass vocabulary. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

## Notes

- 2026-09-29T21:32:24Z (materials-26.04): scope: briefed; confirmed job progress is absent from slots and IPC; source folding/expiry and current ring geometry require design before rendering; brief: docs/notes/2026-09-29-glass-signal-responses-brief.md
- 2026-10-19T15:37:30Z (materials-26.04): scope: briefed; job progress is still absent from slots and IPC; kitty delivers OSC 9;4 (material-07bac9), so a producer exists; filed material-9faf34 (P2 / m / high / planned, owner) to design the channel and ring fill after material-9fb9ec settles source ownership; this idea depends on it; brief: docs/notes/2026-09-29-glass-signal-responses-brief.md
- 2026-10-19T15:57:47Z (materials-26.04): Correction to the 2026-10-19 scope note: kitty progress delivery was demonstrated in a bounded probe only (material-07bac9 read version-specific kitty internals via on_tab_bar_dirty, no stable progress callback); a production adapter is still owed. See the transport limitations in docs/notes/2026-09-29-signal-sources-brief.md#coverage.
