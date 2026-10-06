---
id: material-d1171f
title: Does backdrop grain clipping lift dark texels under blur?
status: idea
priority: 2
created: 2026-10-06T08:56:17Z
updated: 2026-10-06T08:56:17Z
depends: []
tags: [material, noise]
agent: claude-code/claude-opus-5-5
---

The 2026-10-05 roughness cell showed backdrop grain under a blurred roughness-1 pyramid raising blue by one code on 55% of face pixels (mean +0.55), on a backdrop whose plasma has 24% of blue texels within the grain's ±38-code span of 0. If the grained 8-bit source clips at 0, blur averages a one-signed bias into dark regions: a slight lift the glass site does not have. Check with a dark flat backdrop at backdrop vs glass, blurred, and decide whether the grain pass should run in a wider format or accept it.
