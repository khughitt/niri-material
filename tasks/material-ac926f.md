---
id: material-ac926f
title: "Spike: attenuation-tinted edge light and resting ring spill on the live looks"
status: done
priority: 1
size: s
complexity: mid
process: direct
owner: materials-26.04
created: 2026-10-09T23:29:33Z
updated: 2026-10-17T16:37:05Z
started: 2026-10-09T23:29:40Z
completed: 2026-10-17T16:37:05Z
closed: 2026-10-17T16:37:05Z
depends: []
parent: material-6062fd
tags: [spike, rendering]
agent: claude-code/claude-fable-5-1
---

Question: do an attenuation-tinted edge light at the rim and a resting ring spill into the bevel make the glass edge read as one slab instead of a dark frame plus an inset line? Probe, throwaway: in a worktree, (1) in reflection.frag weight the scene sample by a rim profile peaking at the silhouette, tinted by attenuation-color over the path back to the face edge, alongside the Fresnel term; (2) in main.frag scale the ring's bevel spill by the same moving + rest sum the focus glow uses. A spike-only test renders the live active look (prism.kdl, 2026-10-09), the live inactive look, and one lighter look, over a flat and a checker backdrop, with kitty-like translucent content, at a beam instant and at rest; dumps before and after (no host needed). Deliverable: a labelled before/after sheet (goal title, row and column headers, decoded params, control marked, cell ids) for the owner's verdict, and a note here with the numbers. No production shader, no spec, no live capture. Verification: the existing frozen cases render on both builds; opaque client pixels and everything outside the slab identical before and after.

## Notes

- 2026-10-09T23:29:40Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:ff22882b-dfa1-4b0d-a989-d026665b0306","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T23:29:54Z (spike/material-ac926f): resumed
  provenance: {"harness_session":"claude-code:ff22882b-dfa1-4b0d-a989-d026665b0306","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T23:47:24Z (spike/material-ac926f): attached: edge-cohesion-sheet.png (483947 bytes): Owner sheet: control vs edge light + resting spill, three looks x three backdrops, rest corner and top edge, plus the accepted ring look
- 2026-10-09T23:47:24Z (spike/material-ac926f): Spike result 2026-10-09 (branch spike/material-ac926f, dumps under /mnt/ssd3/niri-material/material-ac926f: before2/after2 spike cases, before/after2 the 18 glass-edge cases, ring-before/ring-after). Correction first: the side-exit path idea was wrong physics (the refracted ray bends inward and exits the back; h / -t.z is right), so the probe is an attenuation-tinted edge light riding the reflection optic (rim = u^2, tint = attenuation-color^((1-u) * bevel / attenuation-distance), scene averaged over 5 taps along the edge to kill corner spokes) plus a resting ring spill (BEAM_SPILL * total glow * (1-u), accent and rest included, outside the focus branch). Numbers, active look over the flat backdrop, left bevel at mid-height, green channel code values from silhouette to face edge: control 67 49 43 41 41 41 41 42 42 43 43 (flat); candidate 98 80 69 62 57 54 51 50 50 50 50 (gradient). Light glass: control 137..123 flat; candidate 146..123. Neutrality on the 18 existing cases: outside the slab, the opaque window, and every face region 0 differing pixels; reflection-0 cases identical overall; bevel deltas live-reflection rmse 18.1 (max 63), stock-reflection 10.9 (max 29), live-on 4.7 (max 10) from the spill, live-highlight 0. Caveats: the fixture's resting accent line is far dimmer than the desktop's teal line (ring at +7 code values), so the line-meets-edge question is only partly answered; no comet head was present at 100 ms in the spike cases (the beam columns show the top edge only); row J (accepted ring look) has no reflection node, so only the spill acts there; the live active look has backdrop-blur on, which the fixture pins off.
- 2026-10-09T23:50:13Z (spike/material-ac926f): parked (waiting on user, review): Owner judges the attached sheet (tasks show material-ac926f prints its path; also /mnt/ssd3/niri-material/material-ac926f/sheet/edge-cohesion-sheet.png) and names cells. On a yes, start material-f72d5c (planned: spec amendment, schema reads declared, bevel.rs mirror) and material-58a1a1 (resting spill); on a no, drop both and note why. The spike branch spike/material-ac926f and the worktree .worktrees/material-ac926f are throwaway: remove after the verdict (tt-report first).
  provenance: {"harness_session":"claude-code:ff22882b-dfa1-4b0d-a989-d026665b0306","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-09T23:50:24Z (spike/material-ac926f): Pinned tests the throwaway breaks (expected; they name what the production change must declare): pipeline_optic_glsl_reads_are_declared (reflection's GLSL reads attenuation-color, attenuation-distance and bevel, which its schema reads omit) and material_source_is_prelude_then_optics_in_order_then_main (pins the beam-only spill line). All other material, ring, bevel, glass-edge and noise tests pass (155 of 157 in the filter).
- 2026-10-17T16:36:59Z (materials-26.04): resumed
  provenance: {"harness_session":"claude-code:05694925-34c9-4a16-ae62-2de8550ed7aa","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T16:37:05Z (materials-26.04): done
  provenance: {"harness_session":"claude-code:05694925-34c9-4a16-ae62-2de8550ed7aa","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-17T16:37:05Z (materials-26.04): Owner verdict 2026-10-17: accept — the edge light plus resting ring spill reads as one slab on the sheet. Proceeding with material-f72d5c (edge light, planned) and material-58a1a1 (resting spill). Spike branch and worktree removed; the spike's full commit is retained at /mnt/ssd3/niri-material/material-ac926f/spike-commit-9d1315ea.patch beside the v1/v2 shader patches and dumps.
  provenance: {"harness_session":"claude-code:05694925-34c9-4a16-ae62-2de8550ed7aa","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
