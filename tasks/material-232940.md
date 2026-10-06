---
id: material-232940
title: Design capture fixture outcome finalization separately from hold release
status: todo
priority: 2
size: s
complexity: high
process: planned
created: 2026-10-06T19:18:11Z
updated: 2026-10-06T19:18:46Z
depends: [material-282edc]
parent: material-2834d7
tags: [capture]
source: "docs/notes/2026-10-06-capture-lifecycle-brief.md#fixture-outcome"
agent: codex
---

Why: material-e1ef98 wants artifact-only runs to retain their result. capture-meta currently stamps timing on first release or recovery; that time and release verdict do not establish the fixture result. glass-optic-smoke-lib.sh::summarize releases before PASS, cleanup releases again, and cleanup/checksum operations may fail after either call.
Where to start: docs/notes/2026-10-06-capture-lifecycle-brief.md; tools/capture-meta::cmd_release/release/finish_run/stamp_finish/restore_run/refresh_record_sum; tools/test_capture_meta.py::LifecycleTests, especially repeated-release and manifest checks; docs/materials/scripts/glass-optic-smoke-lib.sh::summarize/cleanup; ring-motion-clips.sh, focus-swap-clips.sh, drag-lag-clips.sh and optic-settling-smoke.sh exit traps; tools/test_glass_optic_smoke.py and test_optic_settling.py; docs/specs/2026-09-11-material-capture-protocol-design.md. External idle-budget traps are not available in this checkout; coordinate their inventory with the research task in the brief.
Bound: Write a reviewed design, then an implementation plan for review. Recommend the smallest numeric outcome record instead of adding an outcome taxonomy. Decide whether it means fixture work, post-restore cleanup or final shell exit; do not call an early zero release a fixture pass. Define first/final write semantics, optional late outcome after an early release, immutable repeated finalization, conflicting reports, shell-status validation, restoration errors, checksum ordering and killed-run unknowns. Retain the existing release return-code and ownership contract; do not infer SIGKILL/OOM status from a recovery timestamp.
Done: The recording boundary is unambiguous; planned offline checks distinguish pass, fixture failure, signal exit, disturbed release, post-release cleanup/checksum failure, repeated calls and recovery without an observed status. Existing timing and provenance remain truthful. No implementation, long capture or new status taxonomy in this design task.
Ideas it wakes: On completion, run tasks note on material-e1ef98 with the reviewed decisions, in the same commit as this result; update the brief.
