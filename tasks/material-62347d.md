---
id: material-62347d
title: Explore material treatment for Noctalia bars and panels
status: idea
priority: 2
created: 2026-10-12T10:50:31Z
updated: 2026-10-15T22:53:06Z
depends: [material-e27dbc, material-6b1c0b]
parent: material-55380c
tags: []
source: "mindful:thought:d6cae16e5bd867ec390c14ab866e7619"
agent: codex
---

Explore applying the existing material treatment to other Noctalia surfaces, including the bar and panels, to improve visual cohesion. Identify suitable surfaces and integration constraints during scoping; the capture asks for cohesion without settling the visual treatment or implementation.

Owner addition (2026-10-15): beyond a static variant of the material, the shell surfaces should follow the hue of the workspace's active coding-agent terminal, the familiar identity hue its glass already carries, so the whole workspace, not only the terminal, reinforces the project being worked on. The brief's "focused hue" material frames the routes and the semantics left for the owner.

## Notes

- 2026-10-14T10:42:33Z (materials-26.04): scope: briefed; layer rules and MappedLayer have background effects but no material pass at d9914514; representative panel geometry and alpha remain unknown; first compare existing effects before renderer design; waits on material-e27dbc under material-55380c; brief: docs/notes/2026-10-14-noctalia-material-brief.md
- 2026-10-15T22:20:43Z (materials-26.04): scope: briefed; owner added focused-hue linkage (shell surfaces follow the active agent terminal's familiar accent); evidence: accent already on niri IPC (Window.signal, Workspace.active_window_id), Noctalia 5.2.1 plugins can color their own UI from niri's event stream but cannot write the palette; filed material-6b1c0b for the compositor binding and now wait on it and material-e27dbc; brief: docs/notes/2026-10-14-noctalia-material-brief.md
- 2026-10-15T22:26:11Z (research/material-6b1c0b): material-6b1c0b finding: niri can bind a layer surface's accent to its output's active window with existing layout state and redraw triggers (Monitor::active_window holds under panel keyboard focus); a layer needs a per-output crossfade, animation coverage and fingerprinted damage; the binding belongs inside a layer-material design. The drawing still waits on material-e27dbc's comparison.
- 2026-10-15T22:53:06Z (materials-26.04): review: brief round 1 — verdict: revise; findings: should-fix 2; reviewer: unverified (pasted by the owner)
