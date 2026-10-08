---
id: material-7e222e
title: Drop glass-noise-type-smoke.sh's own signed_diff now that the lib carries the a - b fix
status: doing
priority: 4
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-10-08T09:13:39Z
updated: 2026-10-08T09:39:41Z
started: 2026-10-08T09:39:41Z
depends: []
tags: [harness]
agent: claude-code/claude-opus-5-5
---

After the noise-layers merge, glass-optic-smoke-lib.sh defines metric/signed_diff (with ff8e4a99's a - b args); glass-noise-type-smoke.sh still defines its own signed_diff after sourcing the lib, shadowing it. Remove the duplicate and confirm the type smoke's metrics keep their sign.

## Notes

- 2026-10-08T09:39:41Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:895d792f-a7ab-466b-afa7-8d313f21a46d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
