---
id: material-18c2a1
title: Design per-lane host conditions for capture preflight and settle evidence
status: doing
priority: 1
size: s
complexity: high
process: planned
owner: capture-host-conditions
created: 2026-10-06T19:18:11Z
updated: 2026-10-08T20:49:50Z
started: 2026-10-08T20:40:22Z
depends: []
parent: material-2834d7
tags: [capture]
source: "docs/notes/2026-10-06-capture-lifecycle-brief.md#renderer-aware-capture"
agent: codex
spec: docs/specs/2026-10-08-capture-host-conditions-design.md
---

Why: material-6bd4a3 records a desktop-up preflight pass, but later per-case settles remain sensitive to desktop activity. The only named software-rendered consumer, glass-view-tilt-smoke.sh (material-cd0e1d), exists only on the dropped material-77db8a branch and passed under the existing GPU gates on 2026-09-24, so material-925518 is shelved and no software lane is in scope. Existing deterministic pixel fixtures already avoid quiet-host measurements; removing GPU checks from timed or hardware evidence requires a distinct, verified contract.
Where to start: docs/notes/2026-10-06-capture-lifecycle-brief.md; docs/specs/2026-09-11-material-capture-protocol-design.md; docs/notes/2026-10-02-workstreams-brief.md; tools/capture-meta::Sample/sample_stream/judge_quiet/judge_settled/preflight/settle; tools/test_capture_meta.py; docs/materials/scripts/glass-optic-smoke-lib.sh::capture_preflight/await_gpu_rest/start_nested; src/tests/ring_pair.rs.
Bound: Write a reviewed design and then an implementation plan for review, covering deterministic pixels, nested hardware measurements and dedicated board power: which evidence class each route belongs to, and which host condition (desktop up on an empty workspace, desktop idle, TTY with the desktop stopped) its preflight and settles require. A software-rendered lane is out of scope until a consumer is demonstrated; record what such a consumer would have to prove (actual renderer evidence per participating compositor/client, including Weston) as an open question, not a design. For the retained routes, define fail-early mismatch handling, lock/hold ownership, preflight-versus-launch ordering and provenance. Keep existing hardware thresholds unchanged unless measured evidence supports a separate change. No production implementation, live desktop experiment or threshold relaxation in this task.
Done: Evidence claims and allowed host conditions are explicit for each route, NVIDIA sampling is not silently fabricated or bypassed, non-NVIDIA behavior has clear acceptance checks, and the plan includes offline fake-host checks plus a pilot before any measured lane. Keep hidden-window attribution under material-7afc31 rather than commissioning duplicate GPU attribution.
Ideas it wakes: On completion, run tasks note on material-6bd4a3 with the reviewed decisions (and on material-925518 only if the design identifies a software-rendered consumer), in the same commit as this result; update the brief.

## Notes

- 2026-10-06T21:17:04Z (materials-26.04): Narrowed in review: the only named llvmpipe consumer (glass-view-tilt-smoke.sh, material-cd0e1d) is on the dropped material-77db8a branch and passed under the GPU gates on 2026-09-24; software lane out of scope, material-925518 shelved. Stale 'record absent' claim removed (cd0e1d is present since 9ad1a716).
- 2026-10-08T20:40:22Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:b6f2497c-748f-44b4-8e1c-9de00d2b47e1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-08T20:40:35Z (capture-host-conditions): resumed
  provenance: {"harness_session":"claude-code:b6f2497c-748f-44b4-8e1c-9de00d2b47e1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-08T20:49:49Z (capture-host-conditions): spec drafted: docs/specs/2026-10-08-capture-host-conditions-design.md — four routes (frozen, nested pixels via new --lane pixels + begin, nested measurements, dedicated); pixels skip GPU sampling and partial-hold timers/services; measurement routes verify the nested renderer names the sampled GPU; desktop-idle for nested measurements decided by a two-run noise-layers-cost pilot against retained TTY medians; thresholds unchanged
