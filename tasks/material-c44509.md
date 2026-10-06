---
id: material-c44509
title: capture.json records when the run finished and how long it took
status: done
priority: 3
size: s
complexity: low
process: direct
owner: capture-finish
created: 2026-09-24T10:01:25Z
updated: 2026-10-06T12:48:30Z
started: 2026-10-06T12:38:38Z
completed: 2026-10-06T12:48:30Z
depends: []
parent: material-2834d7
tags: [capture]
source: ops-2da76d
agent: claude-code/claude-opus-5-5
---

Why: capture.json has run.started but no finish time or duration, and its preflight and sub_runs carry no timestamps. Runs started from a TTY for headless lanes, or launched detached with setsid, are invisible to agent transcripts, so their length can be recovered only from file mtimes. Measured in ops (docs/reports/2026-09-24-quiet-run-estimates-vs-actuals.md there): 6 of 12 quiet runs were artifact-only, including material-f8b6e9's headless cost run and material-912dab's clips.

Done when the capture fixtures (ring-motion-clips.sh, glass-view-tilt-smoke.sh, and the capture-meta preflight they share) write run.finished and run.duration_s, a start and finish per sub_run, and the preflight verdict's time, including on refusal and on abort through a trap. A refused or aborted run still leaves a capture.json with its end time.

Where to look: docs/materials/scripts/ and the capture.json writer they share.

## Notes

- 2026-10-06T12:38:38Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:ccbf10ba-80e6-4814-9bd6-02a8e9cb3816","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T12:40:16Z (capture-finish): resumed
  provenance: {"harness_session":"claude-code:ccbf10ba-80e6-4814-9bd6-02a8e9cb3816","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T12:48:30Z (capture-finish): done
  provenance: {"harness_session":"claude-code:ccbf10ba-80e6-4814-9bd6-02a8e9cb3816","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-06T12:48:30Z (capture-finish): capture-meta stamps run.finished/duration_s on the first release (after its restore, in a finally so refused and aborted runs are stamped), preflight.at on every verdict, and sub_runs started (settle) plus finished/duration_s through a new finish subcommand; ring-motion, focus-swap and drag-lag clips finish each sequence, the optic smoke lib finishes in stop_nested and optic-settling's stop_drm; spec §2.3/§3 updated
  provenance: {"harness_session":"claude-code:ccbf10ba-80e6-4814-9bd6-02a8e9cb3816","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
