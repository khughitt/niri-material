---
id: material-3acc86
title: Verify held Aurora with a real screencast consumer
status: doing
priority: 1
size: m
complexity: mid
process: planned
owner: material-3acc86
created: 2026-10-02T08:06:31Z
updated: 2026-10-02T23:29:10Z
started: 2026-10-02T20:06:27Z
depends: [material-6bf694]
parent: material-f86183
tags: [performance]
source: "docs/materials/2026-09-30-optic-settling-evidence.md#unverified-screencast"
agent: codex
spec: docs/specs/2026-10-02-real-tty-settling-lane-design.md
plan: docs/plans/2026-10-02-real-tty-settling-lane.md
---

Remaining screencast acceptance from material-2ee11e. Run with an actual screencast consumer and client updates while input remains idle; screencopy is not equivalent. Extend the existing bounded capture driver and offline verdict for this control, pilot before matrix, and record consumer identity and capture interval. Assert held optic phase/uniforms and no resumed optic cadence while client frames and capture continue. Use an explicit worktree binary, fresh provenance and owned-process cleanup. Publish evidence and run notes, without claiming watt savings or changing the installed compositor.

## Notes

- 2026-10-02T20:06:27Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T20:06:27Z (materials-26.04): approach: headless-lane case with a private dbus-daemon per case (the nested niri must not take ScreenCast names on the desktop bus), debug dbus-interfaces-in-non-session-instances, PIPEWIRE_RUNTIME_DIR to the host daemon, and a Python consumer (Gio D-Bus ScreenCast session + GStreamer pipewiresrc ! appsink) journalling frame arrival on CLOCK_MONOTONIC. Owner chose installing gst-plugin-pipewire (official package) over a custom C consumer, 2026-10-02.
- 2026-10-02T20:06:31Z (material-3acc86): resumed
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T20:07:56Z (material-3acc86): finding 2026-10-02: the screencast cannot run in the headless lane. Nested niri (winit) has no GBM device: Backend::gbm_device returns None for Winit and Headless, and StartCast fails 'error starting screencast: no GBM device available' (smoke: private dbus-daemon, debug dbus-interfaces-in-non-session-instances, CreateSession/RecordMonitor winit/Start all succeed, then the cast stops). A real consumer needs the TTY/DRM backend, i.e. the same dedicated real-TTY lane as material-f7eb0b. The D-Bus + GStreamer consumer (docs/materials/scripts/screencast-consumer.py) is kept for that lane.
- 2026-10-02T20:07:56Z (material-3acc86): process -> planned: the real-TTY lane leaves unresolved design (DRM takeover of the desktop output, privileged VT switching, unlock authentication); spec §8 requires the cases but not their mechanics
- 2026-10-02T20:09:21Z (material-3acc86): parked (waiting on user, decision): Owner: decide how the real-TTY lane may drive the host (NOPASSWD rule for chvt or another VT-switch route; unlock via swaylock with a dedicated test password or a PAM-free lock client; whether the lane may take DP-1 from a stopped desktop). Then agent: brainstorm one spec covering material-f7eb0b and this task in .worktrees/material-3acc86 (consumer committed in 7c142439; needs gst-plugin-pipewire installed).
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T20:25:58Z (material-3acc86): resumed
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T20:25:58Z (material-3acc86): parked (waiting on user, review): Owner: review docs/specs/2026-10-02-real-tty-settling-lane-design.md in .worktrees/material-3acc86 (covers this task and material-f7eb0b), and complete the §3 host prerequisites (NOPASSWD chvt, gst-plugin-pipewire, ddcutil with i2c access). Then agent: write the implementation plan in the same worktree and submit it for review.
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T20:35:45Z (material-3acc86): review: spec round 1 — verdict: revise; findings: Important 2, Minor 1; reviewer: human
- 2026-10-02T20:35:45Z (material-3acc86): host prerequisites done 2026-10-02: gst-plugin-pipewire and ddcutil installed, NOPASSWD chvt rule, udev uaccess on i2c-dev applied; ddcutil detect finds DP-1 (Gigabyte G34WQC A, i2c-4), VCP 0x10 = 58 of 100
- 2026-10-02T20:37:26Z (material-3acc86): spec round 1 addressed: VT restoration on every exit (bounded, verified, reported in vt-restore.json, nonzero on failure, tested with stub chvt), cast pixel evidence (consumer samples next frame after a request; client crop differs, Aurora crop equal across three samples, no stale frame), §1 qualified for material-80caf4. Host setup recorded in docs/materials/capture-host-setup.md, linked from AGENTS.md (README is upstream's, left alone).
- 2026-10-02T20:38:19Z (material-3acc86): resumed
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T20:38:19Z (material-3acc86): parked (waiting on user, review): Owner: re-review docs/specs/2026-10-02-real-tty-settling-lane-design.md in .worktrees/material-3acc86 (round 2: VT restoration §4, cast pixel evidence §5, §1 qualified). On acceptance, agent writes the implementation plan in the same worktree and submits it for review.
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T20:43:38Z (material-3acc86): review: spec round 2 — verdict: revise; findings: Important 2; reviewer: human
- 2026-10-02T20:44:10Z (material-3acc86): spec round 2 addressed: each cast sample arms the request then causes bounded journaled client damage (thief line for sample-1/3, probe line for sample-2; niri skips undamaged cast frames), quiet >= 2 s before sample-3, client crop also equal sample-2 to sample-3; the development check now includes a complete screencast case
- 2026-10-02T20:44:10Z (material-3acc86): resumed
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T20:44:10Z (material-3acc86): parked (waiting on user, review): Owner: re-review docs/specs/2026-10-02-real-tty-settling-lane-design.md in .worktrees/material-3acc86 (round 3: damage-driven cast samples §5, screencast in the development check §7). On acceptance, agent writes the implementation plan in the same worktree and submits it for review.
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T20:53:31Z (material-3acc86): review: spec round 3 — verdict: accept; findings: none; reviewer: human
- 2026-10-02T20:53:31Z (material-3acc86): resumed
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T21:02:10Z (material-3acc86): parked (waiting on user, review): Owner: review docs/plans/2026-10-02-real-tty-settling-lane.md in .worktrees/material-3acc86 and choose an execution method (recommended: subagent-driven). Then agent executes Tasks 1-6 (material-76c35e, -47bb4a, -d4e0f6, -c096d2, -7b3838, -a3ca2b) in this worktree; Task 7 (material-6bf694) goes to tasks quiet.
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T22:07:23Z (material-3acc86): review: plan round 1 — verdict: revise; findings: Important 6; reviewer: human
- 2026-10-02T22:25:35Z (material-3acc86): plan round 1 addressed: node-discovery timeout removed after discovery (separate loop, tested); DMA-BUF import via GStreamer GL on headless EGL, probed on the desktop niri with owner approval (node 106, XR24:0x0300000000606012/0x0300000000e08014, RGB to EOS, rc 0); chvt bounded (timeout -k 1 2) with a hanging-stub test; TERM test reaps its sleep; edge test uses a separate resume stimulus (hold 4 s); driver TERM tests while casting, locked and switched away (restoration before release). Tasks 1, 3 and 4 tests run green against their planned code in a scratch copy.
- 2026-10-02T22:25:35Z (material-3acc86): resumed
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T22:25:35Z (material-3acc86): parked (waiting on user, review): Owner: re-review docs/plans/2026-10-02-real-tty-settling-lane.md in .worktrees/material-3acc86 (round 2) and choose an execution method (recommended: subagent-driven). Then agent executes Tasks 1-6 in this worktree; Task 7 (material-6bf694) goes to tasks quiet.
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T22:51:12Z (material-3acc86): review: plan round 2 — verdict: revise; findings: Important 3; reviewer: human
- 2026-10-02T22:52:42Z (material-3acc86): plan round 2 addressed: node is an int through ready and summary, with an end-to-end consumer test (fake ScreenCast service on a private bus, videotestsrc in place of PipeWire: ready, armed sample of padded rows, summary) run green from the plan text; probe f-string fixed and its session stops in finally; probe requires gst-launch exit 0 from a log, not a pipe. Every python block and heredoc in the plan now parses.
- 2026-10-02T22:52:42Z (material-3acc86): resumed
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T22:52:42Z (material-3acc86): parked (waiting on user, review): Owner: re-review docs/plans/2026-10-02-real-tty-settling-lane.md in .worktrees/material-3acc86 (round 3) and choose an execution method (recommended: subagent-driven). Then agent executes Tasks 1-6 in this worktree; Task 7 (material-6bf694) goes to tasks quiet.
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T23:28:37Z (material-3acc86): review: plan round 3 — verdict: revise; findings: Important 1, Minor 1; reviewer: human
- 2026-10-02T23:29:08Z (material-3acc86): plan round 3 addressed: Task 3 stages tools/fake_screencast.py; the end-to-end test also skips without PyGObject and Gio/GLib/Gst/GstVideo introspection. Extracted Task 3 tests green; every task's file list checked against its git add.
- 2026-10-02T23:29:08Z (material-3acc86): resumed
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T23:29:08Z (material-3acc86): parked (waiting on user, review): Owner: re-review docs/plans/2026-10-02-real-tty-settling-lane.md in .worktrees/material-3acc86 (round 4) and choose an execution method (recommended: subagent-driven). Then agent executes Tasks 1-6 in this worktree; Task 7 (material-6bf694) goes to tasks quiet.
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
