---
id: material-047f33
title: Pixels-only mode for glass-aurora-smoke.sh on the pixels lane
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

Scope finding (2026-10-10): in docs/materials/scripts/glass-aurora-smoke.sh only part of the pixel evidence is static. These assertions are static: zero_vs_plain_ae, pinned_ae (drift-hz 0, shots 2 s apart), face_rmse_over_one_code and lit_face_brighter. The bucket check (within_bucket_min_ae, across_bucket_ae) is not: it compares shots against wall-clock drift buckets at drift-hz 1. The spec puts that under "pixels sampled at wall-clock instants" in nested measurements, so a pixels mode must drop it, along with the redraw-rate and GPU-median blocks. Open choice for whoever takes this: leave the bucket check in the headless scope only, or move it to an in-process frozen-clock test, which would make it pixels-lane evidence. The lean is headless-only, unless the bucket logic changes. Use the SCOPE convention lean from material-c8857f.

## Notes

- 2026-10-10T13:32:10Z (materials-26.04): shelved: A task needs to run this fixture again (spec 2026-10-08-capture-host-conditions §3.1: the split is taken when the fixture is next needed), or the pixels lane is wanted for it on a desktop in use
- 2026-10-10T13:32:10Z (materials-26.04): scope: shelved; body separates the static aurora assertions from the wall-clock bucket check the pixels lane cannot carry; linked spec 2026-10-08-capture-host-conditions
