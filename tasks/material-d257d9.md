---
id: material-d257d9
title: Design the opt-in window-content optical stage from the glass optics brief
status: todo
priority: 2
size: m
complexity: high
process: planned
created: 2026-10-06T18:28:17Z
updated: 2026-10-06T20:16:40Z
depends: []
parent: material-6062fd
tags: [rendering, design]
source: docs/notes/2026-10-06-glass-optics-brief.md
agent: codex
---

Design the opt-in window-content optical stage from docs/notes/2026-10-06-glass-optics-brief.md. The current film site does not touch opaque client content. Frame G2 content depth as the starting hypothesis against G1 shared film/tone and deferring content treatment; G3 halation and G4 sidechain remain alternatives, not bundled requirements.

Done: a written design and implementation plan reviewed by the owner before code. Settle opt-in and zero/disabled behavior, the window texture's sampling and alpha/compositing order, which content can refract or attenuate, readability at normal terminal text sizes, resize/open/close behavior, and popup/screencast coverage. Coordinate the shared content boundary with material-987655 while retaining its own trigger and parent. Preserve default opaque pixels byte-for-byte and apply window opacity exactly once. Name the height-field information needed from material-be611b; its accepted merge is required for a depth implementation, not for preparing the design. Define frozen-clock neutral/opaque/translucent checks, owner-reviewed text comparisons and a later bounded cost check for loss of the opaque early return. No renderer code or live host changes in the design phase.

On the design result, record the recommendation in the brief and run tasks note on material-7f5751 and material-987655 with the finding, in the same commit, so both ideas can be reconsidered.

## Notes

- 2026-10-06T20:16:40Z (materials-26.04): Scope handoff for material-987655: reuse this design; retain Quiet versus unfocused as a distinct trigger decision and compare client/popup/capture coverage and text readability. The existing wake-up note requirement remains; record the desaturation finding in docs/notes/2026-09-29-glass-signal-responses-brief.md alongside the optics brief, in the design-result commit. No new renderer or capture work is added to the design phase.
