---
id: material-79d1de
title: "Source: OSC 9;4 progress and OSC 9/99 notifications"
status: idea
priority: 2
size: s
reviewed: true
created: 2026-09-02T12:09:35Z
updated: 2026-10-12T18:21:13Z
depends: [material-a54d89, material-07bac9, material-9fb9ec]
parent: material-9b8bf9
tags: [signals, sources]
---

Outcome: map terminal progress reports (OSC 9;4) to the progress channel once it exists, and desktop-notification escapes (OSC 9, OSC 99) to Notice level with a ping. Depends on the shell-integration watcher for the transport. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

## Notes

- 2026-09-29T22:07:57Z (materials-26.04): scope: briefed; notification state fits existing levels/impulses, but numeric job progress is absent; terminal transport research material-07bac9 precedes implementation; brief: docs/notes/2026-09-29-signal-sources-brief.md
- 2026-10-08T20:35:51Z (materials-26.04): Added depends material-07bac9: the signal-sources brief makes that probe the transport answer this idea waits on (found beside material-d277d0 in the 2026-10-08 curation sweep)
- 2026-10-10T14:05:12Z (materials-26.04): scope: briefed; no change: readmitted only by the material-07bac9 depends note, which the brief already records; 07bac9 (todo) is still the unanswered transport question, and numeric progress still waits on material-5d854f; brief: docs/notes/2026-09-29-signal-sources-brief.md
- 2026-10-11T09:27:49Z (material-07bac9): transport finding: material-07bac9 observed kitty OSC 9/99 in a separate notification-filter callback and OSC 9;4=42 through a version-specific tab-bar progress sample; neither arrives through on_cmd_startstop. Reuse the command-source identity design, not its event callback. Ghostty external delivery/identity remains unknown. Numeric progress is absent from current IPC and still requires representation/expiry/arbitration design with material-5d854f; completion is already a Done/Error impulse. Evidence: docs/notes/2026-09-29-signal-sources-brief.md and 07bac9 attachment.
- 2026-10-11T09:56:45Z (materials-26.04): scope: briefed; retained notification/progress outcome and added identity/lifecycle design dependency material-9fb9ec; Ghostty callback coverage is researched by material-53da57; separate kitty notification callbacks and numeric-progress design alongside existing material-5d854f remain required; no duplicate progress task; brief: docs/notes/2026-09-29-signal-sources-brief.md
