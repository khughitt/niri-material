---
id: material-79d1de
title: "Source: OSC 9;4 progress and OSC 9/99 notifications"
status: idea
priority: 2
size: s
created: 2026-09-02T12:09:35Z
updated: 2026-10-10T14:05:12Z
depends: [material-a54d89, material-07bac9]
parent: material-9b8bf9
tags: [signals, sources]
---

Outcome: map terminal progress reports (OSC 9;4) to the progress channel once it exists, and desktop-notification escapes (OSC 9, OSC 99) to Notice level with a ping. Depends on the shell-integration watcher for the transport. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

## Notes

- 2026-09-29T22:07:57Z (materials-26.04): scope: briefed; notification state fits existing levels/impulses, but numeric job progress is absent; terminal transport research material-07bac9 precedes implementation; brief: docs/notes/2026-09-29-signal-sources-brief.md
- 2026-10-08T20:35:51Z (materials-26.04): Added depends material-07bac9: the signal-sources brief makes that probe the transport answer this idea waits on (found beside material-d277d0 in the 2026-10-08 curation sweep)
- 2026-10-10T14:05:12Z (materials-26.04): scope: briefed; no change: readmitted only by the material-07bac9 depends note, which the brief already records; 07bac9 (todo) is still the unanswered transport question, and numeric progress still waits on material-5d854f; brief: docs/notes/2026-09-29-signal-sources-brief.md
