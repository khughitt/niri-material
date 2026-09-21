---
id: material-a2684b
title: Scrub machine layout from tracked files and commit ops-check 5
status: done
priority: 2
size: s
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-21T13:06:06Z
updated: 2026-09-21T20:28:58Z
started: 2026-09-21T20:27:04Z
completed: 2026-09-21T20:28:58Z
depends: []
tags: []
source: ops-0c42b9
model: "claude-opus-5[1m]"
agent: "claude-code/claude-opus-5[1m]"
---

ops-check 5 (ops ops-0c42b9) fails on this machine's layout in any tracked file: the home directory, the hostname, WORK_ROOT, a registered checkout or its parent, the banned tracker host. tools/ops-check is already updated in the working tree but uncommitted, because the pre-commit hook refuses every commit here until these findings are gone. Per file: rewrite the path or hostname neutrally; drop the file when it is captured scratch that does not belong in the repository; or, for evidence that must stay verbatim, list its path prefix under layout_allowed in a root .ops-check.toml. Commit tools/ops-check in the same change.

Findings (23 lines in 8 files):
- docs/materials/2026-09-05-material-glass-noise-saturation-params-evidence.md: a registered checkout
- docs/materials/2026-09-12-material-render-order-evidence.md: the hostname
- docs/materials/2026-09-18-ring-focus-motion-evidence.md: the hostname
- docs/materials/2026-09-19-ring-beam-evidence.md: the hostname
- docs/materials/plans/2026-08-28-v1-daily-driver-rollout.md: the hostname
- docs/plans/2026-09-05-material-glass-noise-saturation-params.md: a registered checkout
- docs/plans/2026-09-10-material-optics.md: a registered checkout
- docs/plans/2026-09-11-material-capture-protocol.md: a registered checkout, the parent of a registered checkout

## Notes

- 2026-09-21T20:27:04Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:ae4ede5c-83d3-4b16-bb13-8f85451d7a6b","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-21T20:28:58Z (materials-26.04): done
  provenance: {"harness_session":"claude-code:ae4ede5c-83d3-4b16-bb13-8f85451d7a6b","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-21T20:28:58Z (materials-26.04): Rewrote the 23 layout lines in 8 docs neutrally (hostname -> the verification host / $(hostname); main checkout -> $MAIN, $NIRI_MATERIAL, or checkout-relative); no layout_allowed needed. Committed tools/ops-check 5.
  provenance: {"harness_session":"claude-code:ae4ede5c-83d3-4b16-bb13-8f85451d7a6b","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
