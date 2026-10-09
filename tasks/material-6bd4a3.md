---
id: material-6bd4a3
title: Check whether per-case capture settles hold with the glass desktop up
status: idea
priority: 1
created: 2026-09-13T14:03:11Z
updated: 2026-10-09T01:15:31Z
depends: []
parent: material-2834d7
tags: []
---

Render-order readiness on 2026-09-12 refused at 23.5% GPU utilization with P5/P8 transitions while only niri and Noctalia were running, against a 5% / P8-throughout threshold. A never-static glass compositor may never pass that on the live desktop, so every capture in the headless lane may actually need a no-desktop (TTY or isolated DRM) session. Decide per lane which host condition the preflight really needs and set the thresholds to match; record it so quiet-queue entries (tasks-811e02) can say idle vs headless correctly.

2026-09-16: elevate because readiness has repeatedly blocked render-order work. Preserve unchanged thresholds until this investigation justifies a documented lane change. Latest ordinary-workspace sample CJw4OM was 37% GPU/P5, load 2.2; after switching to an empty workspace, SxYENG passed at 3% GPU/P8, CPU 2.3%, load 0.53. Thus the current thresholds are meetable with the desktop up in at least one observed state. Compare desktop content and client/compositor attribution rather than assuming every live-desktop capture requires a TTY.

## Notes

- 2026-09-16T12:00:25Z (materials-26.04): Raised P2 to P1 at user request: prioritize avoiding invisible-window and idle resource waste; distinguish observed client GPU use from unproven compositor rendering.
- 2026-10-06T19:17:33Z (materials-26.04): scope: briefed; desktop-up preflight meetability is already observed, but repeated settles and lane isolation remain separate; retain thresholds and frame the evidence-class decision in a shared design; brief: docs/notes/2026-10-06-capture-lifecycle-brief.md
- 2026-10-06T21:17:04Z (materials-26.04): Retitled in review: the original question (preflight meetable with the desktop up) is answered by the 2026-09-16 empty-workspace pass; what remains is whether repeated settles hold, framed by material-18c2a1.
- 2026-10-09T01:15:31Z (capture-host-conditions): Reviewed decisions (material-18c2a1, docs/specs/2026-10-08-capture-host-conditions-design.md): nested measurements stay on a TTY until the desktop-idle pilot (material-3d48b0, under material-2834d7) passes against the pinned TTY reference; static pixel fixtures run on a desktop in use via --lane pixels; every measured launch must verify its renderer against the sampled GPU; dedicated refuses a live desktop and GPU clients before the hold. Thresholds unchanged.
