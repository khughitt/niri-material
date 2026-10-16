---
id: material-e27dbc
title: Identify a Noctalia panel and the smallest material-cohesion comparison
status: done
priority: 2
size: s
complexity: mid
process: direct
needs: [nested]
owner: research/material-e27dbc
created: 2026-10-14T10:42:18Z
updated: 2026-10-16T00:01:48Z
started: 2026-10-15T23:26:10Z
completed: 2026-10-16T00:01:48Z
depends: []
parent: material-55380c
tags: []
source: "docs/notes/2026-10-14-noctalia-material-brief.md#feasibility"
agent: codex
---

Question: Which Noctalia panel is a suitable first material target, and can existing layer effects expose the cohesion cues without a new material renderer?
Where to start: docs/notes/2026-10-14-noctalia-material-brief.md; niri-config/src/layer_rule.rs; src/layer/mapped.rs; docs/wiki/Configuration:-Layer-Rules.md. Inspect the relevant Noctalia source/configuration and its layer-surface namespace, buffer extent, alpha, background paint and input region. The known bar namespace is noctalia-bar-default; a panel namespace is not yet established.
Bound: Trace one representative panel and the bar, inventory the available styling and compositor inputs, and specify one nested comparison using existing effects. If a nested Noctalia instance is available, confirm the surface inventory there; record unavailable dependencies explicitly. No new renderer, Noctalia patch, live desktop changes, or broad theme rollout.
Expected result: Record a surface/constraint table and a recommendation on this task and in the brief. Supply exact settings and a verification recipe for a baseline-versus-treated panel comparison, naming any Noctalia styling changes required; conclude whether existing effects merit that experiment or a layer-material design is necessary. Appearance remains for the owner to judge.
Ideas it wakes: On completion, run tasks note material-62347d --no-reviewed "<finding>" before this result's branch merges.

## Notes

- 2026-10-15T23:26:10Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:c824d38e-5906-4679-a1cd-041a0cd9885d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-15T23:26:35Z (research/material-e27dbc): resumed
  provenance: {"harness_session":"claude-code:c824d38e-5906-4679-a1cd-041a0cd9885d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-16T00:00:33Z (research/material-e27dbc): attached: noctalia-cohesion-sheet.png (2048333 bytes): Owner sheet: does the treated shell (B) read as the same material as the terminal glass? A = live baseline (control); rows: 1 control center, 2 launcher, 3 bar; nested niri 1280x720, live wallpaper and palette
- 2026-10-16T00:01:36Z (research/material-e27dbc): finding: representative panel = control center (noctalia-attached-panel, shared with Prism's and every attached plugin panel, takes the bar's opacity); launcher (noctalia-panel, overlay) stands for floating panels. Noctalia already sends exact-shape ext-background-effect blur for bar, panels, OSD, toasts and dock; buffers are padded, so a footprint should follow that region. Live effective opacity: bar + attached 0.70 (settings override of config 0.93), floating glass 0.55, so niri effects show through 30-45%. Nested comparison (docs/materials/scripts/noctalia-surfaces.sh, sheet attached): treated = all panels 0.55, corners 20 px, glass noise 0.03 / saturation 0.95; change is small over dark glass (bar pixels 5-10 levels), corners match. Conclusion: existing effects reach translucency, grain, saturation and corner radius only; slab optics need a layer-material design if the owner judges B insufficient, and the hue needs a compositor change either way. No hued arm (palette transcode; would preview client tint, not route A).
- 2026-10-16T00:01:48Z (research/material-e27dbc): done
  provenance: {"harness_session":"claude-code:c824d38e-5906-4679-a1cd-041a0cd9885d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-16T00:01:48Z (research/material-e27dbc): Surface inventory, nested comparison recipe (docs/materials/scripts/noctalia-surfaces.sh) and owner sheet recorded; brief updated with the table, constraints and conclusion
  provenance: {"harness_session":"claude-code:c824d38e-5906-4679-a1cd-041a0cd9885d","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
