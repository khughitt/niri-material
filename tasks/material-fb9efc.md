---
id: material-fb9efc
title: "Rigid tilt swing: pull-in not linear in sin(tilt)/distance, and the far edge moves at tilt 20 / distance 6"
status: idea
priority: 2
created: 2026-10-29T19:19:22Z
updated: 2026-10-29T19:19:22Z
depends: []
tags: [rendering, camera]
agent: claude-code/claude-opus-5-5
origin: agent
---

The 2026-10-26 amplitude ladder (material-c5f9d0, tilt-amplitude-ladder.png; evidence doc docs/materials/2026-10-09-rigid-tilt-evidence.md, Clip) measured origin-side pull-in of 7.3 / 22.0 / 27.9 px at view-tilt/view-distance 10/6, 20/6, 20/3 against a first-order W sin(tilt) / 2·distance of 8.7 / 17.2 / 34.4 px. At 20/6 the far (hinge) edge also pulls in 12.2 px (pane edge at x~33 against x~24 at rest in f002), where 10/6 and 20/3 hold it within 1 px. Undiagnosed: is it the angle-scaled camera, the culling footprint, or the measurement? It matters before any setting near 20/6 ships, and to whoever revisits the tilt model; the shipped pick (20/3) is not affected as measured.

## Notes

- 2026-10-29T19:19:22Z (add): created
  provenance: {"harness_session":"claude-code:2db0caf2-46d5-402c-b4aa-03b22c49e8bc","harness_session_source":"CLAUDE_CODE_SESSION_ID","origin":"agent"}
