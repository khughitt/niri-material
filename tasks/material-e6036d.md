---
id: material-e6036d
title: "Nested state machines wired to material properties, with adjustable noise terms"
status: shelved
priority: 2
created: 2026-09-06T22:27:52Z
updated: 2026-09-29T22:44:57Z
depends: []
parent: material-53f873
tags: [signals, design, noise]
source: "mindful:thought:5778c060e57d47dd808e20347223cfd5"
---

Model material state as nested state machines (window state within workspace state within session/ambient state) and wire state signals to material properties explicitly rather than ad hoc per response. Add adjustable noise terms so properties can carry organic jitter. Builds on the signals design (2026-09-02-material-signals-design.md) and the signal-level window-rule idea (material-d88a8f).

## Notes

- 2026-09-29T22:44:57Z (materials-26.04): shelved: A concrete window/workspace/session transition with a reproducible sequence cannot be expressed using existing signal slots, named responses and native animations; identify the missing behavior and an acceptance check, then unshelve for design.
- 2026-09-29T22:44:57Z (materials-26.04): scope: shelved; retain the hierarchical-state and organic-noise proposal until a concrete transition cannot be expressed by existing signals, response blocks and native animations; brief: docs/notes/2026-09-29-material-dynamics-brief.md
