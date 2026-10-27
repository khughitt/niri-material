---
id: material-349482
title: "Hold timers before the build and load wait, not after"
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-10-27T11:33:52Z
updated: 2026-10-27T11:33:52Z
depends: []
parent: material-2834d7
tags: [capture]
agent: claude-code
origin: mixed
---

On 2026-10-27 (material-3d48b0) the first desktop-idle pilot attempt was refused at await_load. obs-index.timer's hourly pass started inside the five-minute load window, adding about 40% CPU and IO pressure. The capture hold already stops 14 user timers and restores them at release, guarded by capture-meta's guard unit, but it starts only after build_binaries and await_load (docs/materials/scripts/glass-optic-smoke-lib.sh). So a timer that fires during the build or the load wait can still refuse the run, and a held unit already running at hold time (familiar-reap.service in that pilot) is only recorded. The user asked for interfering timers to be checked and paused before runs start, and re-enabled afterwards. Approach: either take the timer part of the hold before the build, or make await_load also wait for any active unit the hold would stop. Keep restore-on-release and the guard. System timers (hold.system_timers) are listed but not stopped, since that needs root: record whether any fired during the window, or propose a sudoers route. Done when a fixture run whose hold starts before a due timer is shown to keep that timer stopped through the build, the load wait and the run, and to restore it at release.

## Notes

- 2026-10-27T11:33:52Z (add): created
  provenance: {"harness_session":"claude-code:e4fb3aae-129a-4879-92bc-63b198023ece","harness_session_source":"CLAUDE_CODE_SESSION_ID","origin":"mixed"}
