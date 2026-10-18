---
id: material-9cdccf
title: Determine whether PipeWire audio and capture streams attribute to one niri window
status: todo
priority: 2
size: m
complexity: mid
process: direct
needs: [nested]
created: 2026-10-18T22:59:24Z
updated: 2026-10-18T22:59:24Z
depends: []
parent: material-9b8bf9
tags: [signals, sources]
source: docs/notes/2026-09-29-signal-sources-brief.md
agent: claude-code/claude-opus-5-5
---

Question: Can PipeWire playback streams (audio activity) and capture streams (microphone, camera, and the consumer of a niri screencast node) be attributed to exactly one niri window, and what does each stream's lifetime look like (start, cork or pause, end, client death)?

Where to start: docs/notes/2026-09-29-signal-sources-brief.md (its Audio and Privacy questions, and the terminal finding's rule of a unique match with ambiguity rejected, which material-9fb9ec designs). niri-ipc/src/lib.rs: `Window.pid` is the PID that opened the Wayland connection and is optional; `Cast` carries `pw_node_id`, `is_active`, and a consumer `pid` only for wlr-screencopy casts. `pw-dump` client and node properties (`application.process.id`, `pipewire.sec.pid`, `application.name`, `pipewire.access.portal.app_id`) and the links to a cast node. material-07bac9's attached probe is the fixture template (headless Weston, nested niri, private runtime directories, reaped processes).

Bound: Observation only. Run a nested niri on a per-run headless Weston unit with private runtime directories and a private PipeWire, WirePlumber and pipewire-pulse whose sinks and sources are null or virtual, so the host's audio graph, default devices and speakers are untouched. Cover a native player (pw-play or mpv), Firefox and a Chromium-based browser playing in a tab, one process with two windows, and a capture client (pw-record, and a browser microphone request against a virtual source). Cover a camera only if the private instance offers a virtual video source, and a screencast consumer only if the nested session can serve a PipeWire cast; otherwise record each as unknown. No watcher, IPC, config or rendering change.

Expected result: Record an attribution table per client (which property joins to which niri window: unique, ambiguous or missing) and the stream lifecycle events observed, on this task and in the brief's Audio and Privacy questions, with a recommendation per stream class: a watcher keyed on a unique match that rejects ambiguity, or no per-window attribution for that class.

Ideas it wakes: On completion, run `tasks note material-e91ae9 --no-reviewed "<finding>"` and `tasks note material-588ca9 --no-reviewed "<finding>"`, before this result's branch merges.
