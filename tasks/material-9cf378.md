---
id: material-9cf378
title: "niri-experiments: idle-budget and jelly-motion adopt renderer verification and build-before-preflight"
status: doing
priority: 2
size: s
complexity: low
process: direct
owner: materials-26.04
created: 2026-10-09T01:15:14Z
updated: 2026-10-09T11:09:31Z
started: 2026-10-09T11:09:31Z
depends: []
parent: material-2834d7
tags: [capture]
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-10-08-capture-host-conditions-design.md
---

Spec §6.4/§10. In niri-experiments: idle-budget.sh power start_drm sets RUST_LOG=$NIRI_RENDERER_LOG and calls verify_renderer "$name" "$OUT/niri.log" <offset> after launch; fixtures/test_idle_budget.py asserts it; idle-budget.sh and jelly-motion.sh build, await_load, then capture_preflight; jelly-motion.sh drops its own weston.log GL renderer grep. idle-budget.sh's `capture_meta release "$OUT" || true` must propagate release's status, or a refused release still ends the run clean. Until this lands, capture-meta release exits 1 on idle-budget power runs (their start_drm launches never call renderer) and `show` marks those settled launches 'renderer unchecked', but idle-budget's || true swallows the exit.

## Notes

- 2026-10-09T11:09:31Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:b6f2497c-748f-44b4-8e1c-9de00d2b47e1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
