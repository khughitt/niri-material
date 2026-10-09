---
id: material-ac926f
title: "Spike: side-exit attenuation and resting ring spill on the live looks"
status: doing
priority: 1
size: s
complexity: mid
process: direct
owner: materials-26.04
created: 2026-10-09T23:29:33Z
updated: 2026-10-09T23:29:40Z
started: 2026-10-09T23:29:40Z
depends: []
parent: material-6062fd
tags: [spike, rendering]
agent: claude-code/claude-fable-5-1
---

Question: do a side-wall-clipped attenuation path on the bevel and a resting ring spill into the bevel make the glass edge read as one slab instead of a dark frame plus an inset line? Probe, throwaway: in a worktree, (1) in prelude.frag/main.frag bound the Beer-Lambert path by the distance to the silhouette along the refracted ray (taps unchanged), (2) scale the ring's bevel spill by the same moving + rest sum the focus glow uses. Dump the frozen glass-edge cases and the ring-look fixture before and after (GLASS_EDGE_DUMP, RING_LOOK_DUMP; no host needed), on the live active look (ior 1.28, thickness 31.2, #4c563a at 9), the live inactive look, and one lighter look, since the dark terminal fill hides the glass. Deliverable: a labelled before/after sheet (goal title, row and column headers, decoded params, control marked, cell ids) for the owner's verdict, and a note here with the numbers (bevel transmission across u before and after). No production shader, no spec, no live capture. Verification: just test-one -p niri every_case_renders_frozen and accepted_ring_look render on both builds; opaque client pixels and everything outside the slab identical before and after.

## Notes

- 2026-10-09T23:29:40Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:ff22882b-dfa1-4b0d-a989-d026665b0306","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
