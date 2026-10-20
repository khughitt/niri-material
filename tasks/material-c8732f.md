---
id: material-c8732f
title: GL_INVALID_VALUE 'Size and/or offset out of range' bursts in the niri log
status: idea
priority: 2
reviewed: true
created: 2026-09-06T00:47:17Z
updated: 2026-10-20T21:43:48Z
depends: [material-86c841]
parent: material-7ff4bc
tags: [rendering, bug]
---

journalctl for the running compositor (d47f675a, 2026-09-05 20:22 to 20:28 local) shows repeated 'ERROR smithay::backend::renderer::gles: [GL] GL_INVALID_VALUE error generated. Size and/or offset out of range.' Seen while investigating prism-66b025; not yet correlated with an action. Check whether it recurs on the 5dbe182d build and which draw emits it.

## Current scope (2026-10-20)

The caller remains unknown. Upstream search on 2026-10-20 covered open/closed niri and Smithay issues/PRs for GL_INVALID_VALUE and the exact message, Smithay master update_memory, branch inventory and relevant development/resize branch material, and CHANGELOG.md. Smithay PR #1933 matches the exact message but is closed unmerged; maintainers ask for caller-level attribution rather than a GLES-only clamp. niri #4188 is another glTexSubImage2D bounds report, not proof of our cause. Smithay #2231 is open multigpu copy work, also unproven here. No verified fixing release/commit was established: do not pin or carry a workaround yet. Reuse material-86c841 to compare these candidates against pinned Smithay ff5fa7df and actual binary provenance before one minimal reproduction. Upstream links: https://github.com/Smithay/smithay/pull/1933 ; https://github.com/niri-wm/niri/issues/4188 ; https://github.com/Smithay/smithay/pull/2231 .

Read [ref:brief] for the existing alternatives, constraints and bounded handoff.

## References

- brief (input, required, kind=brief): docs/notes/2026-10-06-render-anomalies-brief.md — existing scope handoff and follow-up boundaries

## Notes

- 2026-10-06T20:44:03Z (materials-26.04): scope: briefed; historical GL_INVALID_VALUE signature recurs four times on 2026-10-06 in the logged 310b4e30 compositor; exact GL caller and trigger remain unknown; separate bounded attribution from any fix; brief: docs/notes/2026-10-06-render-anomalies-brief.md
