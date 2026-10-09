---
id: material-9cf378
title: "niri-experiments: idle-budget and jelly-motion adopt renderer verification and build-before-preflight"
status: todo
priority: 2
size: s
complexity: low
process: direct
created: 2026-10-09T01:15:14Z
updated: 2026-10-09T01:15:14Z
depends: []
parent: material-2834d7
tags: [capture]
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-10-08-capture-host-conditions-design.md
---

Spec §6.4/§10. In niri-experiments: idle-budget.sh power start_drm sets RUST_LOG=$NIRI_RENDERER_LOG and calls verify_renderer "$name" "$OUT/niri.log" <offset> after launch; fixtures/test_idle_budget.py asserts it; idle-budget.sh and jelly-motion.sh build, await_load, then capture_preflight; jelly-motion.sh drops its own weston.log GL renderer grep. Until this lands, release refuses their measurement runs.
