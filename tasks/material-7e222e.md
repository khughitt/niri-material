---
id: material-7e222e
title: Drop glass-noise-type-smoke.sh's own signed_diff now that the lib carries the a - b fix
status: dropped
priority: 4
size: xs
complexity: low
process: direct
owner: noise-followups
created: 2026-10-08T09:13:39Z
updated: 2026-10-08T09:40:28Z
started: 2026-10-08T09:39:41Z
depends: []
tags: [harness]
agent: claude-code/claude-opus-5-5
---

After the noise-layers merge, glass-optic-smoke-lib.sh defines metric/signed_diff (with ff8e4a99's a - b args); glass-noise-type-smoke.sh still defines its own signed_diff after sourcing the lib, shadowing it. Remove the duplicate and confirm the type smoke's metrics keep their sign.

## Notes

- 2026-10-08T09:39:41Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:895d792f-a7ab-466b-afa7-8d313f21a46d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-08T09:39:59Z (noise-followups): resumed
  provenance: {"harness_session":"claude-code:895d792f-a7ab-466b-afa7-8d313f21a46d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-08T09:40:28Z (noise-followups): dropped
  provenance: {"harness_session":"claude-code:895d792f-a7ab-466b-afa7-8d313f21a46d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-08T09:40:28Z (noise-followups): Premise false: glass-noise-type-smoke.sh never sources glass-optic-smoke-lib.sh (standalone, own fail/compare_metric/sd/signed_diff), so nothing is shadowed; its signed_diff already computes a - b since ff8e4a99. The noise-layers plan (Task: lib gains metric/signed_diff/shot_twice) moved them out of glass-noise-site-smoke.sh, which holds no copy now. Removing the type smoke's copy would break it.
  provenance: {"harness_session":"claude-code:895d792f-a7ab-466b-afa7-8d313f21a46d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
