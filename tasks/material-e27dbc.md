---
id: material-e27dbc
title: Identify a Noctalia panel and the smallest material-cohesion comparison
status: todo
priority: 2
size: s
complexity: mid
process: direct
needs: [nested]
created: 2026-10-14T10:42:18Z
updated: 2026-10-14T10:42:18Z
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
