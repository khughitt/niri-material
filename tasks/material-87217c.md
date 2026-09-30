---
id: material-87217c
title: Track the NVIDIA application profile as installable source
status: done
priority: 3
size: xs
complexity: low
process: direct
owner: material-87217c
created: 2026-09-12T20:53:32Z
updated: 2026-09-30T10:20:50Z
started: 2026-09-30T10:17:12Z
completed: 2026-09-30T10:20:50Z
depends: []
tags: [configuration]
source: ops-9be6a9
---

/etc/nvidia/nvidia-application-profiles-rc.d/50-limit-free-buffer-pool-in-wayland-compositors.json sets GLVidHeapReuseRatio=0 for niri. docs/wiki/Nvidia.md documents writing it by hand; nothing tracks the installed file, so drift is invisible. Keep the JSON as a source file the wiki installs from (or decide it stays a wiki recipe and say so). Found by the titan settings inventory (ops hosts/titan/inventory.md, entry 'NVIDIA application profile').

## Notes

- 2026-09-12T20:55:09Z (materials-26.04): Filed from ops-9be6a9; the material pre-commit hook refused the commit: upstream-report reports docs/materials/upstream-divergence.md stale against the staged tree (run just upstream-report and stage the result). Task file left staged and uncommitted in the material checkout for the owner to commit.
- 2026-09-30T10:17:12Z (material-87217c): started
  provenance: {"harness_session":"codex:01a0f1cf-cd43-7080-bc37-3f241c0e4093","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-30T10:20:50Z (material-87217c): done
  provenance: {"harness_session":"codex:01a0f1cf-cd43-7080-bc37-3f241c0e4093","harness_session_source":"CODEX_SESSION_ID"}
- 2026-09-30T10:20:50Z (material-87217c): Tracked the installed NVIDIA profile under config/nvidia and changed the wiki to install that source
  provenance: {"harness_session":"codex:01a0f1cf-cd43-7080-bc37-3f241c0e4093","harness_session_source":"CODEX_SESSION_ID"}
