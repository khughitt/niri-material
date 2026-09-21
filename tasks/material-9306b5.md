---
id: material-9306b5
title: Tune stock ring inset against the refracted within ring
status: done
priority: 2
created: 2026-09-18T23:33:13Z
updated: 2026-09-21T09:27:31Z
completed: 2026-09-21T09:27:31Z
depends: [material-92edaf]
parent: material-a76720
tags: [rendering]
source: "2026-09-12-material-render-order:stock-inset-captures"
agent: codex
---

The render-order acceptance captures support revisiting stock placement: within-pinned-on.png (ring-inset 5, bevel 12) reads as a luminous rim with an inward halo, not a clearly separated embedded band. within-face-on.png (inset 20) demonstrates available face placement but is an acceptance extreme, not a proposed default. Compare retained images under $NIRI_MATERIAL_WORK_ROOT/render-order-within-pixels.eS4KzB/run and scope a small visual tuning comparison after material-92edaf. Defaults remain unchanged in render-order work; choose the final inset with user visual judgment, not from reach bounds. Motion remains separately deferred to material-0e130e.

## Notes

- 2026-09-20T01:02:49Z (materials-26.04): Owner review 2026-09-19 after installing r436 (first live build with the within ring, d5b4188a): the band reads as a flat metallic bevel rather than glass with a light inside it, and the terminal body now appears to float above the ring — body and band look more separate than before. Placement/halo/depth are the questions; pair with the travel-pattern finding on material-a76720.
- 2026-09-21T09:27:31Z (material-2c3984): done
  provenance: {"harness_session":"claude-code:91b58500-0687-468f-b430-142cc24f6ede","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-21T09:27:31Z (material-2c3984): subsumed: the beam runs under the face at ring-gap from its edge; accepted on the ring beam sheets
  provenance: {"harness_session":"claude-code:91b58500-0687-468f-b430-142cc24f6ede","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
