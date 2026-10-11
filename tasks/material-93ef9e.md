---
id: material-93ef9e
title: Validate GPU clock sampling before setup and reject unsupported capture callers
status: todo
priority: 3
size: xs
complexity: low
process: direct
created: 2026-10-11T11:07:17Z
updated: 2026-10-11T11:07:17Z
depends: [material-f386ae]
parent: material-2834d7
tags: [capture, tooling]
source: material-f386ae
agent: codex
---

Why: implementation review of material-f386ae (preparation commit bfd8f357) left two minor findings concerning CAPTURE_GPU_CLOCKS. Keep them together because they share the same flag and error path.

Scope:
- In docs/materials/scripts/glass-optic-smoke-lib.sh, validate CAPTURE_GPU_CLOCKS before allocating OUT or the runtime directory, so invalid values fail without leaving directories behind.
- In docs/materials/scripts/optic-settling-smoke.sh, reject CAPTURE_GPU_CLOCKS=1 before setup/identity or host actions: this caller launches Tracy independently and does not use capture_bg. Its capture record must not claim 100 ms sampling when no sampler ran. Keep the default unsampled route and supported shared capture_bg callers working. Do not add sampling support to this fixture.

Done when: offline behavioral tests prove invalid values leave no runtime or artifact directories, the unsupported caller refuses the sampling flag without starting capture/host work or writing a sampling identity claim, and supported sampled/unsampled captures retain their behavior. Use tools/test_gpu_clocks.py and the existing optic-settling lifecycle fixtures; focused verification via just --set one_cmd "env NIRI_TOOLING_FAST=0 python3 -m unittest" test-one tools.test_gpu_clocks tools.test_optic_settling, then just --set fast_cmd "env NIRI_TOOLING_FAST=0 python3 -m tools.tooling_tests --full" test-fast. No live desktop or quiet host required.

Dependency is the originating branch landing; this follow-up does not replace its outstanding hardware pilot/full/control or attribution verdict.
