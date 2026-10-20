---
id: material-86c841
title: Attribute the recurrent GL_INVALID_VALUE signature to a renderer call
status: todo
priority: 2
size: s
complexity: high
process: direct
needs: [nested]
created: 2026-10-06T20:44:04Z
updated: 2026-10-20T21:45:28Z
depends: []
parent: material-7ff4bc
tags: [harness, rendering]
source: docs/notes/2026-10-06-render-anomalies-brief.md
agent: codex
---

Question: Which GL call and input dimensions/state produce the recurring 'Size and/or offset out of range' signature, and is it the same path as the 2026-09-05 report?

Where to start: docs/notes/2026-10-06-render-anomalies-brief.md; the task's original journal interval; the 2026-10-06 compositor journal (startup reports 310b4e30, errors at 15:53:19, 17:48:47, 18:24:55 and 19:11:45 UTC); Cargo.toml/Cargo.lock's pinned Smithay GLES renderer; src/render_helpers/offscreen.rs, effect_buffer.rs, blur.rs, framebuffer_effect.rs and material/mod.rs; src/tests/fixture.rs and existing surfaceless renderer helpers.

Bound: Preserve timestamp/PID/build/error metadata from existing journals, establish the actual binary/source identity available, and trace candidate calls at that pinned source and current checkout. Attempt at most one smallest in-process or nested headless reproduction selected from this evidence. If a candidate needs diagnostics, add only temporary or narrowly scoped call/argument attribution in the isolated task worktree; do not fork dependencies, introduce a logging framework, suppress the callback, or apply speculative size clamps. No live desktop replay, shared launcher/config changes, service restart, unattended long sweep or quiet performance run. IPC failed from this scoping shell because its socket path was unavailable; use explicit proven endpoints rather than assume it identifies the running binary.

Expected result: Record a minimal reproducer and the GL caller plus invalid arguments/state when established, or a ranked candidate list with what prevented attribution and the next falsifiable check. Include renderer/driver/build provenance and distinguish current-source facts from the installed 310b4e30 label. The old source tree at that label lacks the new grain pass, so do not attribute the installed symptom to that pass without additional binary provenance. Run a focused check through just test-one for any diagnostic fixture, then just test-fast before its code commit. Update this task and the brief; file a fix only when the cause and check are established.

Ideas it wakes: On completion, run tasks note --no-reviewed on material-c8732f with the attribution result before this result's branch merges.

Scope recheck (2026-10-20, 52c14d4d): The caller remains unknown. Upstream search on 2026-10-20 covered open/closed niri and Smithay issues/PRs for GL_INVALID_VALUE and the exact message, Smithay master update_memory, branch inventory and relevant development/resize branch material, and CHANGELOG.md. Smithay PR #1933 matches the exact message but is closed unmerged; maintainers ask for caller-level attribution rather than a GLES-only clamp. niri #4188 is another glTexSubImage2D bounds report, not proof of our cause. Smithay #2231 is open multigpu copy work, also unproven here. No verified fixing release/commit was established: do not pin or carry a workaround yet. Reuse material-86c841 to compare these candidates against pinned Smithay ff5fa7df and actual binary provenance before one minimal reproduction. Upstream links: https://github.com/Smithay/smithay/pull/1933 ; https://github.com/niri-wm/niri/issues/4188 ; https://github.com/Smithay/smithay/pull/2231 .

## References

- brief (input, required, kind=brief): docs/notes/2026-10-06-render-anomalies-brief.md — read the constraints and existing evidence before this follow-up
