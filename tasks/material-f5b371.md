---
id: material-f5b371
title: "Deterministic test: optic settling when one of two outputs is removed"
status: todo
priority: 1
size: s
complexity: mid
process: direct
created: 2026-10-07T08:12:15Z
updated: 2026-10-07T08:12:15Z
depends: []
parent: material-f86183
tags: [performance, testing]
source: material-1af3c6
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-09-29-sustained-optic-settling-design.md
---

Replaces material-1af3c6 (dropped 2026-10-07: no two-output host). In-process, in src/tests/attention_idle.rs style with real calloop timers: a Fixture with two headless outputs (as src/tests/remove_output.rs builds) and an Aurora tile on output 1; remove output 2 (a) while input is active and (b) after the idle edge. Assert, per spec §6/§7 and the Outputs/DPMS row of its capture matrix: active, the shared timeline keeps its configured cadence on the lit output (optic timer armed, redraws continue); idle, the timeline stays held with no optic deadlines and no redraw is queued for the removed output; the lit output's next frame samples the held logical instant (no phase jump). Also cover re-adding the output while idle (still held, no deadline) and while active. Update docs/materials/2026-09-30-optic-settling-evidence.md: the two-output real capture is dropped as unverified on hardware and this test is its deterministic coverage. Then confirm material-f86183's acceptance against its landed lanes and close the goal.
