---
id: material-c8857f
title: Pixels-only mode for glass-iridescence-smoke.sh on the pixels lane
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

Scope finding (2026-10-10): docs/materials/scripts/glass-iridescence-smoke.sh splits cleanly. Its pixel part (zero_vs_plain_ae, chamfer_ab_rmse over one code and over the face) comes from static shots after a fixed settle; the Tracy cost block (tools_ready, trace_run, gpu_median_ns, median3) follows after the asserts. Lean: copy glass-render-order-smoke.sh's SCOPE=pixels|all convention, where SCOPE=pixels runs `capture_preflight pixels` and stops before tools_ready. The lib's no_timing_on_pixels guard (glass-optic-smoke-lib.sh) refuses any timing helper left behind. Check: a tools/test_glass_optic_smoke.py case asserts the pixels scope preflights pixels and never reaches trace_run.

## Notes

- 2026-10-10T13:32:10Z (materials-26.04): shelved: A task needs to run this fixture again (spec 2026-10-08-capture-host-conditions §3.1: the split is taken when the fixture is next needed), or the pixels lane is wanted for it on a desktop in use
- 2026-10-10T13:32:10Z (materials-26.04): scope: shelved; body records the clean pixel/cost split and the SCOPE=pixels convention lean; linked spec 2026-10-08-capture-host-conditions
