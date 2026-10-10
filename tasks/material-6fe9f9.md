---
id: material-6fe9f9
title: Pixels-only mode for glass-noise-site-smoke.sh on the pixels lane
status: shelved
priority: 2
created: 2026-10-09T01:15:14Z
updated: 2026-10-10T13:32:12Z
depends: []
tags: [capture]
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-10-08-capture-host-conditions-design.md
---

Spec §3.1/§10: the fixture mixes pixel assertions with a recorded timing; a mode that drops the timing can preflight --lane pixels.

Scope finding (2026-10-10): docs/materials/scripts/glass-noise-site-smoke.sh records no Tracy timing (no trace_run or gpu_median_ns). Its only wall-clock dependence is ring_cell's sleep of one computed comet lap plus tail, before shot_twice. shot_twice asserts AE 0, so a comet still running on a contended desktop fails the run rather than passing a false result. A pixels mode therefore needs only a deterministic rest for the ring cells, then `capture_preflight pixels`. Options: a config without the beam (if one ring-light setting still leaves the rest glow the band ratio needs), or polling until two shots agree, with a bounded timeout. The site/omitted/baseline cells are already static. Check: the ring band level stays in 0.85..0.95, and a tooling test asserts the lane.

## Notes

- 2026-10-10T13:32:10Z (materials-26.04): shelved: A task needs to run this fixture again (spec 2026-10-08-capture-host-conditions §3.1: the split is taken when the fixture is next needed), or the pixels lane is wanted for it on a desktop in use
- 2026-10-10T13:32:10Z (materials-26.04): scope: shelved; body corrects the premise: no recorded timing, only the comet-lap wall wait before shot_twice; linked spec 2026-10-08-capture-host-conditions
