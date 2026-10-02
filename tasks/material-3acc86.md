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
updated: 2026-10-02T20:07:58Z
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
- 2026-10-02T20:06:31Z (material-3acc86): resumed
  provenance: {"harness_session":"claude-code:97c4dfdd-6c16-4570-9bae-fe749a5862c1","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-02T20:07:56Z (material-3acc86): finding 2026-10-02: the screencast cannot run in the headless lane. Nested niri (winit) has no GBM device: Backend::gbm_device returns None for Winit and Headless, and StartCast fails 'error starting screencast: no GBM device available' (smoke: private dbus-daemon, debug dbus-interfaces-in-non-session-instances, CreateSession/RecordMonitor winit/Start all succeed, then the cast stops). A real consumer needs the TTY/DRM backend, i.e. the same dedicated real-TTY lane as material-f7eb0b. The D-Bus + GStreamer consumer (docs/materials/scripts/screencast-consumer.py) is kept for that lane.
- 2026-10-02T20:07:56Z (material-3acc86): process -> planned: the real-TTY lane leaves unresolved design (DRM takeover of the desktop output, privileged VT switching, unlock authentication); spec §8 requires the cases but not their mechanics
