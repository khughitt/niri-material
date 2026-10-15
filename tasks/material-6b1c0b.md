---
id: material-6b1c0b
title: Determine whether niri can bind a layer surface's accent to its output's active window
status: doing
priority: 2
size: s
complexity: mid
process: direct
owner: materials-26.04
created: 2026-10-15T22:20:17Z
updated: 2026-10-15T22:22:12Z
started: 2026-10-15T22:22:12Z
depends: []
parent: material-55380c
tags: []
source: docs/notes/2026-10-14-noctalia-material-brief.md
agent: claude-code/claude-opus-5-5
---

Question: Can niri give a layer surface the folded signal accent of its output's active workspace's active window, held while a panel holds keyboard focus and crossfaded on change, and what state, damage and lifecycle does that binding need?
Where to start: docs/notes/2026-10-14-noctalia-material-brief.md (focused hue); Niri::render_layer_normal and layers_in_render_order in src/niri.rs; src/layer/mapped.rs; MappedWindow::signal in src/window/mapped.rs; the accent endpoints and crossfade in src/render_helpers/material and src/render_helpers/signal.rs; Workspace.active_window_id in niri-ipc/src/lib.rs; docs/materials/render-pipeline.md.
Bound: Source tracing and at most one headless test that sets a window signal and reads the accent a layer on that output would resolve. No layer material pass, config surface, Noctalia change or live desktop use.
Expected result: Record on this task and in the brief: where the per-output active-window accent can be resolved each frame; what crossfade state a layer would hold and where it lives; how focus, active-window, workspace-switch and overview changes reach layer damage; how a "hold the workspace's last agent hue" rule could be computed from focus timestamps; and the smallest change set a layer-material design would need for the binding. Recommend whether the binding belongs inside that design or needs its own.
Ideas it wakes: On completion, run `tasks note material-62347d --no-reviewed "<finding>"` before this result's branch merges.

## Notes

- 2026-10-15T22:22:12Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:c824d38e-5906-4679-a1cd-041a0cd9885d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
