---
id: material-bae9c9
title: Consistent performance-capture protocol with provenance and environment metadata
status: doing
priority: 2
size: m
owner: material-bae9c9
created: 2026-09-11T23:34:15Z
updated: 2026-09-12T05:19:52Z
started: 2026-09-12T00:33:57Z
depends: []
parent: material-5d6b2c
tags: [quick-add, performance, testing]
source: "mindful:thought:a476e6bcd1fd4297b70824758235d821"
spec: docs/specs/2026-09-11-material-capture-protocol-design.md
plan: docs/plans/2026-09-11-material-capture-protocol.md
---

Define one recipe for measuring the cost of each rendering element (pass, parameter, preset) so results are comparable across runs. Every capture stores the same metadata: provenance (commit, binary hash, preset/config, driver script) and environment (GPU, driver, clocks/power state, resolution, output count, background load). Requirements: runs need a solo-machine window with no other heavy work, and a CPU/GPU/memory baseline check before and between sub-runs to confirm the machine is idle. material-265eb0's hardware note already asks for an isolated workload; this task supplies the method it should use.

Source: mindful:thought:a476e6bcd1fd4297b70824758235d821

## Notes

- 2026-09-12T00:58:10Z (material-bae9c9): parked (waiting on user, review): User reviews docs/specs/2026-09-11-material-capture-protocol-design.md (commit 77f4a4fd); on approval, writing-plans for tools/capture-meta and the two adoptions
- 2026-09-12T01:12:46Z (material-bae9c9): Spec revised at 8d9d65db after review: per-sub-run inputs via settle --input, identity per idle-budget mode, atomic owner-checked lock, memory floor/tolerance, analyzer migration off hardware.json, finish hashes every file.
- 2026-09-12T01:53:17Z (material-bae9c9): Second review round applied at b88990a4: preflight after lib init, settle in start_nested, per-mode identity inputs.
- 2026-09-12T02:11:54Z (material-bae9c9): Third review round at 41285cff: preflight explicit per entry script (lib exposes capture_preflight, never calls it); start_nested <niri> <config> [name]; power lane settles with observation name + case config.
- 2026-09-12T02:32:13Z (material-bae9c9): Fourth review round at 5be9d9e8: scene_evidence migrates to sub_runs[] (exactly one entry, settled, hash match). Spec approved; writing plan next.
- 2026-09-12T02:43:17Z (material-bae9c9): parked (waiting on user, review): User reviews docs/plans/2026-09-11-material-capture-protocol.md; on approval choose subagent-driven or inline execution starting at material-55ecdf
- 2026-09-12T03:04:29Z (material-bae9c9): Plan review round 1 applied at 99f09dc9: guarded lock + link publish, GPU evidence validation, CAPTURE_META_PROC for e2e, jelly-motion adoption, untracked provenance, tt/just test commands.
- 2026-09-12T03:19:28Z (material-bae9c9): Plan review round 2 applied at db12626b: manifests regenerated via .tmp+mv, aurora-iridescence-hardware.sh classified as pinned/frozen, identity tests keep run dirs outside the source checkout.
- 2026-09-12T04:47:03Z (material-bae9c9): Thresholds in DEFAULT_THRESHOLDS are from this host's idle RTX 3070; material-265eb0's first dedicated run should revisit them and update the spec.
- 2026-09-12T04:53:14Z (material-bae9c9): Manual headless preflight on this host exited 2: nvidia-smi static GPU query returned no usable output; reasons/client names were unavailable, release exited 0, and the disposable run was removed.
- 2026-09-12T04:53:15Z (material-bae9c9): Thresholds in DEFAULT_THRESHOLDS are from this host's idle RTX 3070; material-265eb0's first dedicated run should revisit them and update the spec.
- 2026-09-12T04:53:53Z (material-bae9c9): capture-meta (preflight/identity/settle/release/show) with tests; optic smoke lib and idle-budget fixture adopt it; analyzer reads capture.json
- 2026-09-12T04:57:03Z (material-bae9c9): Controller closeout correction: NVML currently fails with driver/library mismatch (library 615.71), so manual busy-host refusal acceptance remains open in parked child material-7f7aa3; host repair is outside this task.
- 2026-09-12T04:58:41Z (material-bae9c9): Read-only diagnosis confirmed loaded NVIDIA kernel module 610.57.04 versus NVML library 615.71; material-7f7aa3 waits on that environment repair.
- 2026-09-12T05:19:52Z (material-bae9c9): Implementation and clean scoped re-review complete: native cff6a865 with companion 9d7d43f; manual busy-host acceptance remains parked in material-7f7aa3.
