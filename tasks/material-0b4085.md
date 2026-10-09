---
id: material-0b4085
title: Retire glass-render-order-smoke.sh's pixel-only capture-meta waiver for the pixels lane
status: idea
priority: 2
created: 2026-10-09T01:15:14Z
updated: 2026-10-09T01:15:14Z
depends: []
tags: [capture]
agent: claude-code/claude-opus-5-5
---

SCOPE=pixels runs under a stubbed CAPTURE_META recorded as gpu-quietness=pixel-only-waiver. Spec §3.1/§10: with --lane pixels the scope can keep a real capture record; check that pixel_matrix and within_matrix under SCOPE=pixels record no wall-clock values first.
