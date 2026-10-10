---
id: material-0b4085
title: Retire glass-render-order-smoke.sh's pixel-only capture-meta waiver for the pixels lane
status: shelved
priority: 2
created: 2026-10-09T01:15:14Z
updated: 2026-10-10T13:32:12Z
depends: []
tags: [capture]
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-10-08-capture-host-conditions-design.md
---

SCOPE=pixels runs under a stubbed CAPTURE_META recorded as gpu-quietness=pixel-only-waiver. Spec §3.1/§10: with --lane pixels the scope can keep a real capture record; check that pixel_matrix and within_matrix under SCOPE=pixels record no wall-clock values first.

Scope finding (2026-10-10), docs/materials/scripts/glass-render-order-smoke.sh: the precondition holds for pixel_matrix (behind) but not for within_matrix. within_matrix calls within_aurora_motion, which writes `date --iso-8601=seconds` stamps into metrics.txt and takes the aurora-motion shots at drift-hz 1 after a set-column-width, so they are wall-clock instants. The waiver path itself sits at lines 46-47 (IDENTITY_EXTRA), and the script still calls `capture_preflight headless` unconditionally, after select_binaries. Retiring it: choose the lane from SCOPE (pixels → `capture_preflight pixels`), drop the CAPTURE_META waiver config, and keep within_aurora_motion out of the pixels scope (cost/all only) or freeze its drift. Keep validate_scope before preflight: test_cost_scope_rejects_capture_override_before_preflight asserts that order.

## Notes

- 2026-10-10T13:32:10Z (materials-26.04): shelved: A task needs to run this fixture again (spec 2026-10-08-capture-host-conditions §3.1: the split is taken when the fixture is next needed), or the pixels lane is wanted for it on a desktop in use
- 2026-10-10T13:32:10Z (materials-26.04): scope: shelved; precondition checked: pixel_matrix is wall-clock free, but within_matrix's within_aurora_motion is not; body names the waiver lines; linked spec 2026-10-08-capture-host-conditions
