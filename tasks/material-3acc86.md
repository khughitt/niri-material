---
id: material-3acc86
title: Verify held Aurora with a real screencast consumer
status: doing
priority: 1
size: m
complexity: mid
process: direct
owner: materials-26.04
created: 2026-10-02T08:06:31Z
updated: 2026-10-02T20:06:28Z
started: 2026-10-02T20:06:27Z
depends: []
parent: material-f86183
tags: [performance]
source: "docs/materials/2026-09-30-optic-settling-evidence.md#unverified-screencast"
agent: codex
spec: docs/specs/2026-09-29-sustained-optic-settling-design.md
---

Remaining screencast acceptance from material-2ee11e. Run with an actual screencast consumer and client updates while input remains idle; screencopy is not equivalent. Extend the existing bounded capture driver and offline verdict for this control, pilot before matrix, and record consumer identity and capture interval. Assert held optic phase/uniforms and no resumed optic cadence while client frames and capture continue. Use an explicit worktree binary, fresh provenance and owned-process cleanup. Publish evidence and run notes, without claiming watt savings or changing the installed compositor.

## Notes

- 2026-10-02T20:06:27Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T20:06:27Z (materials-26.04): approach: headless-lane case with a private dbus-daemon per case (the nested niri must not take ScreenCast names on the desktop bus), debug dbus-interfaces-in-non-session-instances, PIPEWIRE_RUNTIME_DIR to the host daemon, and a Python consumer (Gio D-Bus ScreenCast session + GStreamer pipewiresrc ! appsink) journalling frame arrival on CLOCK_MONOTONIC. Owner chose installing gst-plugin-pipewire (official package) over a custom C consumer, 2026-10-02.
