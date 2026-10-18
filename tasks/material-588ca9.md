---
id: material-588ca9
title: "Source: privacy indicators from cast and microphone state"
status: idea
priority: 2
size: s
reviewed: true
created: 2026-09-02T12:09:35Z
updated: 2026-10-18T23:00:39Z
depends: [material-a54d89]
parent: material-9b8bf9
tags: [signals, sources]
---

Outcome: is-window-cast-target and microphone or camera use become a calm always-on Notice from a native slot or a small PipeWire watcher, with a distinct response such as a warm rim. Decide whether cast state belongs in the native niri slot. Source: docs/materials/2026-09-02-material-signals-design.md section 11.

Established 2026-10-18 (cast half): a cast-target indication already works by configuration, with no signal. A window rule matching `is-window-cast-target=true` can select a `material` (niri-config/src/window_rule.rs). `Mapped::set_is_window_cast_target` sets `need_to_recompute_rules`, the path the tested `is-focused` material swap takes (src/tests/material.rs), and `Tile::refresh_material` re-resolves the material. Limits: the swap replaces the whole material with the accepted hard cut (7f5ee9e2), and `refresh_mapped_cast_window_rules` marks targets regardless of `is_active`, so a paused cast still matches. Output casts have no window to mark. The remaining cast decision is the owner's: whether that material swap is enough, or a native Notice slot is wanted so a warm rim layers over any material and eases in, and whether it follows selected or actively streaming targets.

Microphone, camera and cast consumers are different subjects: the cast IPC names a consumer `pid` for wlr-screencopy casts only (niri-ipc/src/lib.rs `Cast`), and nothing yet ties a capture stream to a window.

Waiting on: material-9cdccf, the PipeWire attribution probe, for the microphone, camera and cast-consumer half. Brief: docs/notes/2026-09-29-signal-sources-brief.md.

## Notes

- 2026-09-29T22:07:57Z (materials-26.04): scope: briefed; cast IPC exposes targets and activity while native cast-target rules do not write signals; separate captured-window indication from microphone/camera consumer attribution; brief: docs/notes/2026-09-29-signal-sources-brief.md
