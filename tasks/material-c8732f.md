---
id: material-c8732f
title: GL_INVALID_VALUE 'Size and/or offset out of range' bursts in the niri log
status: idea
priority: 2
created: 2026-09-06T00:47:17Z
updated: 2026-09-06T00:47:17Z
depends: []
tags: [rendering, bug]
---

journalctl for the running compositor (d47f675a, 2026-09-05 20:22 to 20:28 local) shows repeated 'ERROR smithay::backend::renderer::gles: [GL] GL_INVALID_VALUE error generated. Size and/or offset out of range.' Seen while investigating prism-66b025; not yet correlated with an action. Check whether it recurs on the 5dbe182d build and which draw emits it.
