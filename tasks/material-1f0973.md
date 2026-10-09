---
id: material-1f0973
title: Capture tooling nits from material-18c2a1's review
status: todo
priority: 4
size: xs
complexity: low
process: direct
created: 2026-10-09T04:22:12Z
updated: 2026-10-09T04:22:12Z
depends: []
tags: [capture, tooling]
source: material-18c2a1
agent: claude-code/claude-opus-5-5
---

Deferred minors from material-18c2a1's final review, none affecting behaviour:

- tools/test_glass_optic_smoke.py: the coverage-scan self-test copies the scan's regex; share one helper.
- focus-swap-clips.sh, ring-motion-clips.sh, drag-lag-clips.sh repeat the same Weston/niri log slice + capture_meta renderer lines and the inline load wait; lift them into the smoke lib.
- tools/test_capture_meta.py: the two no-nvidia-smi subtests share one self.run directory, so a failure in the first misreports the second. Give each its own run dir.
