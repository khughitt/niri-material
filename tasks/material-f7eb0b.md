---
id: material-f7eb0b
title: Verify optic settling across real TTY resume and unlock
status: todo
priority: 1
size: m
complexity: mid
process: planned
created: 2026-10-02T08:06:31Z
updated: 2026-10-02T21:02:02Z
depends: []
parent: material-f86183
tags: [performance]
source: "docs/materials/2026-09-30-optic-settling-evidence.md#unverified-session"
agent: codex
spec: docs/specs/2026-10-02-real-tty-settling-lane-design.md
plan: docs/plans/2026-10-02-real-tty-settling-lane.md
---

Remaining acceptance cases tty-resume and unlock from material-2ee11e. Task 3 real-handler tests cover unlock wiring, but a real session capture is still unverified. Use the accepted spec §§6–8 and explicitly identified worktree binary, add the missing dedicated real-TTY lane, then run its end-to-end pilot before matrix on an idle host. Distinguish backend TTY resume from IPC power-on; input/session activation must resume from held phase without catch-up, and later idle must stop optic deadlines. Retain provenance, trace liveness, owned-process cleanup, run notes and evidence links. No host launcher changes or installation; mark unavailable environments unverified.

## Notes

- 2026-10-02T20:07:56Z (material-3acc86): process -> planned (2026-10-02): the dedicated real-TTY lane needs a design for DRM takeover, VT switching (privileged) and an authenticating unlock; material-3acc86's screencast also needs this lane (nested winit has no GBM device). Design both in one spec.
- 2026-10-02T20:25:48Z (material-3acc86): spec drafted: docs/specs/2026-10-02-real-tty-settling-lane-design.md (owner decisions: NOPASSWD chvt, PAM-free lock client, DP-1 takeover, gst pipewiresrc consumer, DDC/CI dimming). Dimming helper filed as ops-a1715a; it never gates a capture, so no dependency.
- 2026-10-02T20:35:45Z (material-3acc86): review: spec round 1 — verdict: revise; findings: Important 2, Minor 1; reviewer: human
- 2026-10-02T20:43:38Z (material-3acc86): review: spec round 2 — verdict: revise; findings: Important 2; reviewer: human
- 2026-10-02T20:53:31Z (material-3acc86): review: spec round 3 — verdict: accept; findings: none; reviewer: human
