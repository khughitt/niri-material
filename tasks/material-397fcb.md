---
id: material-397fcb
title: Simplify the material API so new materials are easy to author
status: done
priority: 2
size: l
owner: feat/material-397fcb
created: 2026-09-06T22:27:52Z
updated: 2026-09-10T12:57:23Z
depends: []
tags: [material, api, community]
source: "mindful:thought:5778c060e57d47dd808e20347223cfd5"
spec: docs/specs/2026-09-10-material-optics-design.md
plan: docs/plans/2026-09-10-material-optics.md
---

Goal: the glass pipeline becomes a slab plus an ordered list of self-contained optics, so a new stage is four files, three one-line registrations, and a docs section. Design: docs/specs/2026-09-10-material-optics-design.md. Children carry the work; ice (material-bb3fe5) and aurora/rainbow (material-f0fc7b) are the first users.

## Notes

- 2026-09-10T09:36:12Z (feat/material-397fcb): parked (waiting on user): User reviews docs/specs/2026-09-10-material-optics-design.md; on approval run writing-plans for the goal (children material-1135bc, material-6ecd63) in the feat/material-397fcb worktree
- 2026-09-10T10:24:41Z (feat/material-397fcb): parked (waiting on user): User picks execution mode for docs/plans/2026-09-10-material-optics.md (subagent-driven or inline); then start Task 1 (material-cc6d07) in the feat/material-397fcb worktree
- 2026-09-10T10:37:56Z (feat/material-397fcb): Plan review 2026-09-10: scalar bounds tested via knuffel::parse of Glass (no Material::validate), pixel compare via magick AE with exit status, focused fixture starts active, upstream-report regenerated before every commit, tests through tools/tt with one filter
- 2026-09-10T12:57:23Z (feat/material-397fcb): Material optics API sections 1–6 landed with generated parameter metadata, registry-driven rendering, independent deadlines, contributor docs, and decoded-pixel identity evidence.
