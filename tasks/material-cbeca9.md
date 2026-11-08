---
id: material-cbeca9
title: Make the hue transition strip sample the workspace switch
status: todo
priority: 4
size: s
complexity: low
process: direct
created: 2026-10-17T15:13:27Z
updated: 2026-10-17T15:13:27Z
depends: []
parent: material-55380c
tags: [tooling]
agent: claude-code/claude-opus-5-5
---

Deferred from material-62347d's Task 8 and final reviews. docs/materials/scripts/noctalia-surfaces.sh run_strip shows the transition in 2 of 12 frames: shots start after the switch is under way and each is gated by wait_for polling, so 'about 0.3 s apart' is unmeasured, and the label calls a switch blend a crossfade. Start shooting before focus-workspace-up, drop the sleep, record real timestamps per frame in the label, and say switch blend. Also: run_strip skips the noctalia config validate run_case does; the SCALE override is undocumented and the look text hard-codes the cell percentage; the strip label hard-codes cyan/red and 0.03 though ACCENT/ACCENT2 are overridable; case_note text assumes the default CASES order.
