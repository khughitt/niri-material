---
id: material-718ec9
title: "Ice: smoke with recorded draw cost"
status: todo
priority: 2
size: s
complexity: mid
process: direct
needs: [quiet]
created: 2026-10-15T18:39:51Z
updated: 2026-10-15T18:39:51Z
depends: [material-19dd35]
parent: material-bb3fe5
tags: [material, performance]
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-09-10-material-optics-design.md
---

Why: the spec requires the ice smoke to record its cost (material-optics spec §11); that is the only part of material-bb3fe5 that needs an idle host (material-5595fe note, 2026-10-03).

Where: glass-iridescence-smoke.sh is the closest template (neutral identity, effect in a region, GPU-median cost) over glass-optic-smoke-lib.sh with capture_preflight headless. Draw-time cost is a nested measurement under the TTY condition (docs/specs/2026-10-08-capture-host-conditions-design.md §3, §5). Pilot the smallest all-gates run first.

Done: an ice smoke script, its pilot and one full run recorded (run: notes), the cost in the ice evidence doc and docs/materials/performance.md.
