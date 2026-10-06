---
id: material-6cca0a
title: "Design: per-workspace signal summaries in the event stream"
status: shelved
priority: 2
size: s
created: 2026-09-02T12:09:35Z
updated: 2026-10-06T20:15:24Z
depends: [material-a54d89]
parent: material-b5cbd6
tags: [signals, design, ipc]
---

Outcome: decide whether the IPC event stream should carry a per-workspace fold of window signals for bars and overviews, and its shape. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

## Notes

- 2026-09-29T22:31:41Z (materials-26.04): scope: briefed; existing window signals, workspace membership and event reduction support testing client-side folding before adding a workspace API; brief: docs/notes/2026-09-29-signal-model-extensions-brief.md
- 2026-10-03T00:43:31Z (material-c1330b): material-c1330b: existing IPC (WindowOpenedOrChanged/WindowSignalChanged/WindowClosed + reconnect snapshot) lets a client maintain a correct per-workspace max level through moves, TTL demotion, clear, close and reconnect; gaps are no refresh boundary (transient states), dragged windows unattributed, unknown cross-workspace atomicity. Recommend client-side summary; no compositor summary event until a consumer needs atomicity, drag attribution or a central fold (brief: Workspace replay).
- 2026-10-06T20:15:23Z (materials-26.04): shelved: A named bar or overview consumer requires refresh-atomic cross-workspace summaries, defined dragged-window attribution, a centrally defined fold beyond maximum level, or demonstrates unacceptable client folding cost; record the requirement and budget before unshelving.
- 2026-10-06T20:15:23Z (materials-26.04): scope: shelved; completed workspace replay 4a586bc8 supports client-side summaries; revisit a compositor event only when a named consumer needs atomicity, drag attribution, a shared richer fold or demonstrates excessive client cost; brief: docs/notes/2026-09-29-signal-model-extensions-brief.md
