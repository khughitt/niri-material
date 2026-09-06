---
id: material-6d4de5
title: "Revisit the dynamics: drag, move, focus, flex, ripple"
status: idea
priority: 2
created: 2026-09-06T00:32:54Z
updated: 2026-09-06T00:32:54Z
depends: [prism-66b025]
tags: [rendering, dynamics]
---

Reassess how the glass responds to drag, move, and focus changes. Prism reports that the Flex and Ripple sliders have no visible effect; if the values reach the renderer, the jelly math (render_helpers/material.rs) or its gating may need to change.
