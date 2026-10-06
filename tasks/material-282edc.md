---
id: material-282edc
title: Verify the idle-budget DRM consumer and its sub-run finish boundary
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-10-06T19:18:11Z
updated: 2026-10-06T19:18:11Z
depends: []
parent: material-2834d7
tags: [capture]
source: "docs/notes/2026-10-06-capture-lifecycle-brief.md#external-idle-budget-consumer"
agent: codex
---

Question: Is the retained idle-budget DRM fixture still a supported consumer of glass-optic-smoke-lib.sh, and which normal stop path leaves a settled sub-run unfinished?
Where to start: docs/notes/2026-10-06-capture-lifecycle-brief.md; docs/specs/2026-09-27-idle-budget-fail-fast-design.md; docs/materials/2026-09-11-idle-budget-evidence.md; docs/materials/scripts/glass-optic-smoke-lib.sh::settle_before_launch/finish_sub_run/stop_nested; docs/materials/scripts/optic-settling-smoke.sh::start_drm/stop_drm; material-c44509. The consumer is documented on niri-experiments results/idle-budget and results/capture-protocol, outside this checkout.
Bound: Locate the retained experiments checkout and branch without switching its shared branch or live pointers. Read the actual consumer, pin its source revision and imported lib revision, and trace one start/settle/normal-stop lifecycle alongside refused and aborted paths. If it is unavailable, name the missing checkout/ref precisely. No capture, host takeover or production consumer patch. An isolated existing stub-based shell check may demonstrate the call boundary if available.
Expected result: Record whether the consumer is active, the exact insertion point and verification needed for a minimal finish call, or why the task should be shelved or dropped; distinguish normal completed sub-runs from refused or abandoned ones. Do not mark aborted sub-runs finished merely because cleanup ran. Record findings on this task and in the brief.
Ideas it wakes: On completion, run tasks note on material-a9a455 with the finding, in the same commit as this result.
