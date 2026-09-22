---
id: material-ff5de5
title: test_real_binary_against_fake_nvidia_smi flakes under full-suite load
status: todo
priority: 2
size: xs
complexity: low
process: direct
created: 2026-09-22T15:42:03Z
updated: 2026-09-22T15:42:03Z
depends: []
tags: [testing, bug, capture]
agent: claude-code/claude-opus-5
---

`tools/test_capture_meta.py::EndToEndTest::test_real_binary_against_fake_nvidia_smi` failed the pre-commit `just check` on 2026-09-22 with `AssertionError: 2 != 0` on the first `cm_run("preflight", ..., "--seconds", "1", sampling=True)`; exit 2 is a refusal. Run alone it passed three times in a row, in ~2.7 s each. The change being committed was task markdown only, so the failure cannot have come from the tree.

Cause is in the fixture, not in capture-meta: `cm_run(sampling=True)` arms a `threading.Timer(.2, ...)` that writes the faked `/proc/stat` busy counters while the preflight samples for 1 s. Under the load of the full suite the timer can fire late (or the preflight can read before it fires), so `cpu_busy_pct` comes out high enough to refuse. Everything else in that test is a fully faked `/proc` via `CAPTURE_META_PROC`, so the only real-time dependence is this timer.

Fix: make the sample deterministic — write the post-sample `/proc/stat` before the preflight starts rather than on a timer, or have capture-meta take the sample window from an env var the test pins, so the fixture no longer races wall-clock. Do not widen the refusal threshold to hide it.

Related: material-e2759c makes the refusal name the load it refused on, which would have turned this `2 != 0` into a readable message.
