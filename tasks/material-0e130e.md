---
id: material-0e130e
title: Use one-shot focus ring effects; gate sustained attention motion on visibility and activity
status: idea
priority: 1
size: m
complexity: high
process: planned
created: 2026-09-16T12:00:25Z
updated: 2026-09-16T12:00:25Z
depends: []
parent: material-5d6b2c
tags: [performance, dynamics, signals]
agent: codex
---

Replace continuously pulsing or drifting ordinary focus-ring light with a finite one-shot effect, such as a single pulse on focus gain, then settle with no periodic redraw solely for focus. A static focus indication may remain. Alert/attention state may justify sustained dynamics only while BOTH the window is actually visible on an active workspace/output and the user is active rather than in extended keyboard/mouse inactivity. Preserve static alert state while hidden or idle; visibility/activity changes must not restart an endless focus pulse. Reuse existing impulse/envelope and motion-policy mechanisms where they fit. Design pulse duration/retrigger behavior, input-idle threshold, resume semantics and reduced-motion behavior before implementation. Verify ordinary focus settles, hidden/idle attention stops scheduling animation work, and visible active attention still signals appropriately. Coordinate with material-7afc31 (visibility), material-f86183 (inactivity settle), material-265eb0 (idle budget), and material-6d4de5 (dynamics review). Source: user request in render-order session, 2026-09-16.
