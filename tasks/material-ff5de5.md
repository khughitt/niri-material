---
id: material-ff5de5
title: test_real_binary_against_fake_nvidia_smi flakes under full-suite load
status: done
priority: 2
size: xs
complexity: low
process: direct
owner: materials-26.04
created: 2026-09-22T15:42:03Z
updated: 2026-09-27T12:01:32Z
started: 2026-09-27T11:58:10Z
completed: 2026-09-27T12:01:32Z
depends: []
tags: [testing, bug, capture]
agent: claude-code/claude-opus-5
---

`tools/test_capture_meta.py::EndToEndTest::test_real_binary_against_fake_nvidia_smi` failed the pre-commit `just check` on 2026-09-22 with `AssertionError: 2 != 0` on the first `cm_run("preflight", ..., "--seconds", "1", sampling=True)`; exit 2 is a refusal. Run alone it passed three times in a row, in ~2.7 s each. The change being committed was task markdown only, so the failure cannot have come from the tree.

Cause is in the fixture, not in capture-meta: `cm_run(sampling=True)` arms a `threading.Timer(.2, ...)` that writes the faked `/proc/stat` busy counters while the preflight samples for 1 s. Under the load of the full suite the timer can fire late (or the preflight can read before it fires), so `cpu_busy_pct` comes out high enough to refuse. Everything else in that test is a fully faked `/proc` via `CAPTURE_META_PROC`, so the only real-time dependence is this timer.

Fix: make the sample deterministic — write the post-sample `/proc/stat` before the preflight starts rather than on a timer, or have capture-meta take the sample window from an env var the test pins, so the fixture no longer races wall-clock. Do not widen the refusal threshold to hide it.

Related: material-e2759c makes the refusal name the load it refused on, which would have turned this `2 != 0` into a readable message.

## Notes

- 2026-09-25T15:31:28Z (materials-26.04): Recurred 2026-09-25 at the settle step (2 != 0) in hook-pre-commit, which now runs under host-budget run (background.slice, CPUWeight 30) while other budgeted jobs loaded the host; passed 6/6 alone, wrapped and unwrapped. The lower weight under contention may make the 0.2 s fake-stat timer race more frequent.
- 2026-09-27T11:58:10Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:2b4916ff-4c11-43fa-b146-a7c0e4cc51d8","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-27T12:01:32Z (material-ff5de5): done
  provenance: {"harness_session":"claude-code:2b4916ff-4c11-43fa-b146-a7c0e4cc51d8","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-27T12:01:32Z (material-ff5de5): The end-to-end fixture serves /proc/stat through a FIFO: the advanced counters replace it before capture-meta's first read is released, so the one-second sample no longer races a 0.2 s timer. A 0.5 s delayed tool start fails the old fixture (2 != 0) and passes the new one; each exit-code assertion now carries the tool's stderr.
  provenance: {"harness_session":"claude-code:2b4916ff-4c11-43fa-b146-a7c0e4cc51d8","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
