---
id: material-72f410
title: Correct timer fixture dispatch and leak assertions in the ring motion plan
status: done
priority: 2
size: xs
complexity: low
process: direct
owner: material-82323e
created: 2026-09-19T09:50:39Z
updated: 2026-09-19T09:51:59Z
started: 2026-09-19T09:50:43Z
completed: 2026-09-19T09:51:59Z
depends: []
parent: material-0e130e
tags: [docs]
agent: codex
---

Apply the final plan review: explicitly dispatch the nested server event loop at timer-only checkpoints, and observe idle callback activity after cancelled deadlines so leaked timers cannot pass on unchanged state. Revise the implementation plan only; do not implement the feature.

## Notes

- 2026-09-19T09:50:43Z (material-82323e): started
  provenance: {"harness_session":"codex:01a0b756-0182-7091-918b-c55fd64f987f","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-19T09:51:59Z (material-82323e): done
  provenance: {"harness_session":"codex:01a0b756-0182-7091-918b-c55fd64f987f","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-19T09:51:59Z (material-82323e): Revised the plan to dispatch the nested server loop at all timer checkpoints, count idle callbacks in test builds, and assert no callback occurs after both cancelled deadlines.
  provenance: {"harness_session":"codex:01a0b756-0182-7091-918b-c55fd64f987f","harness_session_source":"CODEX_SESSION_ID"}
