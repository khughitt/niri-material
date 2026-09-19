---
id: material-743692
title: "Prism: signal tuning parameters"
status: idea
priority: 2
size: s
complexity: mid
process: planned
created: 2026-09-02T12:09:35Z
updated: 2026-09-19T02:49:37Z
depends: [material-82323e, prism-eff23a]
parent: material-a76720
tags: [signals, prism]
---

Outcome: ring-inset, ring-width, oscillator periods, and impulse constants exposed as ordinary glass params in prism defs/glass.yaml and rendered into the response "default" block by the niri sink. Requires the native config to accept those constants first; today the oscillator and envelope constants are fixed in the solver. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

Scope refinement (2026-09-18): this original source mixes already-shipped controls, stable placement and unapproved motion knobs. prism-28e29c already provides focus/color-source/color/driftHz. Track missing placement through prism-d8ee06; reuse prism-0ea68f for light-ior and prism-71b7d1 for native geometry regression. Do not expose oscillator periods or impulse constants until material-82323e reviews the native motion contract for material-0e130e. This remains a coordination idea, not a duplicate implementation task. Done means approved controls persist/reset in Prism and validate in both native materials, with familiar/noctalia/manual behavior intact. Handoff: docs/notes/2026-09-18-ring-next-steps-brief.md.

## Notes

- 2026-09-19T00:26:26Z (materials-26.04): scope: briefed; separated visual placement, native motion design and existing/new Prism wiring; brief: docs/notes/2026-09-18-ring-next-steps-brief.md
- 2026-09-19T02:49:26Z (material-82323e): Finding from material-82323e: Prism's contract is spec §5 — glass.ring.driftHz becomes glass.ring.sweepMs (0-10000, default 1500) via a 'replaces:' declaration; 'prism migrate' rewrites base and every profile/wallpaper context (0->0, positive->1500, an existing sweepMs kept) with a backup and report; doctor names the command; rollout is install build, migrate+apply, restart, with rollback restoring the backup first. idle-after-ms stays native. Oscillator/impulse constants remain unexposed.
