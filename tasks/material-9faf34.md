---
id: material-9faf34
title: Design a job-progress signal channel and ring fill from the glass-response brief
status: todo
priority: 2
size: m
complexity: high
process: planned
needs: [owner]
created: 2026-10-19T15:37:18Z
updated: 2026-10-19T15:37:18Z
depends: [material-9fb9ec]
parent: material-0a4093
tags: [signals, design, rendering]
source: docs/notes/2026-09-29-glass-signal-responses-brief.md
agent: claude-code
---

Why: two ideas wait on a numeric job-progress channel that no task owns. The ring fill (material-5d854f) needs a progress value to draw, and the OSC 9;4 source (material-79d1de) needs somewhere to put it. Both docs/notes/2026-09-29-glass-signal-responses-brief.md and docs/notes/2026-09-29-signal-sources-brief.md leave representation and arbitration unresolved. Established: kitty 0.49.2 delivered OSC 9;4 value 42 in the material-07bac9 probe, through kitty internals (on_tab_bar_dirty) rather than a stable callback; Ghostty documents OSC 9;4, with delivery pending material-53da57. Slot, SetSlot and Folded (src/window/signal.rs) and signal requests and events (niri-ipc/src/lib.rs) carry no job progress; the solver's impulse progress is animation age.

Done: a written design and implementation plan, reviewed by the owner before code. It must settle:
- representation: a per-slot value such as Option<f32>, plus indeterminate (OSC 9;4 state 3), error and paused states;
- arbitration across slots and sources on one window, reusing the source naming and ownership chosen by material-9fb9ec;
- validation, expiry, clearing and completion (does reaching 1 clear, demote or pulse Done?), and reconciliation after a writer dies;
- IPC requests and events, including event volume for frequent updates and whether progress-only changes emit WindowSignalChanged;
- the ring fill: geometry on the current arc-length ring beam (src/render_helpers/material/ring.rs, docs/specs/2026-09-19-ring-beam-design.md), its interaction with the focus light and signal accent, the response name in the glass vocabulary, and reduced/off motion;
- finite redraws: a changed value redraws once and an unchanged one never does, with no deadlines at rest.

Verification contract: unit tests for fold, arbitration and expiry; an IPC round trip; frozen headless renders of 0, partial, indeterminate and complete fills with byte identity when no progress is set; an owner-reviewed sheet before defaults. No live-host capture in the design phase.

Where to look: src/window/signal.rs, niri-ipc/src/lib.rs, src/ipc/server.rs (ipc_refresh_windows), src/render_helpers/signal.rs, src/render_helpers/material/ring.rs, docs/materials/2026-09-02-material-signals-design.md section 11, the two briefs above.

Ideas it wakes: on the design result, update the glass-response brief and run `tasks note <id> --no-reviewed "<finding>"` on material-5d854f and material-79d1de, before this result's branch merges.
