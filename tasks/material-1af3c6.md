---
id: material-1af3c6
title: Verify optic settling when one of two outputs is removed
status: todo
priority: 1
size: m
complexity: mid
process: direct
needs: [quiet]
created: 2026-10-02T08:06:31Z
updated: 2026-10-04T03:28:50Z
depends: []
parent: material-f86183
tags: [performance]
source: "docs/materials/2026-09-30-optic-settling-evidence.md#unverified-output-removal"
agent: codex
spec: docs/specs/2026-09-29-sustained-optic-settling-design.md
---

Remaining output-removal acceptance from material-2ee11e. Use a real two-output lane with one output still lit; remove or disable the other while active and while input idle. The shared timeline must continue configured active cadence on the lit output or remain held with no optic deadlines according to global input state. Add the missing bounded capture lane, run its end-to-end pilot before matrix, and publish topology, provenance, transition/draw counts, hashes, cleanup and run notes. Use an explicitly identified worktree binary and preserve unavailable hardware cases as unverified; no installed-compositor changes.

## Notes

- 2026-10-04T03:28:50Z (materials-26.04): 2026-10-03 quiet TTY session: not runnable here, only one output is connected on titan (card1-DP-1; every other connector disconnected). Needs a second monitor attached (or a host with two) before its pilot.
