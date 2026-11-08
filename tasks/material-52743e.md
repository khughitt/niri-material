---
id: material-52743e
title: Cover the surface remap wiring end to end in niri tests
status: todo
priority: 3
size: s
complexity: mid
process: direct
created: 2026-10-17T15:13:21Z
updated: 2026-10-17T15:13:21Z
depends: []
parent: material-55380c
tags: [testing]
agent: claude-code/claude-opus-5-5
---

Deferred from material-62347d's reviews; all coverage gaps, no known defect. (1) Render-level: the bar and its xdg popups come out as LayerSurfaceRenderElement::SurfaceRemap and input-method popups do not (today only the popup_treatment unit test covers the decision). (2) A niri-level mid-switch blend: with animations on, an output between two workspaces hands its layers the lerped HueBlend, and an animating crossfade keeps are_animations_ongoing true. (3) an_exclusive_panel_keeps_the_hue opens the panel after the hue settles, so it cannot tell keyboard focus from the active window; add a case that opens it inside the dwell. (4) Assert the blend test's second workspace id, and that interrupted_fade_continues_from_its_point really samples mid-fade. (5) apply() on a hued (non-neutral) blend.
