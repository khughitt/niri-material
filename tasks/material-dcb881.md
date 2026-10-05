---
id: material-dcb881
title: Screencast sample can take a frame before the damage it armed for
status: done
priority: 2
size: s
complexity: mid
process: direct
owner: fix/screencast-sampling-race
created: 2026-10-05T03:27:25Z
updated: 2026-10-05T11:04:43Z
started: 2026-10-05T10:49:28Z
completed: 2026-10-05T11:04:42Z
depends: []
parent: material-2834d7
tags: [testing]
agent: claude-code/claude-opus-5-5
---

In the dedicated-lane screencast case, cast_sample arms the consumer, causes damage, then takes the first frame after arming; nothing ties that frame to the damage. 2026-10-05 dev check (hold-pilot/pilot-2, binary bin-257f847b): sample-1's frame came 390 ms after its request (passing runs ~30 ms), and the probe's print_line change landed one sample late: client-1 == client-2, client-3 differs, verdict invalid. An immediate rerun (pilot-3) passed. Make the sample wait for a frame that contains the damage (e.g. frame after the client's commit, or compare against the previous sample until changed within a bound) so a slow frame cannot shift the pattern.

## Notes

- 2026-10-05T03:27:25Z (disturber-hold): concerns: material-188aaa extension — the hold's first evidence run exposed a sampling race in the screencast instrument
- 2026-10-05T09:10:26Z (materials-26.04): correction: the concerns: note above was written before material-188aaa closed, so by the note rules it is a review finding of material-188aaa's live validation, not a concern against closed work; outcome measures should not count it.
- 2026-10-05T10:49:28Z (materials-26.04): started
  provenance: {"harness_session":"codex:01a10ba8-8f94-7c11-b2f0-393e0df02dad","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-05T10:50:15Z (fix/screencast-sampling-race): resumed
  provenance: {"harness_session":"codex:01a10ba8-8f94-7c11-b2f0-393e0df02dad","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-05T10:53:32Z (fix/screencast-sampling-race): Root cause: SIGUSR1 precedes asynchronous kitty FIFO damage, so arrival time alone can select queued pre-damage pixels. Gate sample-2 on a changed probe text crop versus sample-1 (whole-frame changes can come from the thief); keep one bounded request and journal skipped arrivals. First/third samples observe the held probe rather than proving causal attribution to thief damage. Serialize request/completion publication across GLib and appsink threads.
- 2026-10-05T11:02:42Z (fix/screencast-sampling-race): review: impl round 1 — verdict: revise; findings: Important 1; reviewer: codex
- 2026-10-05T11:04:22Z (fix/screencast-sampling-race): review: impl round 2 — verdict: accept; findings: none; reviewer: codex
- 2026-10-05T11:04:22Z (fix/screencast-sampling-race): Round 1 disposition: synchronized handle_sample's pending snapshot and acquired the production request timestamp inside the sampler lock. Real private-bus/GStreamer regression with delayed reference I/O failed before the correction (nine missing journal entries) and passed afterward; focused consumer/analyzer run passed 53 tests.
- 2026-10-05T11:04:42Z (fix/screencast-sampling-race): Verification: just test-one consumer/analyzer (53 tests); just test-fast with full Python tooling override (405 tests, two existing optional retained-binary skips); stub dedicated screencast reaches crops and fails within the unchanged-client bound while reaping children. bash -n and git diff --check passed. No live DRM/desktop capture was run.
- 2026-10-05T11:04:42Z (fix/screencast-sampling-race): done
  provenance: {"harness_session":"codex:01a10ba8-8f94-7c11-b2f0-393e0df02dad","harness_session_source":"CODEX_SESSION_ID"}
- 2026-10-05T11:04:42Z (fix/screencast-sampling-race): Probe-update samples reject unchanged calibrated client crops until the update arrives, recording every skipped arrival for analyzer validation. Shared locking serializes request time/state and completion publication. Consumer, driver, analyzer regressions and updated sampling contract included; full tooling passed 405 tests.
  provenance: {"harness_session":"codex:01a10ba8-8f94-7c11-b2f0-393e0df02dad","harness_session_source":"CODEX_SESSION_ID"}
