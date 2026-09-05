---
id: material-5a5fff
title: Interpolate glass parameters across a focus material swap
status: idea
priority: 2
created: 2026-09-05T01:10:50Z
updated: 2026-09-05T01:10:50Z
depends: []
tags: [material]
---

If the is-focused material swap reads as a visual pop, animate between the two resolved parameter sets using niri's existing animation/spring machinery rather than a hard cut. Only worth scoping after material-cb348e lands and the pop is observed.
