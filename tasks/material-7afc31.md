---
id: material-7afc31
title: Skip material rendering for windows that are not visible
status: idea
priority: 2
created: 2026-09-11T23:34:15Z
updated: 2026-09-11T23:34:15Z
depends: []
parent: material-5d6b2c
tags: [quick-add, performance]
source: "mindful:thought:a476e6bcd1fd4297b70824758235d821"
---

Check whether occluded windows, windows on inactive workspaces, or windows on other outputs still run the material shader or its prefilter passes. If they do, gate that work on visibility. Measure the saving with the capture protocol before deciding whether the complexity is worth it.

Source: mindful:thought:a476e6bcd1fd4297b70824758235d821
