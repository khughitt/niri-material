---
id: material-80caf4
title: Verify optic settling under a real idle inhibitor
status: doing
priority: 1
size: s
complexity: mid
process: direct
owner: material-80caf4
created: 2026-10-02T08:06:31Z
updated: 2026-10-03T04:00:09Z
started: 2026-10-02T19:52:10Z
depends: []
parent: material-f86183
tags: [performance]
source: "docs/materials/2026-09-30-optic-settling-evidence.md#unverified-idle-inhibitor"
agent: codex
spec: docs/specs/2026-09-29-sustained-optic-settling-design.md
---

Remaining idle-inhibitor capture acceptance from material-2ee11e; the real-handler fixture already covers the wiring. Extend the existing bounded driver with a real idle-inhibitor client, prove that inhibition does not notify input activity or restart Aurora deadlines while held, and retain normal client updates. Pilot before matrix, explicit worktree binary, trace liveness controls, hashes, cleanup and run notes. Update the acceptance evidence; do not substitute the fixture test for the missing lifecycle capture.

## Notes

- 2026-10-02T19:52:10Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T19:52:10Z (materials-26.04): approach: real Wayland idle-inhibit client (C, built per run with wayland-scanner; no client is installed), a Tracy 'IdleInhibit inhibited=N' message on niri's inhibited-state change so the trace proves inhibition took hold, the idle-inhibitor case moved into the headless lane, and an analyzer check that a stimulus's required messages fall inside its journaled window. The pilot needs the headless host and is parked for tasks quiet.
- 2026-10-02T19:52:14Z (material-80caf4): resumed
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T20:03:43Z (material-80caf4): parked (waiting on user, quiet; headless, 75 min): Agent, from .worktrees/material-80caf4 on a TTY with the desktop stopped (the headless lane's host, as for pilot-20261002-1): (1) at the worktree's HEAD, cargo build --release --features profile-with-tracy, snapshot the binary to $NIRI_MATERIAL_WORK_ROOT/optic-settling/bin-<short HEAD>/niri-tracy with a niri-tracy.identity.json sidecar (source_commit = HEAD, features [profile-with-tracy], binary_sha256), as bin-2f3b5b6a was (build ~4 min; wait for load1 < 1.0 afterwards). (2) Smallest end-to-end check: OUT=$NIRI_MATERIAL_WORK_ROOT/optic-settling/dev-<date>-inhibit CASES=idle-inhibitor CAPTURE_TASK=material-80caf4 NIRI_BIN=<snapshot> docs/materials/scripts/optic-settling-smoke.sh pilot --lane headless (~3 min); read analysis.json: one pause edge, IdleInhibit inhibited=1 and =0 inside the inhibit and release windows, client draws, no resume edge. (3) Full pilot with a fresh OUT, no CASES (~27 min), then matrix with PILOT_DIR (~41 min); write a run: note per attempt. (4) Update docs/materials/2026-09-30-optic-settling-evidence.md (identity table to the new binary, verdict rows, idle-inhibitor moved out of Unverified), close material-80caf4, merge into materials-26.04. Known pitfalls: tracy-csvexport hang (exports bounded at 120 s), schedule slip under load fails a case.
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T03:47:29Z (material-80caf4): resumed
  provenance: {"harness_session":"claude-code:e081ff94-3f7a-43b5-9bb1-a565c5568d01","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-03T04:00:08Z (material-80caf4): run: 2 min (est 3, headless); dev check 2; failed: analyzer 'idle-inhibitor: short observation' — the case's stimuli at 21/26/29 s left no stimulus-free 5 s span after the ~17.3 s pause (longest 1.95 s); IdleInhibit inhibited=1 and =0 both traced. Binary bin-8e9b1e2d, OUT dev-20261003-inhibit
- 2026-10-03T04:00:08Z (material-80caf4): schedule fix: idle-inhibitor stimuli moved to 25/30/33 s (the lane's 5 s hold after the pause) and capture_s 36 -> 44
