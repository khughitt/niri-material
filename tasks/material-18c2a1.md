---
id: material-18c2a1
title: Design renderer-aware capture evidence and preflight rules
status: todo
priority: 1
size: s
complexity: high
process: planned
created: 2026-10-06T19:18:11Z
updated: 2026-10-06T19:18:11Z
depends: []
parent: material-2834d7
tags: [capture]
source: "docs/notes/2026-10-06-capture-lifecycle-brief.md#renderer-aware-capture"
agent: codex
---

Why: material-6bd4a3 records a desktop-up preflight pass, but later per-case settles remain sensitive to desktop activity. material-925518 reports software-rendered fixtures blocked by NVIDIA telemetry. Existing deterministic pixel fixtures already avoid quiet-host measurements; removing GPU checks from timed or hardware evidence requires a distinct, verified contract.
Where to start: docs/notes/2026-10-06-capture-lifecycle-brief.md; docs/specs/2026-09-11-material-capture-protocol-design.md; docs/notes/2026-10-02-workstreams-brief.md; tools/capture-meta::Sample/sample_stream/judge_quiet/judge_settled/preflight/settle; tools/test_capture_meta.py; docs/materials/scripts/glass-optic-smoke-lib.sh::capture_preflight/await_gpu_rest/start_nested; src/tests/ring_pair.rs. The referenced glass-view-tilt-smoke.sh and material-cd0e1d record are not present in this fixed checkout; locate and verify the actual consumer before relying on its historical renderer claim.
Bound: Write a reviewed design and then an implementation plan for review, covering deterministic pixels, software-rendered timing, nested hardware measurements and dedicated board power. Decide whether a software lane is still needed after moving visual identity checks into frozen-time fixtures. For any retained software lane, require actual renderer evidence per participating compositor/client, including Weston; define fail-early mismatch handling, CPU/load/memory gates, applicable telemetry, lock/hold ownership, preflight-versus-launch ordering and provenance. Keep existing hardware thresholds unchanged unless measured evidence supports a separate change. No production implementation, live desktop experiment or threshold relaxation in this task.
Done: Evidence claims and allowed host conditions are explicit for each route, NVIDIA sampling is not silently fabricated or bypassed, non-NVIDIA and wrong-renderer behavior has clear acceptance checks, and the plan includes offline fake-host checks plus a pilot before any measured lane. Keep hidden-window attribution under material-7afc31 rather than commissioning duplicate GPU attribution.
Ideas it wakes: On completion, run tasks note on material-6bd4a3 and material-925518 with the reviewed decisions, in the same commit as this result; update the brief.
