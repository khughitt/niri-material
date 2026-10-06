---
id: material-0966ba
title: "Capture script hygiene: signed_diff's sign and Tracy-client IPC in trace_run"
status: todo
priority: 3
size: xs
complexity: low
process: direct
created: 2026-10-06T08:56:17Z
updated: 2026-10-06T08:56:17Z
depends: []
tags: [material]
agent: claude-code/claude-opus-5-5
---

signed_diff in glass-noise-site-smoke.sh and glass-noise-type-smoke.sh computes B - A + 0.5 (ImageMagick's Mathematics args apply to source minus destination), so grain-*.png is zero minus cell; sd is unaffected but any mean read from it has the wrong sign (it misled the 2026-10-05 residue analysis). Swap the operands or rename. glass-optic-smoke-lib.sh trace_run calls spawn_probe and steal_focus through niri-tracy, whose msg client spends ~0.5 s per call on Tracy calibration; use the plain binary as the IPC client as noise-placement-cost.sh now does.
