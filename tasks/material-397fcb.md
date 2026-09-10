---
id: material-397fcb
title: Simplify the material API so new materials are easy to author
status: doing
priority: 2
size: l
owner: feat/material-397fcb
created: 2026-09-06T22:27:52Z
updated: 2026-09-10T09:33:23Z
depends: []
tags: [material, api, community]
source: "mindful:thought:5778c060e57d47dd808e20347223cfd5"
spec: docs/specs/2026-09-10-material-optics-design.md
---

Goal: the glass pipeline becomes a slab plus an ordered list of self-contained optics, so a new stage is four files, three one-line registrations, and a docs section. Design: docs/specs/2026-09-10-material-optics-design.md. Children carry the work; ice (material-bb3fe5) and aurora/rainbow (material-f0fc7b) are the first users.
