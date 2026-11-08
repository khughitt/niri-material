---
id: material-6901e0
title: "View tilt: a geometric tilt that moves window content and glass together"
status: doing
priority: 1
process: direct
owner: material-abc08c
created: 2026-09-22T23:46:32Z
updated: 2026-11-08T20:15:05Z
started: 2026-11-08T20:02:05Z
depends: [material-77db8a]
related: [material-abc08c, material-fc84e5]
parent: material-53f873
tags: [rendering, camera]
agent: "claude-code/claude-opus-5-5[1m]"
---

The optical-only view tilt (material-77db8a, spec docs/specs/2026-09-22-focus-view-tilt-design.md) was rejected in review on 2026-10-06: the terminal text stays static while the glass optics swing, which breaks the goal of one cohesive material. Any tilt or perspective must move the window's content and the glass as one body: the slab, its optics and the window pixels share one transform.

Costs to weigh when scoping: text softening while the pane moves (resampling under a projective transform), element geometry, damage and hit-testing during the swing, and whether the static by-position perspective survives at all once content must follow it.

Reusable from the unmerged branch material-77db8a: the swing curve and settle (render_helpers/material/view.rs), the Layout's focus origin (layout/focus_origin.rs), the swing run on the tile, the config keys, and the smoke and clip scripts. The shader's view-ray routing may still serve as the optical half of a geometric tilt.

## Notes

- 2026-10-06T21:03:54Z (materials-26.04): scope: briefed; P1 under lane material-53f873; rigid in-shader tilt is the lean (window and glass as one body, backdrop at true screen position); spike material-89fb6b (owner stills) then design material-abc08c; brief: docs/notes/2026-10-06-transient-depth-brief.md
- 2026-10-06T22:10:40Z (materials-26.04): finding (material-89fb6b, 2026-10-06): a planar in-shader tilt reads to the owner as one object and its text softening is acceptable; side walls are missing first, interior detail second. Next: material-abc08c, building side walls on material-be611b's height-field edge.
- 2026-10-09T20:43:19Z (material-abc08c): design (material-abc08c, 2026-10-09): transient rigid tilt accepted at spec round 4 — focus-gain swing about the pane centre, angle-scaled camera so rest is today's path, ray-traced slab on material-be611b's edge profile shows the side wall; spec docs/specs/2026-10-09-transient-rigid-tilt-design.md; brief result section added; follow-ups material-98beac, material-df50f2, prism-2a8fea; implementation plan next
- 2026-11-08T20:02:05Z (material-abc08c): started
  provenance: {"harness_session":"claude-code:f2252ded-1918-4960-a1f1-d29c8039757f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-11-08T20:02:05Z (material-abc08c): retained: {"v":1,"host":"titan","kind":"evidence","path":"/mnt/ssd3/niri-material/ring-motion-clips-f905c9ed/ring-clips-978014-1793855426","state":"kept","verification":"verified","purpose":"2026-11-05 sign and settle clips at L3 behind the owner's 2026-11-08 verdict (shipped sign, 900 ms): GIFs, swing-measured-sheet.png, swing-measured.json, capture.json; src-flipped/ and target-scratch/ are the rejected flipped build's regenerable inputs, outside the manifest","next_owner":"task:material-6901e0","cleanup_trigger":"after material-abc08c merges into materials-26.04","manifest":"/mnt/ssd3/niri-material/ring-motion-clips-f905c9ed/ring-clips-978014-1793855426/manifest.json","sha256":"9696b0fb76bf2dfd9c9b64b2783d2fb3fce646152a7cd5c6c2823d034a915e1b"}
- 2026-11-08T20:02:05Z (material-abc08c): retained: {"v":1,"host":"titan","kind":"scratch","path":"$WORK_ROOT/niri-material/material-c5f9d0","state":"kept","verification":"verified","purpose":"ladder-sheet.py, tilt-amplitude-ladder.png (also attached to material-c5f9d0), and ref-1006-remeasure/ from the amplitude ladder","next_owner":"task:material-6901e0","cleanup_trigger":"after material-abc08c merges into materials-26.04","manifest":"$WORK_ROOT/niri-material/material-c5f9d0/manifest.json","sha256":"9130c1ef44c9b4b5056807a687d945b1d62b3e9db326eaf44456b4c2edb2b386"}
- 2026-11-08T20:02:05Z (material-abc08c): retained: {"v":1,"host":"titan","kind":"worktree","path":".worktrees/material-abc08c","state":"kept","verification":"verified","purpose":"material-abc08c branch (plan steps all done) and SDD workspace .superpowers/sdd/2026-10-09-transient-rigid-tilt","next_owner":"task:material-6901e0","cleanup_trigger":"after material-abc08c merges and the ledger rulings and deferred minors reach the task record"}
- 2026-11-08T20:02:05Z (material-abc08c): retention checkpoint: verified 2026-11-08 on titan by this session; evidence manifest 124 listed files all present and unchanged (manifest sha256 9696b0fb...; src-flipped/ and target-scratch/ unlisted), scratch manifest 114 files unchanged with nothing unlisted (sha256 9130c1ef...), worktree present; received from material-c5f9d0 at its close
- 2026-11-08T20:03:36Z (material-abc08c): process: direct — goal closeout: every plan step is done; remaining work is the main-into-branch merge, checks, local integration and cleanup
- 2026-11-08T20:03:53Z (material-abc08c): deferred minors from every task review and both final reviews filed as material-fc84e5 (open owner)
- 2026-11-08T20:14:49Z (material-abc08c): review: impl round 1 — verdict: accept; findings: minor 1; reviewer: claude-code/claude-opus-5-5
- 2026-11-08T20:15:05Z (material-abc08c): base merge 418b0130 (materials-26.04 into the branch): conflicts in upstream-divergence.md (regenerated) and src/tests/client.rs (both kept, duplicate subcompositor code removed); pre-commit full route passed; just test 831 passed, 0 failed incl. doctests; merge review accepted with one minor, added to material-fc84e5
