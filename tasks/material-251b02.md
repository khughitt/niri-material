---
id: material-251b02
title: "Settling tests flake under single-process cargo test: timeline not running at start"
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-10-08T09:31:15Z
updated: 2026-10-08T09:31:15Z
depends: []
tags: [testing, flaky]
agent: claude-code/claude-opus-5-5
---

makepkg check() (cargo test --workspace --all-targets, one process, RAYON_NUM_THREADS=1) at 99196eba failed tests::attention_idle::optic_settling_edges_queue_a_redraw_and_drop_then_rearm_the_optic_timer (attention_idle.rs:325 'a running Aurora arms its next bucket') and optic_settling_removing_one_of_two_outputs_keeps_the_shared_timeline (:400 before.running) in a 44.9 s run right after the release build. The same tree passed: nextest (just test-fast 659/659, pre-push), cargo test --lib attention_idle alone, and two full cargo test --lib runs (35.7/36.0 s). Both assert the optic timeline is running right after aurora_fixture + set_input_idle_threshold(10 s) + dispatch(). The fixture config sets signal idle-after-ms 60, so a slow setup under load likely lets the 60 ms input-idle timer fire (or fire in dispatch) after the threshold change; set_threshold recomputes idle from elapsed time, so suspect the stale timer callback. Reproduce under load (e.g. stress + cargo test --lib attention_idle repeated), then make the fixture's idle threshold safe before setup or have the timer callback recheck the threshold.
