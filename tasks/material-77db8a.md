---
id: material-77db8a
title: "Focus view tilt: perspective camera and an arrival swing on focus gain"
status: dropped
priority: 2
size: l
complexity: high
process: planned
owner: material-77db8a
created: 2026-09-23T00:02:27Z
updated: 2026-10-06T20:50:44Z
started: 2026-09-23T00:02:39Z
depends: []
tags: [rendering, camera]
agent: "claude-code/claude-opus-5-5[1m]"
spec: docs/specs/2026-09-22-focus-view-tilt-design.md
plan: docs/plans/2026-09-22-focus-view-tilt.md
---

Give the glass a sense of depth and movement: replace the orthographic view ray with a view vector (static perspective by screen position, plus a transient arrival swing on focus gain that settles to exactly zero). Optical only: window pixels and the slab silhouette do not move. Follow-ups: material-6901e0 (geometric tilt), material-d0518d (other lighting).

## Notes

- 2026-09-23T00:02:39Z (material-77db8a): started
  provenance: {"harness_session":"claude-code:51e7f95c-ffa2-4d57-bc6f-c71eb38b2b6b","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-23T11:30:27Z (material-77db8a): Prism keys: prism-96bc50
- 2026-09-23T11:34:21Z (material-77db8a): resumed
  provenance: {"harness_session":"claude-code:66e9c2d5-22bf-40bc-bb0a-ea5774109c31","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-23T11:34:21Z (material-77db8a): took over a live claim held by session 51e7f95c-ffa2-4d57-bc6f-c71eb38b2b6b (owner material-77db8a, host titan, pid 2101322, worktree /mnt/ssd3/work/niri-material/.worktrees/material-77db8a, since 2026-09-23T00:02:39Z, age 41502s, live)
- 2026-09-23T11:34:25Z (material-77db8a): parked (waiting on user, review): First: run material-cd0e1d's smoke and clips on an idle desktop GPU (rerun commands in its park note). Then the owner reviews the view-swing and view-persp grids (paths in the evidence doc), then live: merge material-77db8a to materials-26.04, package and install per the arch-package-pin rollout, set view-tilt about 20 and view-perspective about 0.3 in the material, judge on a focus change. Defaults and Prism preset values follow from that review.
  provenance: {"harness_session":"claude-code:66e9c2d5-22bf-40bc-bb0a-ea5774109c31","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T16:05:21Z (material-77db8a): resumed
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T16:05:21Z (material-77db8a): 2026-10-02: the cd0e1d smoke and clips already passed on 2026-09-24 (view-tilt-smoke-20461987); the old 'First: run' park step was stale. The branch is 239 commits behind materials-26.04 with 8 conflicting files (niri-config lib.rs and material/mod.rs, src/layout/tile.rs, main.frag, material-config.md, render-pipeline.md, ring-motion-clips.sh, upstream-divergence.md), so the grids no longer show current glass. Owner asked to queue the rerun for a quiet host.
- 2026-10-02T16:05:21Z (material-77db8a): parked (waiting on user, quiet; idle, 20 min): Agent, from .worktrees/material-77db8a: (0) merge materials-26.04 into the branch and resolve the 8 conflicts (just upstream-report for the divergence doc; ~45 min, needs no idle host), just test-fast. Then on an idle host with monitors off (niri msg action power-off-monitors after 15 s) and wali-rotate.timer stopped (restart after): (1) OUT=$NIRI_MATERIAL_WORK_ROOT/view-tilt-smoke-$(git rev-parse --short HEAD) CAPTURE_TASK=material-77db8a BASE_NIRI=$NIRI_MATERIAL_WORK_ROOT/view-tilt-base-niri docs/materials/scripts/glass-view-tilt-smoke.sh, expect renderer_verified_launches=15 and base_vs_plain_ae=0 (~10 min); (2) CAPTURE_TASK=material-77db8a SEQUENCES='view-swing-t10-p1 view-swing-t10-p3 view-swing-t25-p1 view-swing-t25-p3 view-swing-beam view-persp-k25-p1 view-persp-k25-p3 view-persp-k50-p1 view-persp-k50-p3' docs/materials/scripts/ring-motion-clips.sh, expect clips: OK (~6 min); write a run: note. Known pitfalls: tracy-csvexport hang (exports now bounded at 120 s; retry that launch once), wallpaper rotation mid-probe. Then owner reviews the view-swing/view-persp grids, then live rollout per the original plan (merge, package per arch-package-pin, view-tilt ~20, view-perspective ~0.3).
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T00:49:19Z (material-77db8a): material-77be96 (2026-10-02): this worktree's .cargo/config.toml (only [build] target-dir = "/mnt/ssd3/niri-material/target") was removed, so it builds into its own target/; the shared dir served other trees' builds as fresh. The next build here is a full one. Uncommitted record change: commit it with the next commit here.
- 2026-10-03T03:48:39Z (material-77db8a): resumed
  provenance: {"harness_session":"claude-code:e081ff94-3f7a-43b5-9bb1-a565c5568d01","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T06:48:46Z (material-77db8a): merge: materials-26.04 (d4a82742) merged as 07901170; 9 files, 22 hunks; layout/mod.rs adapted to mainline's local input_active; upstream-divergence regenerated; just test-fast 577 passed
- 2026-10-03T06:48:46Z (material-77db8a): run: 14 min (est 10, idle); smoke 14; passed: renderer_verified_launches=15, base_vs_plain_ae=0, neutral_vs_plain_ae=0, gpu persp +8.0% / deep +4.5% vs plain on llvmpipe. OUT view-tilt-smoke-07901170 (TTY, desktop stopped, wali-rotate.timer stopped and restored)
- 2026-10-03T06:48:46Z (material-77db8a): run: 5 min (est 6, idle); clips 5; passed: clips: OK, 9 view sequences; grids /mnt/ssd3/niri-material/ring-motion-clips-07901170/ring-clips-1565146-1791009768/view-swing-grid.png and /mnt/ssd3/niri-material/ring-motion-clips-07901170/ring-clips-1565146-1791009768/view-persp-grid.png
- 2026-10-03T06:48:46Z (material-77db8a): parked (waiting on user, review): Owner: review the view-swing and view-persp grids (and view-swing-beam.gif) in $NIRI_MATERIAL_WORK_ROOT/ring-motion-clips-07901170/ring-clips-1565146-1791009768/. Then agent: merge materials-26.04 again (it moved past d4a82742), live rollout per the original plan (merge, package per arch-package-pin, view-tilt ~20, view-perspective ~0.3).
  provenance: {"harness_session":"claude-code:e081ff94-3f7a-43b5-9bb1-a565c5568d01","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T20:50:29Z (material-77db8a): resumed
  provenance: {"harness_session":"claude-code:2b1bf82b-c436-48cf-9d01-3e841f43eb09","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T20:50:29Z (material-77db8a): review: impl round 1 — verdict: revise; findings: design 1; reviewer: human
- 2026-10-06T20:50:29Z (material-77db8a): 2026-10-06 owner verdict on the view-swing/view-persp grids: rejected. The terminal text stays static while the glass optics move, which breaks the goal of one cohesive material. The optical-only premise (spec: window pixels and the slab silhouette do not move) is the defect, not the tuning; no rollout. Branch material-77db8a kept unmerged as reference for material-6901e0.
- 2026-10-06T20:50:44Z (material-77db8a): dropped
  provenance: {"harness_session":"claude-code:2b1bf82b-c436-48cf-9d01-3e841f43eb09","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T20:50:44Z (material-77db8a): Rejected in review 2026-10-06: optical-only tilt leaves window text static while the glass moves, breaking the cohesive material. Branch material-77db8a kept unmerged as reference; the direction moves to material-6901e0.
  provenance: {"harness_session":"claude-code:2b1bf82b-c436-48cf-9d01-3e841f43eb09","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
